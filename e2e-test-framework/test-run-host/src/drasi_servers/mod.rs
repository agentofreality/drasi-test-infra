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

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use derive_more::Debug;
use drasi_server::{ApplicationHandle, RuntimeConfig, server_core::DrasiServerCore};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use utoipa::ToSchema;

use test_data_store::{
    test_repo_storage::models::{
        DrasiServerConfig as TestDrasiServerConfig, TestDrasiServerDefinition,
    },
    test_run_storage::{
        ParseTestRunIdError, TestRunDrasiServerId, TestRunDrasiServerStorage, TestRunId,
    },
};

pub mod api_models;
pub mod programmatic_api;

#[cfg(test)]
mod tests;

/// Runtime configuration for a test run Drasi Server
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct TestRunDrasiServerConfig {
    #[serde(default = "default_start_immediately")]
    pub start_immediately: bool,
    pub test_id: String,
    pub test_repo_id: String,
    pub test_run_id: Option<String>,
    pub test_drasi_server_id: String,
    pub test_run_overrides: Option<TestRunDrasiServerOverrides>,
}

fn default_start_immediately() -> bool {
    true
}

/// Overrides for Drasi Server configuration at runtime
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct TestRunDrasiServerOverrides {
    /// Override authentication settings
    pub auth: Option<test_data_store::test_repo_storage::models::DrasiServerAuthConfig>,

    /// Override storage settings
    pub storage: Option<test_data_store::test_repo_storage::models::DrasiServerStorageConfig>,

    /// Override log level (trace, debug, info, warn, error)
    pub log_level: Option<String>,
}

impl TryFrom<&TestRunDrasiServerConfig> for TestRunId {
    type Error = ParseTestRunIdError;

    fn try_from(value: &TestRunDrasiServerConfig) -> Result<Self, Self::Error> {
        Ok(TestRunId::new(
            &value.test_repo_id,
            &value.test_id,
            value
                .test_run_id
                .as_deref()
                .unwrap_or(&chrono::Utc::now().format("%Y%m%d%H%M%S").to_string()),
        ))
    }
}

impl TryFrom<&TestRunDrasiServerConfig> for TestRunDrasiServerId {
    type Error = test_data_store::test_run_storage::ParseTestRunDrasiServerIdError;

    fn try_from(value: &TestRunDrasiServerConfig) -> Result<Self, Self::Error> {
        match TestRunId::try_from(value) {
            Ok(test_run_id) => Ok(TestRunDrasiServerId::new(
                &test_run_id,
                &value.test_drasi_server_id,
            )),
            Err(e) => Err(
                test_data_store::test_run_storage::ParseTestRunDrasiServerIdError::InvalidValues(
                    e.to_string(),
                ),
            ),
        }
    }
}

impl fmt::Display for TestRunDrasiServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TestRunDrasiServerConfig: Repo: test_repo_id: {:?}, test_id: {:?}, test_run_id: {:?}, test_drasi_server_id: {:?}", 
            self.test_repo_id, self.test_id, self.test_run_id, self.test_drasi_server_id)
    }
}

/// Combined test and runtime configuration for a Drasi Server
#[derive(Clone, Debug)]
pub struct TestRunDrasiServerDefinition {
    pub id: TestRunDrasiServerId,
    pub start_immediately: bool,
    pub test_drasi_server_definition: TestDrasiServerDefinition,
    pub test_run_overrides: Option<TestRunDrasiServerOverrides>,
}

impl TestRunDrasiServerDefinition {
    pub fn new(
        config: TestRunDrasiServerConfig,
        test_drasi_server_definition: TestDrasiServerDefinition,
    ) -> anyhow::Result<Self> {
        let id = TestRunDrasiServerId::try_from(&config)?;

        Ok(Self {
            id,
            start_immediately: config.start_immediately,
            test_drasi_server_definition,
            test_run_overrides: config.test_run_overrides,
        })
    }

    /// Get the effective configuration with overrides applied
    pub fn effective_config(&self) -> TestDrasiServerConfig {
        let mut config = self.test_drasi_server_definition.config.clone();

        if let Some(overrides) = &self.test_run_overrides {
            if let Some(auth) = &overrides.auth {
                config.auth = Some(auth.clone());
            }
            if let Some(storage) = &overrides.storage {
                config.storage = Some(storage.clone());
            }
            if let Some(log_level) = &overrides.log_level {
                config.log_level = Some(log_level.clone());
            }
        }

        config
    }
}

/// State of a test run Drasi Server
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum TestRunDrasiServerState {
    Uninitialized,
    Running {
        start_time: chrono::DateTime<chrono::Utc>,
    },
    Stopped {
        stop_time: chrono::DateTime<chrono::Utc>,
        reason: Option<String>,
    },
    Error {
        error_time: chrono::DateTime<chrono::Utc>,
        message: String,
    },
}

impl fmt::Display for TestRunDrasiServerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TestRunDrasiServerState::Uninitialized => write!(f, "Uninitialized"),
            TestRunDrasiServerState::Running { start_time } => {
                write!(f, "Running since {}", start_time)
            }
            TestRunDrasiServerState::Stopped { stop_time, reason } => {
                if let Some(reason) = reason {
                    write!(f, "Stopped at {} ({})", stop_time, reason)
                } else {
                    write!(f, "Stopped at {}", stop_time)
                }
            }
            TestRunDrasiServerState::Error {
                error_time,
                message,
            } => {
                write!(f, "Error at {}: {}", error_time, message)
            }
        }
    }
}


/// Test run Drasi Server component
#[derive(Debug)]
pub struct TestRunDrasiServer {
    pub definition: TestRunDrasiServerDefinition,
    pub state: Arc<RwLock<TestRunDrasiServerState>>,
    pub storage: TestRunDrasiServerStorage,
    #[debug(skip)]
    drasi_core: Arc<RwLock<Option<Arc<DrasiServerCore>>>>,
    #[debug(skip)]
    application_handles: Arc<RwLock<HashMap<String, ApplicationHandle>>>,
}

impl TestRunDrasiServer {
    pub async fn new(
        definition: TestRunDrasiServerDefinition,
        storage: TestRunDrasiServerStorage,
    ) -> anyhow::Result<Self> {
        let server = Self {
            definition,
            state: Arc::new(RwLock::new(TestRunDrasiServerState::Uninitialized)),
            storage,
            drasi_core: Arc::new(RwLock::new(None)),
            application_handles: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start immediately if configured
        if server.definition.start_immediately {
            log::info!(
                "Auto-starting Drasi Server {} with start_immediately=true",
                server.definition.id
            );
            match server.start().await {
                Ok(()) => {
                    let endpoint = server.get_api_endpoint().await;
                    log::info!(
                        "Drasi Server {} auto-started successfully at {}",
                        server.definition.id,
                        endpoint.unwrap_or_else(|| "unknown endpoint".to_string())
                    );
                }
                Err(e) => {
                    log::error!(
                        "Failed to auto-start Drasi Server {}: {}",
                        server.definition.id,
                        e
                    );
                    return Err(e);
                }
            }
        } else {
            log::info!(
                "Drasi Server {} created with start_immediately=false, manual start required",
                server.definition.id
            );
        }

        Ok(server)
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        let mut state = self.state.write().await;

        match &*state {
            TestRunDrasiServerState::Uninitialized => {
                // Get effective configuration
                let config = self.definition.effective_config();

                // Determine log level (default to "info" if not specified)
                let log_level = config.log_level.as_deref().unwrap_or("info");
                
                // Convert our configs to drasi_server configs
                let drasi_sources: Vec<drasi_server::config::SourceConfig> = config.sources.iter().map(|s| {
                    drasi_server::config::SourceConfig {
                        name: s.name.clone(),
                        source_type: s.source_type.clone(),
                        auto_start: s.auto_start,
                        properties: s.properties.clone(),
                    }
                }).collect();
                
                let drasi_queries: Vec<drasi_server::config::QueryConfig> = config.queries.iter().map(|q| {
                    drasi_server::config::QueryConfig {
                        name: q.name.clone(),
                        query: q.query.clone(),
                        sources: q.sources.clone(),
                        auto_start: q.auto_start,
                        properties: q.properties.clone(),
                    }
                }).collect();
                
                let drasi_reactions: Vec<drasi_server::config::ReactionConfig> = config.reactions.iter().map(|r| {
                    drasi_server::config::ReactionConfig {
                        name: r.name.clone(),
                        reaction_type: r.reaction_type.clone(),
                        queries: r.queries.clone(),
                        auto_start: r.auto_start,
                        properties: r.properties.clone(),
                    }
                }).collect();

                // Create RuntimeConfig for DrasiServerCore with all components
                let runtime_config = Arc::new(RuntimeConfig {
                    server: drasi_server::config::schema::ServerSettings {
                        host: "0.0.0.0".to_string(),
                        port: 0, // Not used by DrasiServerCore (embedded library)
                        log_level: log_level.to_string(),
                        max_connections: 1000,
                        shutdown_timeout_seconds: 30,
                    },
                    sources: drasi_sources,
                    queries: drasi_queries,
                    reactions: drasi_reactions,
                });
                
                // Create the DrasiServerCore instance
                let mut core = DrasiServerCore::new(runtime_config);
                
                // Log configuration summary
                log::info!(
                    "Created DrasiServerCore with {} sources, {} queries, {} reactions pre-configured",
                    config.sources.len(),
                    config.queries.len(),
                    config.reactions.len()
                );
                
                // Initialize the core to create all components
                log::info!("Initializing DrasiServerCore to create components...");
                core.initialize().await
                    .map_err(|e| anyhow::anyhow!("Failed to initialize DrasiServerCore: {}", e))?;
                
                // Store the core after initialization but before starting
                let core = Arc::new(core);
                
                // Start the core to start all auto-start components
                log::info!("Starting DrasiServerCore to start auto-start components...");
                core.start().await
                    .map_err(|e| anyhow::anyhow!("Failed to start DrasiServerCore: {}", e))?;
                
                // Store configured component names for validation
                let configured_source_names: std::collections::HashSet<String> =
                    config.sources.iter().map(|s| s.name.clone()).collect();
                let configured_query_names: std::collections::HashSet<String> =
                    config.queries.iter().map(|q| q.name.clone()).collect();
                let configured_reaction_names: std::collections::HashSet<String> =
                    config.reactions.iter().map(|r| r.name.clone()).collect();

                // Store the core reference
                {
                    let mut core_guard = self.drasi_core.write().await;
                    *core_guard = Some(core.clone());
                }
                
                log::info!("DrasiServerCore initialized with {} sources, {} queries, {} reactions configured",
                    config.sources.len(), config.queries.len(), config.reactions.len());
                
                // Log the status of components
                log::info!("DrasiServerCore ready, verifying component status...");
                
                // Verify query status
                for query_config in &config.queries {
                    match core.query_manager().get_query_status(query_config.name.clone()).await {
                        Ok(status) => {
                            log::info!("Query '{}' status after startup: {:?}", query_config.name, status);
                        }
                        Err(e) => {
                            log::error!("Failed to get status for query '{}': {}", query_config.name, e);
                        }
                    }
                }
                
                // Get and store application handles from the core managers
                {
                    let mut stored_handles = self.application_handles.write().await;
                    stored_handles.clear();
                    
                    // Get handles from source manager for configured sources
                    for source_config in &config.sources {
                        if let Some(handle) = core.source_manager().get_application_handle(&source_config.name).await {
                            stored_handles.insert(source_config.name.clone(), ApplicationHandle::source_only(handle));
                            log::info!(
                                "Stored ApplicationHandle for source '{}' on Drasi Server {}",
                                source_config.name,
                                self.definition.id
                            );
                        } else {
                            log::warn!(
                                "Could not get ApplicationHandle for source '{}' on Drasi Server {}",
                                source_config.name,
                                self.definition.id
                            );
                        }
                    }
                    
                    // Get handles from reaction manager for configured reactions  
                    for reaction_config in &config.reactions {
                        if let Some(handle) = core.reaction_manager().get_application_handle(&reaction_config.name).await {
                            stored_handles.insert(reaction_config.name.clone(), ApplicationHandle::reaction_only(handle));
                            log::info!(
                                "Stored ApplicationHandle for reaction '{}' on Drasi Server {}",
                                reaction_config.name,
                                self.definition.id
                            );
                        } else {
                            log::warn!(
                                "Could not get ApplicationHandle for reaction '{}' on Drasi Server {}",
                                reaction_config.name,
                                self.definition.id
                            );
                        }
                    }
                    
                    // Note: Query manager doesn't provide application handles
                    
                    log::info!(
                        "Stored {} application handles for Drasi Server {} after starting",
                        stored_handles.len(),
                        self.definition.id
                    );
                }

                // Log validation information
                if configured_source_names.is_empty()
                    && configured_query_names.is_empty()
                    && configured_reaction_names.is_empty()
                {
                    log::warn!(
                        "Drasi Server {} configured without any sources, queries, or reactions",
                        self.definition.id
                    );
                } else {
                    log::info!(
                        "Drasi Server {} configured with {} sources, {} queries, {} reactions",
                        self.definition.id,
                        configured_source_names.len(),
                        configured_query_names.len(),
                        configured_reaction_names.len()
                    );
                }

                // Update state
                *state = TestRunDrasiServerState::Running {
                    start_time: chrono::Utc::now(),
                };


                // Write server config to storage
                let config_json = serde_json::to_value(&config)?;
                self.storage.write_server_config(&config_json).await?;

                log::info!(
                    "DrasiServerCore {} started successfully",
                    self.definition.id
                );
                
                // Add a small delay to ensure all async initialization completes
                log::info!("Waiting 100ms for DrasiServerCore components to fully initialize...");
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                Ok(())
            }
            TestRunDrasiServerState::Running { .. } => {
                anyhow::bail!("Server is already running");
            }
            TestRunDrasiServerState::Stopped { .. } => {
                anyhow::bail!("Server has been stopped and cannot be restarted");
            }
            TestRunDrasiServerState::Error { .. } => {
                anyhow::bail!("Server is in error state");
            }
        }
    }

    pub async fn stop(&self, reason: Option<String>) -> anyhow::Result<()> {
        let mut state = self.state.write().await;

        match &*state {
            TestRunDrasiServerState::Running { .. } => {
                // Clear the core reference
                // Note: DrasiServerCore doesn't need explicit shutdown
                {
                    let mut core_guard = self.drasi_core.write().await;
                    *core_guard = None;
                }

                // Clear application handles
                {
                    let mut handles = self.application_handles.write().await;
                    handles.clear();
                }

                // Update state
                *state = TestRunDrasiServerState::Stopped {
                    stop_time: chrono::Utc::now(),
                    reason,
                };

                log::info!("Drasi Server {} stopped", self.definition.id);
                Ok(())
            }
            _ => {
                anyhow::bail!("Server is not running");
            }
        }
    }

    pub async fn get_state(&self) -> TestRunDrasiServerState {
        self.state.read().await.clone()
    }

    pub async fn get_server_core(&self) -> Option<Arc<drasi_server::server_core::DrasiServerCore>> {
        let core_guard = self.drasi_core.read().await;
        core_guard.clone()
    }

    pub async fn get_server_port(&self) -> Option<u16> {
        // DrasiServerCore doesn't use ports - it's an embedded library, not a network server
        None
    }

    /// Returns the API endpoint for this Drasi Server.
    /// 
    /// **Note**: This always returns `None` because DrasiServerCore is an embedded library
    /// that provides programmatic access to Drasi functionality, not a standalone server
    /// with HTTP endpoints. The test infrastructure wraps DrasiServerCore with its own
    /// REST API (test-service) for external access.
    pub async fn get_api_endpoint(&self) -> Option<String> {
        None
    }

    pub async fn get_application_handle(&self, name: &str) -> Option<ApplicationHandle> {
        self.application_handles.read().await.get(name).cloned()
    }

    pub(crate) async fn with_core<F, Fut, T>(&self, f: F) -> anyhow::Result<T>
    where
        F: FnOnce(Arc<drasi_server::server_core::DrasiServerCore>) -> Fut,
        Fut: std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
        T: Send + 'static,
    {
        let core_guard = self.drasi_core.read().await;
        match core_guard.as_ref() {
            Some(core) => {
                f(core.clone()).await
            }
            None => Err(anyhow::anyhow!("DrasiServerCore not available - server not running")),
        }
    }

    pub async fn write_summary(&self) -> anyhow::Result<()> {
        let summary = serde_json::json!({
            "id": self.definition.id.to_string(),
            "name": self.definition.test_drasi_server_definition.name,
            "state": self.get_state().await,
            "config": self.definition.effective_config(),
        });

        self.storage.write_test_run_summary(&summary).await?;
        Ok(())
    }
}

impl Drop for TestRunDrasiServer {
    fn drop(&mut self) {
        // Schedule cleanup of the server if it's still running
        let state = self.state.clone();
        let drasi_core = self.drasi_core.clone();
        let id = self.definition.id.clone();

        tokio::spawn(async move {
            let current_state = state.read().await;
            if matches!(*current_state, TestRunDrasiServerState::Running { .. }) {
                log::warn!(
                    "Drasi Server {} is being dropped while still running, clearing core reference",
                    id
                );

                // Clear the core reference
                let mut core_guard = drasi_core.write().await;
                *core_guard = None;
            }
        });
    }
}
