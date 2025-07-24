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

use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::SystemTime};

use async_trait::async_trait;
use axum::{
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::IntoResponse,
    routing::any,
    Router, Server,
};
use test_data_store::{
    test_repo_storage::models::HttpReactionHandlerDefinition, test_run_storage::TestRunQueryId,
};
use tokio::sync::{
    mpsc::{Receiver, Sender},
    Notify, RwLock,
};

use crate::queries::{
    output_handler_message::{
        HandlerError, HandlerPayload, HandlerRecord, HandlerType, OutputHandlerMessage,
    },
    unified_handler::{OutputHandler, UnifiedHandlerStatus},
};

#[derive(Clone, Debug)]
pub struct HttpReactionHandlerSettings {
    pub host: String,
    pub port: u16,
    pub path: String,
    pub correlation_header: Option<String>,
    pub test_run_query_id: TestRunQueryId,
}

impl HttpReactionHandlerSettings {
    pub fn new(
        id: TestRunQueryId,
        definition: HttpReactionHandlerDefinition,
    ) -> anyhow::Result<Self> {
        Ok(HttpReactionHandlerSettings {
            host: definition
                .host
                .clone()
                .unwrap_or_else(|| "0.0.0.0".to_string()),
            port: definition.port.unwrap_or(8081),
            path: definition
                .path
                .clone()
                .unwrap_or_else(|| "/reaction".to_string()),
            correlation_header: definition.correlation_header,
            test_run_query_id: id,
        })
    }
}

#[derive(Clone)]
struct HttpServerState {
    tx: Sender<OutputHandlerMessage>,
    settings: HttpReactionHandlerSettings,
}

pub struct HttpReactionHandler {
    notifier: Arc<Notify>,
    settings: HttpReactionHandlerSettings,
    status: Arc<RwLock<UnifiedHandlerStatus>>,
    shutdown_notify: Arc<Notify>,
}

impl HttpReactionHandler {
    #[allow(clippy::new_ret_no_self)]
    pub async fn new(
        id: TestRunQueryId,
        definition: HttpReactionHandlerDefinition,
    ) -> anyhow::Result<Box<dyn OutputHandler + Send + Sync>> {
        let settings = HttpReactionHandlerSettings::new(id, definition)?;
        log::trace!("Creating HttpReactionHandler with settings {:?}", settings);

        let notifier = Arc::new(Notify::new());
        let status = Arc::new(RwLock::new(UnifiedHandlerStatus::Uninitialized));
        let shutdown_notify = Arc::new(Notify::new());

        Ok(Box::new(Self {
            notifier,
            settings,
            status,
            shutdown_notify,
        }))
    }
}

#[async_trait]
impl OutputHandler for HttpReactionHandler {
    async fn init(&self) -> anyhow::Result<Receiver<OutputHandlerMessage>> {
        log::debug!("Initializing HttpReactionHandler");

        if let Ok(mut status) = self.status.try_write() {
            match *status {
                UnifiedHandlerStatus::Uninitialized => {
                    let (handler_tx_channel, handler_rx_channel) = tokio::sync::mpsc::channel(100);

                    *status = UnifiedHandlerStatus::Paused;

                    tokio::spawn(http_server_thread(
                        self.settings.clone(),
                        self.status.clone(),
                        self.notifier.clone(),
                        self.shutdown_notify.clone(),
                        handler_tx_channel,
                    ));

                    Ok(handler_rx_channel)
                }
                UnifiedHandlerStatus::Running => {
                    anyhow::bail!("Can't Init Handler, Handler currently Running");
                }
                UnifiedHandlerStatus::Paused => {
                    anyhow::bail!("Can't Init Handler, Handler currently Paused");
                }
                UnifiedHandlerStatus::Stopped => {
                    anyhow::bail!("Can't Init Handler, Handler currently Stopped");
                }
                UnifiedHandlerStatus::Error => {
                    anyhow::bail!("Handler in Error state");
                }
                UnifiedHandlerStatus::BootstrapStarted
                | UnifiedHandlerStatus::BootstrapComplete
                | UnifiedHandlerStatus::Deleted => {
                    anyhow::bail!("Handler in invalid state for init: {:?}", *status);
                }
            }
        } else {
            anyhow::bail!("Could not acquire status lock");
        }
    }

    async fn start(&self) -> anyhow::Result<()> {
        log::debug!("Starting HttpReactionHandler");

        if let Ok(mut status) = self.status.try_write() {
            match *status {
                UnifiedHandlerStatus::Uninitialized => {
                    anyhow::bail!("Can't Start Handler, Handler Uninitialized");
                }
                UnifiedHandlerStatus::Running => Ok(()),
                UnifiedHandlerStatus::Paused => {
                    *status = UnifiedHandlerStatus::Running;
                    self.notifier.notify_one();
                    Ok(())
                }
                UnifiedHandlerStatus::Stopped => {
                    anyhow::bail!("Can't Start Handler, Handler already Stopped");
                }
                UnifiedHandlerStatus::Error => {
                    anyhow::bail!("Handler in Error state");
                }
                UnifiedHandlerStatus::BootstrapStarted
                | UnifiedHandlerStatus::BootstrapComplete
                | UnifiedHandlerStatus::Deleted => {
                    anyhow::bail!("Handler in invalid state for start: {:?}", *status);
                }
            }
        } else {
            anyhow::bail!("Could not acquire status lock");
        }
    }

    async fn pause(&self) -> anyhow::Result<()> {
        log::debug!("Pausing HttpReactionHandler");

        if let Ok(mut status) = self.status.try_write() {
            match *status {
                UnifiedHandlerStatus::Uninitialized => {
                    anyhow::bail!("Can't Pause Handler, Handler Uninitialized");
                }
                UnifiedHandlerStatus::Running => {
                    *status = UnifiedHandlerStatus::Paused;
                    Ok(())
                }
                UnifiedHandlerStatus::Paused => Ok(()),
                UnifiedHandlerStatus::Stopped => {
                    anyhow::bail!("Can't Pause Handler, Handler already Stopped");
                }
                UnifiedHandlerStatus::Error => {
                    anyhow::bail!("Handler in Error state");
                }
                UnifiedHandlerStatus::BootstrapStarted
                | UnifiedHandlerStatus::BootstrapComplete
                | UnifiedHandlerStatus::Deleted => {
                    anyhow::bail!("Handler in invalid state for pause: {:?}", *status);
                }
            }
        } else {
            anyhow::bail!("Could not acquire status lock");
        }
    }

    async fn stop(&self) -> anyhow::Result<()> {
        log::debug!("Stopping HttpReactionHandler");

        if let Ok(mut status) = self.status.try_write() {
            match *status {
                UnifiedHandlerStatus::Uninitialized => {
                    anyhow::bail!("Handler not initialized, current status: Uninitialized");
                }
                UnifiedHandlerStatus::Running | UnifiedHandlerStatus::Paused => {
                    *status = UnifiedHandlerStatus::Stopped;
                    self.shutdown_notify.notify_one();
                    Ok(())
                }
                UnifiedHandlerStatus::Stopped => Ok(()),
                UnifiedHandlerStatus::Error => {
                    anyhow::bail!("Handler in Error state");
                }
                UnifiedHandlerStatus::BootstrapStarted
                | UnifiedHandlerStatus::BootstrapComplete
                | UnifiedHandlerStatus::Deleted => {
                    anyhow::bail!("Handler in invalid state for stop: {:?}", *status);
                }
            }
        } else {
            anyhow::bail!("Could not acquire status lock");
        }
    }

    async fn status(&self) -> UnifiedHandlerStatus {
        *self.status.read().await
    }
}

async fn http_server_thread(
    settings: HttpReactionHandlerSettings,
    status: Arc<RwLock<UnifiedHandlerStatus>>,
    notify: Arc<Notify>,
    shutdown_notify: Arc<Notify>,
    result_handler_tx_channel: Sender<OutputHandlerMessage>,
) {
    log::debug!("Starting HttpReactionHandler Server Thread");

    // Wait for the handler to be started
    loop {
        let current_status = {
            if let Ok(status) = status.try_read() {
                *status
            } else {
                log::warn!("Could not acquire status lock while waiting to start");
                continue;
            }
        };

        match current_status {
            UnifiedHandlerStatus::Running => break,
            UnifiedHandlerStatus::Paused => {
                log::debug!("HTTP server waiting to be started");
                notify.notified().await;
            }
            UnifiedHandlerStatus::Stopped => {
                log::debug!("Handler stopped before server could start");
                return;
            }
            _ => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }

    let state = HttpServerState {
        tx: result_handler_tx_channel.clone(),
        settings: settings.clone(),
    };

    let app = Router::new()
        .route(&settings.path, any(handle_reaction))
        .route(&format!("{}/*path", &settings.path), any(handle_reaction))
        .with_state(state);

    let addr = match format!("{}:{}", settings.host, settings.port).parse::<SocketAddr>() {
        Ok(addr) => addr,
        Err(e) => {
            log::error!("Failed to parse server address: {}", e);
            *status.write().await = UnifiedHandlerStatus::Error;
            let _ = result_handler_tx_channel
                .send(OutputHandlerMessage::Error {
                    handler_type: HandlerType::Reaction,
                    error: HandlerError::HttpServerError(format!("Invalid address: {}", e)),
                })
                .await;
            return;
        }
    };

    log::info!("HTTP Reaction Handler listening on http://{}", addr);

    let server = Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(async move {
            shutdown_notify.notified().await;
            log::debug!("HTTP server received shutdown signal");
        });

    if let Err(e) = server.await {
        log::error!("HTTP server error: {}", e);
        *status.write().await = UnifiedHandlerStatus::Error;
        let _ = result_handler_tx_channel
            .send(OutputHandlerMessage::Error {
                handler_type: HandlerType::Reaction,
                error: HandlerError::HttpServerError(format!("Server error: {}", e)),
            })
            .await;
    }

    log::debug!("HTTP server thread shutting down, sending HandlerStopping message");
    let _ = result_handler_tx_channel
        .send(OutputHandlerMessage::HandlerStopping {
            handler_type: HandlerType::Reaction,
        })
        .await;
}

async fn handle_reaction(
    State(state): State<HttpServerState>,
    method: Method,
    headers: HeaderMap,
    uri: axum::http::Uri,
    body: String,
) -> impl IntoResponse {
    let invocation_time_ns = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    // Parse request body as JSON
    let request_body: serde_json::Value = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(_) => serde_json::json!({ "raw": body }),
    };

    // Extract headers
    let mut header_map = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value_str) = value.to_str() {
            header_map.insert(name.as_str().to_string(), value_str.to_string());
        }
    }

    // Extract trace context
    let traceparent = header_map.get("traceparent").cloned();
    let tracestate = header_map.get("tracestate").cloned();

    // Extract sequence from correlation header or request body
    let sequence = if let Some(correlation_header) = &state.settings.correlation_header {
        header_map
            .get(correlation_header)
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0)
    } else {
        request_body
            .get("sequence")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    };

    // Determine reaction type from path or request body
    let reaction_type = if uri.path().contains("/added") {
        "added".to_string()
    } else if uri.path().contains("/updated") {
        "updated".to_string()
    } else if uri.path().contains("/deleted") {
        "deleted".to_string()
    } else {
        request_body
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string()
    };

    let query_id = state.settings.test_run_query_id.test_query_id.clone();
    let record = HandlerRecord {
        id: format!("{}-{}", query_id, sequence),
        sequence,
        created_time_ns: invocation_time_ns,
        processed_time_ns: invocation_time_ns,
        traceparent,
        tracestate,
        payload: HandlerPayload::ReactionInvocation {
            reaction_type,
            query_id,
            request_method: method.to_string(),
            request_path: uri.path().to_string(),
            request_body,
            headers: header_map,
        },
    };

    log::debug!(
        "Received reaction invocation: {} {} (sequence: {})",
        method,
        uri.path(),
        sequence
    );

    match state.tx.send(OutputHandlerMessage::Record(record)).await {
        Ok(_) => (StatusCode::OK, "OK"),
        Err(e) => {
            log::error!("Failed to send reaction message: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
        }
    }
}
