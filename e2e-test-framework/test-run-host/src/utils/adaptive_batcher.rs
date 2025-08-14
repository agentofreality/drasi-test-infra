use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::timeout;

/// Throughput levels for adaptive batching
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThroughputLevel {
    Idle,     // < 1 msg/sec
    Low,      // 1-100 msgs/sec
    Medium,   // 100-1,000 msgs/sec
    High,     // 1,000-10,000 msgs/sec
    Burst,    // > 10,000 msgs/sec
}

/// Configuration for adaptive batching
#[derive(Debug, Clone)]
pub struct AdaptiveBatchConfig {
    /// Maximum number of items in a batch
    pub max_batch_size: usize,
    /// Minimum number of items to consider efficient batching
    pub min_batch_size: usize,
    /// Maximum time to wait for batch to fill
    pub max_wait_time: Duration,
    /// Minimum time to wait (allows for message coalescing)
    pub min_wait_time: Duration,
    /// Window size for throughput calculation
    pub throughput_window: Duration,
    /// Enable adaptive parameter adjustment
    pub adaptive_enabled: bool,
}

impl Default for AdaptiveBatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 1000,
            min_batch_size: 10,
            max_wait_time: Duration::from_millis(100),
            min_wait_time: Duration::from_millis(1),
            throughput_window: Duration::from_secs(5),
            adaptive_enabled: true,
        }
    }
}

/// Monitors throughput and provides traffic classification
pub struct ThroughputMonitor {
    window_size: Duration,
    events: Vec<(Instant, usize)>, // (timestamp, batch_size)
}

impl ThroughputMonitor {
    pub fn new(window_size: Duration) -> Self {
        Self {
            window_size,
            events: Vec::new(),
        }
    }

    pub fn record_batch(&mut self, batch_size: usize) {
        let now = Instant::now();
        self.events.push((now, batch_size));
        
        // Clean old events outside the window
        let cutoff = now - self.window_size;
        self.events.retain(|(timestamp, _)| *timestamp > cutoff);
    }

    pub fn get_throughput_level(&self) -> ThroughputLevel {
        if self.events.is_empty() {
            return ThroughputLevel::Idle;
        }

        let now = Instant::now();
        let window_start = now - self.window_size;
        
        // Calculate messages per second
        let total_messages: usize = self.events
            .iter()
            .filter(|(timestamp, _)| *timestamp > window_start)
            .map(|(_, size)| size)
            .sum();
        
        let elapsed = self.window_size.as_secs_f64();
        let msgs_per_sec = (total_messages as f64) / elapsed;
        
        match msgs_per_sec {
            x if x < 1.0 => ThroughputLevel::Idle,
            x if x < 100.0 => ThroughputLevel::Low,
            x if x < 1000.0 => ThroughputLevel::Medium,
            x if x < 10000.0 => ThroughputLevel::High,
            _ => ThroughputLevel::Burst,
        }
    }

    pub fn get_messages_per_second(&self) -> f64 {
        if self.events.is_empty() {
            return 0.0;
        }

        let now = Instant::now();
        let window_start = now - self.window_size;
        
        let total_messages: usize = self.events
            .iter()
            .filter(|(timestamp, _)| *timestamp > window_start)
            .map(|(_, size)| size)
            .sum();
        
        let elapsed = self.window_size.as_secs_f64();
        (total_messages as f64) / elapsed
    }
}

/// Adaptive batcher that adjusts batch size and timing based on throughput
pub struct AdaptiveBatcher<T> {
    receiver: mpsc::Receiver<T>,
    config: AdaptiveBatchConfig,
    monitor: ThroughputMonitor,
    current_batch_size: usize,
    current_wait_time: Duration,
}

impl<T> AdaptiveBatcher<T> {
    pub fn new(receiver: mpsc::Receiver<T>, config: AdaptiveBatchConfig) -> Self {
        let monitor = ThroughputMonitor::new(config.throughput_window);
        Self {
            receiver,
            current_batch_size: config.min_batch_size,
            current_wait_time: config.min_wait_time,
            monitor,
            config,
        }
    }

    /// Adjust batching parameters based on current throughput
    fn adapt_parameters(&mut self) {
        if !self.config.adaptive_enabled {
            return;
        }

        let level = self.monitor.get_throughput_level();
        let msgs_per_sec = self.monitor.get_messages_per_second();
        
        match level {
            ThroughputLevel::Idle => {
                // Optimize for latency - send immediately
                self.current_batch_size = self.config.min_batch_size;
                self.current_wait_time = self.config.min_wait_time;
            }
            ThroughputLevel::Low => {
                // Small batches, minimal wait
                self.current_batch_size = (self.config.min_batch_size * 2).min(self.config.max_batch_size);
                self.current_wait_time = Duration::from_millis(1).max(self.config.min_wait_time);
            }
            ThroughputLevel::Medium => {
                // Moderate batching
                self.current_batch_size = ((self.config.max_batch_size - self.config.min_batch_size) / 4 + self.config.min_batch_size)
                    .min(self.config.max_batch_size);
                self.current_wait_time = Duration::from_millis(10).max(self.config.min_wait_time).min(self.config.max_wait_time);
            }
            ThroughputLevel::High => {
                // Larger batches for efficiency
                self.current_batch_size = ((self.config.max_batch_size - self.config.min_batch_size) / 2 + self.config.min_batch_size)
                    .min(self.config.max_batch_size);
                self.current_wait_time = Duration::from_millis(25).max(self.config.min_wait_time).min(self.config.max_wait_time);
            }
            ThroughputLevel::Burst => {
                // Maximum throughput mode
                self.current_batch_size = self.config.max_batch_size;
                self.current_wait_time = Duration::from_millis(50).max(self.config.min_wait_time).min(self.config.max_wait_time);
            }
        }
        
        log::trace!(
            "Adapted batching parameters - Level: {:?}, Rate: {:.1} msgs/sec, Batch: {}, Wait: {:?}",
            level, msgs_per_sec, self.current_batch_size, self.current_wait_time
        );
    }

    /// Collect the next batch of items
    pub async fn next_batch(&mut self) -> Option<Vec<T>> {
        let mut batch = Vec::new();
        
        // First, wait for at least one message
        match self.receiver.recv().await {
            Some(item) => batch.push(item),
            None => return None, // Channel closed
        }
        
        // Adapt parameters based on recent throughput
        self.adapt_parameters();
        
        // Now we have at least one message, decide how to batch
        let deadline = Instant::now() + self.current_wait_time;
        
        // Try to collect more messages up to current batch size
        while batch.len() < self.current_batch_size {
            // Check if more messages are immediately available
            match self.receiver.try_recv() {
                Ok(item) => {
                    batch.push(item);
                    // If we have a good batch size, consider sending
                    if batch.len() >= self.current_batch_size / 2 {
                        // Check if channel has many waiting messages (burst detection)
                        let pending = self.estimate_pending();
                        if pending > self.current_batch_size * 2 {
                            // Many messages waiting, fill the batch completely
                            while batch.len() < self.current_batch_size {
                                match self.receiver.try_recv() {
                                    Ok(item) => batch.push(item),
                                    Err(_) => break,
                                }
                            }
                            break;
                        }
                    }
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // No immediate messages, wait up to deadline
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        break;
                    }
                    
                    match timeout(remaining, self.receiver.recv()).await {
                        Ok(Some(item)) => batch.push(item),
                        Ok(None) => break, // Channel closed
                        Err(_) => break,    // Timeout
                    }
                }
                Err(mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        
        // Record batch for throughput monitoring
        self.monitor.record_batch(batch.len());
        
        log::debug!(
            "Adaptive batch collected - Size: {}, Target: {}, Wait: {:?}, Level: {:?}",
            batch.len(),
            self.current_batch_size,
            self.current_wait_time,
            self.monitor.get_throughput_level()
        );
        
        Some(batch)
    }
    
    /// Estimate number of pending messages (heuristic)
    fn estimate_pending(&self) -> usize {
        // Since we can't peek at the channel without consuming messages,
        // we use throughput level as a heuristic
        let throughput = self.monitor.get_throughput_level();
        match throughput {
            ThroughputLevel::Burst => 100,
            ThroughputLevel::High => 50,
            ThroughputLevel::Medium => 20,
            ThroughputLevel::Low => 5,
            ThroughputLevel::Idle => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_throughput_monitor() {
        let mut monitor = ThroughputMonitor::new(Duration::from_secs(1));
        
        // Initially idle
        assert_eq!(monitor.get_throughput_level(), ThroughputLevel::Idle);
        
        // Add some events
        monitor.record_batch(10);
        sleep(Duration::from_millis(100)).await;
        monitor.record_batch(10);
        
        // Should be low throughput (20 msgs in 1 sec window)
        assert_eq!(monitor.get_throughput_level(), ThroughputLevel::Low);
    }

    #[tokio::test]
    async fn test_adaptive_batcher_low_traffic() {
        let (tx, rx) = mpsc::channel(100);
        let config = AdaptiveBatchConfig::default();
        let mut batcher = AdaptiveBatcher::new(rx, config);
        
        // Send a few messages slowly
        tx.send(1).await.unwrap();
        tx.send(2).await.unwrap();
        
        let batch = batcher.next_batch().await.unwrap();
        assert!(batch.len() <= 10); // Should batch small for low traffic
    }

    #[tokio::test]
    async fn test_adaptive_batcher_burst() {
        let (tx, rx) = mpsc::channel(1000);
        let config = AdaptiveBatchConfig::default();
        let mut batcher = AdaptiveBatcher::new(rx, config);
        
        // Send many messages quickly
        for i in 0..100 {
            tx.send(i).await.unwrap();
        }
        
        let batch = batcher.next_batch().await.unwrap();
        assert!(batch.len() > 10); // Should batch larger for burst
    }
}