// Copyright 2025 The Drasi Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_trait::async_trait;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinHandle;

use test_data_store::{
    scripts::SourceChangeEvent,
    test_repo_storage::models::HttpSourceChangeDispatcherDefinition,
    test_run_storage::TestRunSourceStorage,
};

use crate::utils::{AdaptiveBatcher, AdaptiveBatchConfig};
use super::SourceChangeDispatcher;

use log::{debug, error, info};
use serde::{Deserialize, Serialize};

/// Batch event request that wraps multiple events
#[derive(Debug, Serialize, Deserialize)]
struct BatchEventRequest {
    events: Vec<HttpChangeEvent>,
}

/// HTTP change event format matching Drasi Server's expectation
#[derive(Debug, Serialize, Deserialize, Clone)]
struct HttpChangeEvent {
    op: String,
    #[serde(rename = "reactivatorStart_ns", skip_serializing_if = "Option::is_none")]
    reactivator_start_ns: Option<i64>,
    #[serde(rename = "reactivatorEnd_ns", skip_serializing_if = "Option::is_none")]
    reactivator_end_ns: Option<i64>,
    payload: serde_json::Value,
}

/// Response from the HTTP endpoint
#[derive(Debug, Deserialize)]
struct EventResponse {
    success: bool,
    message: String,
    error: Option<String>,
}

pub struct AdaptiveHttpSourceChangeDispatcher {
    url: String,
    port: u16,
    endpoint: String,
    batch_endpoint: String,
    timeout_seconds: u64,
    source_id: String,
    adaptive_config: AdaptiveBatchConfig,
    // Channel for sending events to the batcher
    event_tx: Option<mpsc::Sender<SourceChangeEvent>>,
    // Handle to the background batcher task
    batcher_handle: Option<Arc<Mutex<Option<JoinHandle<()>>>>>,
    client: Arc<Client>,
    batch_enabled: bool,
}

impl AdaptiveHttpSourceChangeDispatcher {
    pub fn new(
        definition: &HttpSourceChangeDispatcherDefinition,
        _storage: TestRunSourceStorage,
    ) -> anyhow::Result<Self> {
        info!("Creating AdaptiveHttpSourceChangeDispatcher");
        
        // Configure adaptive batching
        let mut adaptive_config = AdaptiveBatchConfig::default();
        
        // Check if we have explicit batch settings
        if let Some(batch_size) = definition.batch_size {
            adaptive_config.max_batch_size = batch_size as usize;
            adaptive_config.min_batch_size = (batch_size / 10).max(1) as usize;
        }
        if let Some(timeout_ms) = definition.batch_timeout_ms {
            adaptive_config.max_wait_time = Duration::from_millis(timeout_ms);
            adaptive_config.min_wait_time = Duration::from_millis(timeout_ms / 10);
        }
        
        // Check if adaptive mode is enabled (default true for adaptive dispatcher)
        let adaptive_enabled = definition.adaptive_enabled.unwrap_or(true);
        adaptive_config.adaptive_enabled = adaptive_enabled;
        
        // Determine if batch endpoint should be used
        let batch_enabled = definition.batch_events.unwrap_or(true);
        
        // Extract source_id from definition or use default
        let source_id = definition.source_id.clone()
            .unwrap_or_else(|| "test-source".to_string());
        
        // Construct endpoints
        let endpoint = if let Some(ep) = &definition.endpoint {
            ep.clone()
        } else {
            format!("/sources/{}/events", source_id)
        };
        
        // For Drasi Server adaptive source, batch endpoint has /batch suffix
        let batch_endpoint = format!("{}/batch", endpoint);
        
        // Create HTTP client with connection pooling (HTTP/1.1 for compatibility)
        let client = Client::builder()
            .timeout(Duration::from_secs(definition.timeout_seconds.unwrap_or(30)))
            .pool_idle_timeout(Duration::from_secs(90))
            .pool_max_idle_per_host(10)
            // Don't use http2_prior_knowledge as it can cause broken pipe errors
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Ok(Self {
            url: definition.url.clone(),
            port: definition.port,
            endpoint,
            batch_endpoint,
            timeout_seconds: definition.timeout_seconds.unwrap_or(30),
            source_id,
            adaptive_config,
            event_tx: None,
            batcher_handle: None,
            client: Arc::new(client),
            batch_enabled,
        })
    }
    
    fn start_batcher(&mut self) -> anyhow::Result<()> {
        if self.batcher_handle.is_some() {
            return Ok(()); // Already started
        }
        
        // Create channel for batching
        let (event_tx, event_rx) = mpsc::channel(1000);
        self.event_tx = Some(event_tx);
        
        // Clone values for the spawned task
        let url = self.url.clone();
        let port = self.port;
        let endpoint = self.endpoint.clone();
        let batch_endpoint = self.batch_endpoint.clone();
        let source_id = self.source_id.clone();
        let adaptive_config = self.adaptive_config.clone();
        let client = self.client.clone();
        let batch_enabled = self.batch_enabled;
        
        // Spawn batcher task
        let handle = tokio::spawn(async move {
            let mut batcher = AdaptiveBatcher::new(event_rx, adaptive_config);
            let mut successful_batches = 0u64;
            let mut failed_batches = 0u64;
            let mut total_events = 0u64;
            
            info!("Adaptive HTTP batcher started for source {}", source_id);
            
            while let Some(batch) = batcher.next_batch().await {
                if batch.is_empty() {
                    continue;
                }
                
                let batch_size = batch.len();
                total_events += batch_size as u64;
                
                debug!("Adaptive HTTP batch ready with {} events", batch_size);
                
                // Convert events to HttpChangeEvent format
                let http_events: Vec<HttpChangeEvent> = batch.into_iter()
                    .filter_map(|event| {
                        // Convert SourceChangeEvent to HttpChangeEvent
                        // The payload already has the correct structure, just convert it to Value
                        match serde_json::to_value(&event.payload) {
                            Ok(payload_value) => {
                                // Ensure the payload has the expected structure
                                if payload_value.get("source").is_some() {
                                    Some(HttpChangeEvent {
                                        op: event.op,
                                        reactivator_start_ns: Some(event.reactivator_start_ns as i64),
                                        reactivator_end_ns: Some(event.reactivator_end_ns as i64),
                                        payload: payload_value,
                                    })
                                } else {
                                    error!("Payload missing 'source' field: {:?}", payload_value);
                                    None
                                }
                            },
                            Err(e) => {
                                error!("Failed to serialize event payload: {}", e);
                                None
                            }
                        }
                    })
                    .collect();
                
                if http_events.is_empty() {
                    continue;
                }
                
                // Send batch or individual events
                let success = if batch_enabled && http_events.len() > 1 {
                    // Send as batch - Drasi Server adaptive source expects BatchEventRequest
                    let batch_url = format!("{}:{}{}", url, port, batch_endpoint);
                    let batch_request = BatchEventRequest {
                        events: http_events.clone(),
                    };
                    
                    // Log the batch being sent for debugging
                    debug!("Sending batch to {}: {}", 
                           batch_url,
                           serde_json::to_string_pretty(&batch_request).unwrap_or_else(|_| "Failed to serialize".to_string()));
                    
                    match client.post(&batch_url)
                        .json(&batch_request)
                        .send()
                        .await {
                        Ok(response) => {
                            let status = response.status();
                            if status.is_success() {
                                debug!("Batch of {} events sent successfully", batch_size);
                                true
                            } else {
                                // Get response body for debugging
                                let body = response.text().await.unwrap_or_else(|_| "Failed to get response body".to_string());
                                error!("Batch request failed with status: {} - Response: {}", status, body);
                                false
                            }
                        }
                        Err(e) => {
                            error!("Failed to send batch: {}", e);
                            false
                        }
                    }
                } else {
                    // Send individual events
                    let single_url = format!("{}:{}{}", url, port, endpoint);
                    let mut all_success = true;
                    
                    for event in http_events {
                        match client.post(&single_url)
                            .json(&event)
                            .send()
                            .await {
                            Ok(response) => {
                                if !response.status().is_success() {
                                    error!("Event request failed with status: {}", response.status());
                                    all_success = false;
                                }
                            }
                            Err(e) => {
                                error!("Failed to send event: {}", e);
                                all_success = false;
                            }
                        }
                    }
                    all_success
                };
                
                if success {
                    successful_batches += 1;
                } else {
                    failed_batches += 1;
                }
                
                if (successful_batches + failed_batches) % 100 == 0 {
                    info!(
                        "Adaptive HTTP metrics - Successful: {}, Failed: {}, Total events: {}",
                        successful_batches, failed_batches, total_events
                    );
                }
            }
            
            info!(
                "Adaptive HTTP batcher completed - Successful: {}, Failed: {}, Total events: {}",
                successful_batches, failed_batches, total_events
            );
        });
        
        self.batcher_handle = Some(Arc::new(Mutex::new(Some(handle))));
        Ok(())
    }
    
    async fn send_single_event(&self, event: &SourceChangeEvent) -> anyhow::Result<()> {
        let url = format!("{}:{}{}", self.url, self.port, self.endpoint);
        
        // Convert to HttpChangeEvent format
        let payload = serde_json::to_value(&event.payload)?;
        
        let http_event = HttpChangeEvent {
            op: event.op.clone(),
            reactivator_start_ns: Some(event.reactivator_start_ns as i64),
            reactivator_end_ns: Some(event.reactivator_end_ns as i64),
            payload,
        };
        
        let response = self.client
            .post(&url)
            .json(&http_event)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "HTTP request failed with status {}: {}",
                status,
                error_text
            ));
        }
        
        Ok(())
    }
}

#[async_trait]
impl SourceChangeDispatcher for AdaptiveHttpSourceChangeDispatcher {
    async fn close(&mut self) -> anyhow::Result<()> {
        info!("Closing AdaptiveHttpSourceChangeDispatcher");
        
        // Close the event channel to signal batcher to stop
        self.event_tx = None;
        
        // Wait for batcher to complete if running
        if let Some(handle_arc) = self.batcher_handle.take() {
            let mut handle_guard = handle_arc.lock().await;
            if let Some(join_handle) = handle_guard.take() {
                drop(handle_guard); // Release lock before awaiting
                // Don't wait forever - use a timeout
                let _ = tokio::time::timeout(Duration::from_secs(5), join_handle).await;
            }
        }
        
        Ok(())
    }
    
    async fn dispatch_source_change_events(
        &mut self,
        events: Vec<&SourceChangeEvent>,
    ) -> anyhow::Result<()> {
        if events.is_empty() {
            return Ok(());
        }
        
        // Start batcher if not already running
        if self.batcher_handle.is_none() {
            self.start_batcher()?;
        }
        
        // If we have a batch channel, use adaptive batching
        if let Some(ref tx) = self.event_tx {
            for event in events {
                if tx.send(event.clone()).await.is_err() {
                    error!("Failed to send event to batcher");
                    // Fall back to direct sending
                    self.send_single_event(event).await?;
                }
            }
        } else {
            // Fallback: send events directly
            for event in events {
                self.send_single_event(event).await?;
            }
        }
        
        Ok(())
    }
}