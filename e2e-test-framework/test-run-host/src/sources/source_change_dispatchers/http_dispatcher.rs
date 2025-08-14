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

use test_data_store::{
    scripts::SourceChangeEvent, test_repo_storage::models::HttpSourceChangeDispatcherDefinition,
    test_run_storage::TestRunSourceStorage,
};

use super::SourceChangeDispatcher;

use reqwest::Client;
use std::time::Duration;

use tracing::{debug, error, trace};

#[derive(Debug)]
pub struct HttpSourceChangeDispatcherSettings {
    pub url: String,
    pub port: u16,
    pub endpoint: String,
    pub timeout_seconds: u64,
    pub batch_events: bool,
    pub source_id: String,
}

impl HttpSourceChangeDispatcherSettings {
    pub fn new(
        definition: &HttpSourceChangeDispatcherDefinition,
        source_id: String,
    ) -> anyhow::Result<Self> {
        // If endpoint is provided, use it as-is, otherwise construct from source_id
        let endpoint = if let Some(ep) = &definition.endpoint {
            ep.clone()
        } else {
            format!("/sources/{}/events", source_id)
        };

        Ok(Self {
            url: definition.url.clone(),
            port: definition.port,
            endpoint,
            timeout_seconds: definition.timeout_seconds.unwrap_or(30),
            batch_events: definition.batch_events.unwrap_or(true),
            source_id,
        })
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
    ) -> anyhow::Result<Self> {
        log::debug!(
            "Creating HttpSourceChangeDispatcher from {:?}, ",
            definition
        );

        let source_id = storage.id.test_source_id.clone();
        let settings = HttpSourceChangeDispatcherSettings::new(definition, source_id)?;
        trace!(
            "Creating HttpSourceChangeDispatcher with settings {:?}, ",
            settings
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(settings.timeout_seconds))
            .build()?;

        Ok(Self { settings, client })
    }
}

#[async_trait]
impl SourceChangeDispatcher for HttpSourceChangeDispatcher {
    async fn close(&mut self) -> anyhow::Result<()> {
        debug!("Closing HTTP source change dispatcher");
        Ok(())
    }

    async fn dispatch_source_change_events(
        &mut self,
        events: Vec<&SourceChangeEvent>,
    ) -> anyhow::Result<()> {
        trace!("Dispatching {} events to HTTP endpoint", events.len());

        if events.is_empty() {
            return Ok(());
        }

        let url = self.settings.full_url();
        
        log::info!(
            "HTTP dispatcher sending {} events to {} (source_id: {}, batch: {})",
            events.len(),
            url,
            self.settings.source_id,
            self.settings.batch_events
        );

        if self.settings.batch_events {
            // Log request body at debug level
            debug!(
                "HTTP dispatcher sending batch request to {}: {}",
                url,
                serde_json::to_string_pretty(&events)
                    .unwrap_or_else(|e| format!("Failed to serialize: {}", e))
            );

            let response = match self.client.post(&url).json(&events).send().await {
                Ok(resp) => resp,
                Err(e) => {
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
                error!(
                    "Failed to dispatch events batch to {}: {} - {}",
                    url, status, response_body
                );
                anyhow::bail!("HTTP request failed with status: {}", status);
            }

            log::info!(
                "Successfully dispatched batch of {} events to {} - Status: {}",
                events.len(),
                url,
                status
            );
        } else {
            let event_count = events.len();
            for event in &events {
                // Log request body at debug level
                debug!(
                    "HTTP dispatcher sending individual event to {}: {}",
                    url,
                    serde_json::to_string_pretty(event)
                        .unwrap_or_else(|e| format!("Failed to serialize: {}", e))
                );

                let response = self.client.post(&url).json(event).send().await?;

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

            trace!(
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
        let settings = HttpSourceChangeDispatcherSettings::new(&definition, source_id).unwrap();

        assert_eq!(settings.url, "http://localhost");
        assert_eq!(settings.port, 8080);
        assert_eq!(settings.endpoint, "/sources/test-source/events");
        assert_eq!(settings.timeout_seconds, 30);
        assert!(settings.batch_events);
        assert_eq!(
            settings.full_url(),
            "http://localhost:8080/sources/test-source/events"
        );
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
        let settings = HttpSourceChangeDispatcherSettings::new(&definition, source_id).unwrap();

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
