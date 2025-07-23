use super::SourceChangeDispatcher;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use test_data_store::{
    scripts::SourceChangeEvent, test_repo_storage::models::HttpSourceChangeDispatcherDefinition,
    test_run_storage::TestRunSourceStorage,
};
use tracing::{debug, error, trace};

#[derive(Debug, Serialize, Deserialize)]
struct TransformedEvent {
    op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    node_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<Value>,
}

impl TransformedEvent {
    fn from_source_change_event(event: &SourceChangeEvent) -> Result<Self> {
        let op = event.op.to_lowercase();
        
        match op.as_str() {
            "i" | "insert" => {
                // For insert operations, use the 'after' data
                let data = &event.payload.after;
                Ok(TransformedEvent {
                    op: "insert".to_string(),
                    data: Some(data.clone()),
                    node_type: None,
                    before: None,
                    after: None,
                })
            }
            "u" | "update" => {
                // For update operations, include both before and after
                Ok(TransformedEvent {
                    op: "update".to_string(),
                    data: None,
                    node_type: Some("node".to_string()),
                    before: Some(event.payload.before.clone()),
                    after: Some(event.payload.after.clone()),
                })
            }
            "d" | "delete" => {
                // For delete operations, use the 'before' data
                let data = &event.payload.before;
                Ok(TransformedEvent {
                    op: "delete".to_string(),
                    data: Some(data.clone()),
                    node_type: None,
                    before: None,
                    after: None,
                })
            }
            _ => anyhow::bail!("Unknown operation type: {}", event.op),
        }
    }
}

pub struct HttpSourceChangeDispatcherSettings {
    pub url: String,
    pub port: u16,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub batch_events: bool,
    pub source_id: String,
}

impl HttpSourceChangeDispatcherSettings {
    pub fn new(definition: &HttpSourceChangeDispatcherDefinition, source_id: String) -> Self {
        // If endpoint is provided, use it as-is, otherwise construct from source_id
        let endpoint = if let Some(ep) = &definition.endpoint {
            ep.clone()
        } else {
            format!("/sources/{}/events", source_id)
        };

        Self {
            url: definition.url.clone(),
            port: definition.port,
            endpoint,
            timeout_seconds: definition.timeout_seconds.unwrap_or(30),
            batch_events: definition.batch_events.unwrap_or(true),
            source_id,
        }
    }

    pub fn full_url(&self) -> String {
        format!("{}:{}{}", self.url, self.port, self.endpoint)
    }
}

pub struct HttpSourceChangeDispatcher {
    settings: HttpSourceChangeDispatcherSettings,
    client: Client,
}

impl HttpSourceChangeDispatcher {
    pub fn new(
        definition: &HttpSourceChangeDispatcherDefinition,
        storage: TestRunSourceStorage,
    ) -> Result<Self> {
        let source_id = storage.id.test_source_id.clone();
        let settings = HttpSourceChangeDispatcherSettings::new(definition, source_id);

        let client = Client::builder()
            .timeout(Duration::from_secs(settings.timeout_seconds))
            .build()?;

        eprintln!(
            "HTTP dispatcher created - URL: {}, Batch: {}",
            settings.full_url(),
            settings.batch_events
        );
        debug!(
            "Created HTTP source change dispatcher targeting {}",
            settings.full_url()
        );

        Ok(Self { settings, client })
    }
}

#[async_trait]
impl SourceChangeDispatcher for HttpSourceChangeDispatcher {
    async fn close(&mut self) -> Result<()> {
        debug!("Closing HTTP source change dispatcher");
        Ok(())
    }

    async fn dispatch_source_change_events(
        &mut self,
        events: Vec<&SourceChangeEvent>,
    ) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        eprintln!("HTTP dispatcher: Dispatching {} events to {}", events.len(), self.settings.full_url());
        trace!("Dispatching {} events to HTTP endpoint", events.len());

        let url = self.settings.full_url();

        // Transform events to the required format
        let transformed_events: Result<Vec<TransformedEvent>> = events
            .iter()
            .map(|event| TransformedEvent::from_source_change_event(event))
            .collect();
        let transformed_events = transformed_events?;

        if self.settings.batch_events {
            // Log request body at debug level
            debug!(
                "HTTP dispatcher sending batch request to {}: {}",
                url,
                serde_json::to_string_pretty(&transformed_events).unwrap_or_else(|e| format!("Failed to serialize: {}", e))
            );

            let response = match self
                .client
                .post(&url)
                .json(&transformed_events)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    eprintln!("HTTP dispatcher ERROR: Failed to connect to {}: {}", url, e);
                    error!("Failed to connect to {}: {}", url, e);
                    return Err(e.into());
                }
            };

            let status = response.status();
            let response_body = response.text().await.unwrap_or_default();
            
            // Log response at debug level
            debug!(
                "HTTP dispatcher received response from {}: Status: {}, Body: {}",
                url, status, response_body
            );
            
            if !status.is_success() {
                eprintln!(
                    "HTTP dispatcher ERROR: Failed to dispatch events batch to {}: {} - {}",
                    url, status, response_body
                );
                error!(
                    "Failed to dispatch events batch to {}: {} - {}",
                    url, status, response_body
                );
                anyhow::bail!("HTTP request failed with status: {}", status);
            }

            eprintln!(
                "HTTP dispatcher SUCCESS: Dispatched batch of {} events to {}",
                events.len(),
                url
            );
            debug!(
                "Successfully dispatched batch of {} events to {}",
                events.len(),
                url
            );
        } else {
            let event_count = events.len();
            for transformed_event in &transformed_events {
                // Log request body at debug level
                debug!(
                    "HTTP dispatcher sending individual event to {}: {}",
                    url,
                    serde_json::to_string_pretty(transformed_event).unwrap_or_else(|e| format!("Failed to serialize: {}", e))
                );

                let response = self
                    .client
                    .post(&url)
                    .json(transformed_event)
                    .send()
                    .await?;

                let status = response.status();
                let response_body = response.text().await.unwrap_or_default();
                
                // Log response at debug level
                debug!(
                    "HTTP dispatcher received response from {}: Status: {}, Body: {}",
                    url, status, response_body
                );
                
                if !status.is_success() {
                    error!(
                        "Failed to dispatch event to {}: {} - {}",
                        url, status, response_body
                    );
                    anyhow::bail!("HTTP request failed with status: {}", status);
                }
            }

            eprintln!(
                "HTTP dispatcher SUCCESS: Dispatched {} individual events to {}",
                event_count, url
            );
            debug!(
                "Successfully dispatched {} individual events to {}",
                event_count, url
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use test_data_store::scripts::{SourceChangeEventPayload, SourceChangeEventSourceInfo};

    fn create_test_event(op: &str, before: Value, after: Value) -> SourceChangeEvent {
        SourceChangeEvent {
            op: op.to_string(),
            reactivator_start_ns: 1000,
            reactivator_end_ns: 2000,
            payload: SourceChangeEventPayload {
                source: SourceChangeEventSourceInfo {
                    db: "test_db".to_string(),
                    table: "test_table".to_string(),
                    ts_ns: 3000,
                    lsn: 1,
                },
                before,
                after,
            },
        }
    }

    #[test]
    fn test_transform_insert_event() {
        let after_data = json!({
            "type": "node",
            "id": "user_123",
            "labels": ["User"],
            "properties": {
                "name": "John Doe",
                "email": "john@example.com"
            }
        });
        
        let event = create_test_event("i", json!(null), after_data.clone());
        let transformed = TransformedEvent::from_source_change_event(&event).unwrap();
        
        assert_eq!(transformed.op, "insert");
        assert_eq!(transformed.data, Some(after_data));
        assert_eq!(transformed.node_type, None);
        assert_eq!(transformed.before, None);
        assert_eq!(transformed.after, None);
    }

    #[test]
    fn test_transform_update_event() {
        let before_data = json!({
            "id": "user_123",
            "type": "node",
            "labels": ["User"],
            "properties": {
                "name": "John Doe",
                "email": "john@example.com"
            }
        });
        
        let after_data = json!({
            "id": "user_123",
            "type": "node",
            "labels": ["User"],
            "properties": {
                "name": "John Smith",
                "email": "john.smith@example.com"
            }
        });
        
        let event = create_test_event("u", before_data.clone(), after_data.clone());
        let transformed = TransformedEvent::from_source_change_event(&event).unwrap();
        
        assert_eq!(transformed.op, "update");
        assert_eq!(transformed.data, None);
        assert_eq!(transformed.node_type, Some("node".to_string()));
        assert_eq!(transformed.before, Some(before_data));
        assert_eq!(transformed.after, Some(after_data));
    }

    #[test]
    fn test_transform_delete_event() {
        let before_data = json!({
            "type": "node",
            "id": "user_123",
            "labels": ["User"],
            "properties": {}
        });
        
        let event = create_test_event("d", before_data.clone(), json!(null));
        let transformed = TransformedEvent::from_source_change_event(&event).unwrap();
        
        assert_eq!(transformed.op, "delete");
        assert_eq!(transformed.data, Some(before_data));
        assert_eq!(transformed.node_type, None);
        assert_eq!(transformed.before, None);
        assert_eq!(transformed.after, None);
    }

    #[test]
    fn test_transform_unknown_operation() {
        let event = create_test_event("x", json!(null), json!(null));
        let result = TransformedEvent::from_source_change_event(&event);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown operation type: x"));
    }

    #[test]
    fn test_settings_with_defaults() {
        let definition = HttpSourceChangeDispatcherDefinition {
            url: "http://localhost".to_string(),
            port: 8080,
            endpoint: None,
            timeout_seconds: None,
            batch_events: None,
        };

        let source_id = "test-source".to_string();
        let settings = HttpSourceChangeDispatcherSettings::new(&definition, source_id);

        assert_eq!(settings.url, "http://localhost");
        assert_eq!(settings.port, 8080);
        assert_eq!(settings.endpoint, "/sources/test-source/events");
        assert_eq!(settings.timeout_seconds, 30);
        assert!(settings.batch_events);
        assert_eq!(settings.full_url(), "http://localhost:8080/sources/test-source/events");
    }

    #[test]
    fn test_settings_with_custom_values() {
        let definition = HttpSourceChangeDispatcherDefinition {
            url: "https://api.example.com".to_string(),
            port: 443,
            endpoint: Some("/webhooks/changes".to_string()),
            timeout_seconds: Some(60),
            batch_events: Some(false),
        };

        let source_id = "test-source".to_string();
        let settings = HttpSourceChangeDispatcherSettings::new(&definition, source_id);

        assert_eq!(settings.url, "https://api.example.com");
        assert_eq!(settings.port, 443);
        assert_eq!(settings.endpoint, "/webhooks/changes");
        assert_eq!(settings.timeout_seconds, 60);
        assert!(!settings.batch_events);
        assert_eq!(
            settings.full_url(),
            "https://api.example.com:443/webhooks/changes"
        );
    }
}
