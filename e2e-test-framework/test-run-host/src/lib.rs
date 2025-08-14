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

use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use derive_more::Debug;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use drasi_servers::{
    TestRunDrasiServer, TestRunDrasiServerConfig, TestRunDrasiServerDefinition,
    TestRunDrasiServerState,
};
use queries::{
    query_result_observer::QueryResultObserverCommandResponse,
    result_stream_loggers::ResultStreamLoggerResult, TestRunQuery, TestRunQueryConfig,
    TestRunQueryDefinition, TestRunQueryState,
};
use reactions::{
    reaction_observer::ReactionObserverCommandResponse, TestRunReaction, TestRunReactionConfig,
    TestRunReactionDefinition, TestRunReactionState,
};
use sources::{
    bootstrap_data_generators::BootstrapData, create_test_run_source,
    source_change_generators::SourceChangeGeneratorCommandResponse, SourceStartMode, TestRunSource,
    TestRunSourceConfig, TestRunSourceState,
};
use test_data_store::{
    test_repo_storage::models::SpacingMode,
    test_run_storage::{
        TestRunDrasiServerId, TestRunId, TestRunQueryId, TestRunReactionId, TestRunSourceId,
    },
    TestDataStore,
};

pub mod common;
pub mod drasi_server_api_impl;
pub mod drasi_servers;
pub mod grpc_converters;
pub mod queries;
pub mod reactions;
pub mod sources;
pub mod utils;

// Re-export api_models for use by test-service
pub use drasi_servers::api_models;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestRunConfig {
    pub test_id: String,
    pub test_repo_id: String,
    pub test_run_id: String,
    #[serde(default)]
    pub drasi_servers: Vec<TestRunDrasiServerConfig>,
    #[serde(default)]
    pub queries: Vec<TestRunQueryConfig>,
    #[serde(default)]
    pub reactions: Vec<TestRunReactionConfig>,
    #[serde(default)]
    pub sources: Vec<TestRunSourceConfig>,
}

#[derive(Debug)]
pub struct TestRun {
    pub id: TestRunId,
    pub drasi_servers: HashMap<String, TestRunDrasiServer>,
    pub queries: HashMap<String, TestRunQuery>,
    pub reactions: HashMap<String, TestRunReaction>,
    pub sources: HashMap<String, Box<dyn TestRunSource + Send + Sync>>,
    pub status: TestRunStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum TestRunStatus {
    Initialized,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TestRunHostConfig {
    #[serde(default)]
    pub test_runs: Vec<TestRunConfig>,
}

// An enum that represents the current state of the TestRunHost.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum TestRunHostStatus {
    // The TestRunHost is Initialized and is ready to start.
    Initialized,
    // The TestRunHost has been started.
    Running,
    // The TestRunHost is in an Error state. and will not be able to process requests.
    Error(String),
}

impl fmt::Display for TestRunHostStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestRunHostStatus::Initialized => write!(f, "Initialized"),
            TestRunHostStatus::Running => write!(f, "Running"),
            TestRunHostStatus::Error(msg) => write!(f, "Error: {}", msg),
        }
    }
}

#[derive(Debug)]
pub struct TestRunHost {
    data_store: Arc<TestDataStore>,
    test_runs: Arc<RwLock<HashMap<TestRunId, TestRun>>>,
    status: Arc<RwLock<TestRunHostStatus>>,
}

impl TestRunHost {
    pub async fn new(
        config: TestRunHostConfig,
        data_store: Arc<TestDataStore>,
    ) -> anyhow::Result<Self> {
        log::debug!("Creating TestRunHost from {:?}", config);

        let test_run_host = TestRunHost {
            data_store: data_store.clone(),
            test_runs: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(TestRunHostStatus::Initialized)),
        };

        // Add test runs from config
        for test_run_config in config.test_runs {
            test_run_host.add_test_run(test_run_config).await?;
        }

        log::debug!("TestRunHost created -  {:?}", &test_run_host);

        match &test_run_host.get_status().await? {
            TestRunHostStatus::Initialized => {
                log::info!("Starting TestRunHost...");
                test_run_host.set_status(TestRunHostStatus::Running).await;
            }
            TestRunHostStatus::Running => {
                let msg = "TestRunHost created with unexpected status: Running";
                log::error!("{}", msg);
                anyhow::bail!("{}", msg);
            }
            TestRunHostStatus::Error(_) => {
                let msg = "TestRunHost is in an Error state, cannot Start.".to_string();
                log::error!("{}", msg);
                anyhow::bail!("{}", msg);
            }
        };

        Ok(test_run_host)
    }

    pub async fn add_test_run(&self, config: TestRunConfig) -> anyhow::Result<TestRunId> {
        let test_run_id =
            TestRunId::new(&config.test_repo_id, &config.test_id, &config.test_run_id);

        let mut test_runs_lock = self.test_runs.write().await;
        if test_runs_lock.contains_key(&test_run_id) {
            anyhow::bail!("TestRun already exists with ID: {:?}", test_run_id);
        }

        let mut test_run = TestRun {
            id: test_run_id.clone(),
            drasi_servers: HashMap::new(),
            queries: HashMap::new(),
            reactions: HashMap::new(),
            sources: HashMap::new(),
            status: TestRunStatus::Initialized,
        };

        // Add drasi servers first (they need to be available for other components)
        for mut server_config in config.drasi_servers {
            server_config.test_id = Some(config.test_id.clone());
            server_config.test_repo_id = Some(config.test_repo_id.clone());
            server_config.test_run_id = Some(config.test_run_id.clone());
            self.add_drasi_server_to_test_run(&mut test_run, server_config)
                .await?;
        }

        // Add queries
        for mut query_config in config.queries {
            query_config.test_id = Some(config.test_id.clone());
            query_config.test_repo_id = Some(config.test_repo_id.clone());
            query_config.test_run_id = Some(config.test_run_id.clone());
            self.add_query_to_test_run(&mut test_run, query_config)
                .await?;
        }

        // Add reactions
        for mut reaction_config in config.reactions {
            reaction_config.test_id = Some(config.test_id.clone());
            reaction_config.test_repo_id = Some(config.test_repo_id.clone());
            reaction_config.test_run_id = Some(config.test_run_id.clone());
            self.add_reaction_to_test_run(&mut test_run, reaction_config)
                .await?;
        }

        // Add sources
        for mut source_config in config.sources {
            source_config.test_id = Some(config.test_id.clone());
            source_config.test_repo_id = Some(config.test_repo_id.clone());
            source_config.test_run_id = Some(config.test_run_id.clone());
            self.add_source_to_test_run(&mut test_run, source_config)
                .await?;
        }

        test_run.status = TestRunStatus::Running;
        test_runs_lock.insert(test_run_id.clone(), test_run);

        Ok(test_run_id)
    }

    pub async fn initialize_sources(&self, self_ref: Arc<Self>) -> anyhow::Result<()> {
        log::info!("Initializing sources with TestRunHost reference");

        let test_runs = self.test_runs.read().await;
        for (test_run_id, test_run) in test_runs.iter() {
            // Set TestRunHost on all sources
            for (source_id, source) in test_run.sources.iter() {
                log::debug!(
                    "Setting TestRunHost on source {} in test run {:?}",
                    source_id,
                    test_run_id
                );
                source.set_test_run_host(self_ref.clone());
            }

            // Set TestRunHost on all reactions (for handlers that need it)
            for (reaction_id, reaction) in test_run.reactions.iter() {
                log::debug!(
                    "Setting TestRunHost on reaction {} in test run {:?}",
                    reaction_id,
                    test_run_id
                );
                reaction.set_test_run_host(self_ref.clone());
            }

            // Start reactions with start_immediately BEFORE sources
            for (reaction_id, reaction) in test_run.reactions.iter() {
                if reaction.start_immediately {
                    log::info!(
                        "Auto-starting reaction {} in test run {:?} (before sources)",
                        reaction_id,
                        test_run_id
                    );
                    reaction.start_reaction_observer().await?;
                }
            }

            // Give reaction handlers time to fully initialize and start listening
            if test_run.reactions.values().any(|r| r.start_immediately) {
                log::info!("Waiting 2 seconds for reaction handlers to initialize...");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }

            // Start auto-start sources AFTER reactions are ready
            for (source_id, source) in test_run.sources.iter() {
                let state = source.get_state().await?;
                if state.start_mode == SourceStartMode::Auto {
                    log::info!(
                        "Auto-starting source {} in test run {:?} (after reactions are ready)",
                        source_id,
                        test_run_id
                    );
                    source.start_source_change_generator().await?;
                }
            }
        }

        Ok(())
    }

    async fn add_drasi_server_to_test_run(
        &self,
        test_run: &mut TestRun,
        test_run_drasi_server: TestRunDrasiServerConfig,
    ) -> anyhow::Result<()> {
        let test_drasi_server_id = test_run_drasi_server.test_drasi_server_id.clone();

        // Get the test definition and extract the drasi server definition
        let test_definition = self
            .data_store
            .get_test_definition(
                test_run_drasi_server.test_repo_id.as_ref().unwrap(),
                test_run_drasi_server.test_id.as_ref().unwrap(),
            )
            .await?;

        let test_drasi_server_definition = test_definition
            .drasi_servers
            .iter()
            .find(|s| s.id == test_drasi_server_id)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Drasi server definition not found: {}",
                    test_drasi_server_id
                )
            })?
            .clone();

        let definition =
            TestRunDrasiServerDefinition::new(test_run_drasi_server, test_drasi_server_definition)?;

        let id = TestRunDrasiServerId::new(&test_run.id, &test_drasi_server_id);
        let output_storage = self
            .data_store
            .get_test_run_drasi_server_storage(&id)
            .await?;

        let test_run_drasi_server = TestRunDrasiServer::new(definition, output_storage).await?;
        test_run
            .drasi_servers
            .insert(test_drasi_server_id, test_run_drasi_server);

        Ok(())
    }

    async fn add_query_to_test_run(
        &self,
        test_run: &mut TestRun,
        test_run_query: TestRunQueryConfig,
    ) -> anyhow::Result<()> {
        let test_query_id = test_run_query.test_query_id.clone();

        let repo = self
            .data_store
            .get_test_repo_storage(test_run_query.test_repo_id.as_ref().unwrap())
            .await?;
        repo.add_remote_test(test_run_query.test_id.as_ref().unwrap(), false)
            .await?;

        let id = TestRunQueryId::new(&test_run.id, &test_query_id);
        let test_query_definition = self
            .data_store
            .get_test_query_definition_for_test_run_query(&id)
            .await?;

        let definition = TestRunQueryDefinition::new(test_run_query, test_query_definition)?;
        let output_storage = self.data_store.get_test_run_query_storage(&id).await?;
        let test_run_query = TestRunQuery::new(definition, output_storage).await?;

        test_run.queries.insert(test_query_id, test_run_query);
        Ok(())
    }

    async fn add_reaction_to_test_run(
        &self,
        test_run: &mut TestRun,
        test_run_reaction: TestRunReactionConfig,
    ) -> anyhow::Result<()> {
        let test_reaction_id = test_run_reaction.test_reaction_id.clone();

        let repo = self
            .data_store
            .get_test_repo_storage(test_run_reaction.test_repo_id.as_ref().unwrap())
            .await?;
        repo.add_remote_test(test_run_reaction.test_id.as_ref().unwrap(), false)
            .await?;

        let test_definition = self
            .data_store
            .get_test_definition(
                test_run_reaction.test_repo_id.as_ref().unwrap(),
                test_run_reaction.test_id.as_ref().unwrap(),
            )
            .await?;

        let test_reaction_definition = test_definition.get_test_reaction(&test_reaction_id)?;

        let reaction_handler_definition = test_reaction_definition
            .output_handler
            .clone()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No reaction handler defined for reaction {}",
                    test_reaction_id
                )
            })?;

        let output_loggers = test_run_reaction.output_loggers.clone();
        let definition = TestRunReactionDefinition::new(
            test_run_reaction,
            test_reaction_definition.clone(),
            reaction_handler_definition,
            output_loggers,
        )?;

        let id = TestRunReactionId::new(&test_run.id, &test_reaction_id);
        let output_storage = self.data_store.get_test_run_reaction_storage(&id).await?;
        let test_run_reaction = TestRunReaction::new(definition, output_storage).await?;

        test_run
            .reactions
            .insert(test_reaction_id, test_run_reaction);
        Ok(())
    }

    async fn add_source_to_test_run(
        &self,
        test_run: &mut TestRun,
        test_run_config: TestRunSourceConfig,
    ) -> anyhow::Result<()> {
        let test_source_id = test_run_config.test_source_id.clone();

        let repo = self
            .data_store
            .get_test_repo_storage(test_run_config.test_repo_id.as_ref().unwrap())
            .await?;
        repo.add_remote_test(test_run_config.test_id.as_ref().unwrap(), false)
            .await?;

        let id = TestRunSourceId::new(&test_run.id, &test_source_id);
        let test_source_definition = self
            .data_store
            .get_test_source_definition_for_test_run_source(&id)
            .await?;

        let input_storage = self
            .data_store
            .get_test_source_storage_for_test_run_source(&id)
            .await?;
        let output_storage = self.data_store.get_test_run_source_storage(&id).await?;

        let test_run_source = create_test_run_source(
            &test_run_config,
            &test_source_definition,
            input_storage,
            output_storage,
        )
        .await?;

        test_run.sources.insert(test_source_id, test_run_source);
        Ok(())
    }

    pub async fn add_test_query(
        &self,
        test_run_id: &TestRunId,
        mut test_run_query: TestRunQueryConfig,
    ) -> anyhow::Result<TestRunQueryId> {
        log::trace!("Adding TestRunQuery from {:?}", test_run_query);

        // If the TestRunHost is in an Error state, return an error.
        if let TestRunHostStatus::Error(msg) = &self.get_status().await? {
            anyhow::bail!("TestRunHost is in an Error state: {}", msg);
        };

        // Set the test run IDs from the parent TestRun
        test_run_query.test_id = Some(test_run_id.test_id.clone());
        test_run_query.test_repo_id = Some(test_run_id.test_repo_id.clone());
        test_run_query.test_run_id = Some(test_run_id.test_run_id.clone());

        let query_id = test_run_query.test_query_id.clone();
        let id = TestRunQueryId::new(test_run_id, &query_id);

        let mut test_runs_lock = self.test_runs.write().await;
        let test_run = test_runs_lock
            .get_mut(test_run_id)
            .ok_or_else(|| anyhow::anyhow!("TestRun not found: {:?}", test_run_id))?;

        if test_run.queries.contains_key(&query_id) {
            anyhow::bail!(
                "TestRun already contains TestRunQuery with ID: {}",
                query_id
            );
        }

        // Get the TestRepoStorage that is associated with the Repo for the TestRunQuery
        let repo = self
            .data_store
            .get_test_repo_storage(test_run_query.test_repo_id.as_ref().unwrap())
            .await?;
        repo.add_remote_test(test_run_query.test_id.as_ref().unwrap(), false)
            .await?;
        let test_query_definition = self
            .data_store
            .get_test_query_definition_for_test_run_query(&id)
            .await?;

        let definition = TestRunQueryDefinition::new(test_run_query, test_query_definition)?;
        log::trace!("TestRunQueryDefinition: {:?}", &definition);

        // Get the OUTPUT storage for the new TestRunQuery.
        // This is where the TestRunQuery will write the output to.
        let output_storage = self.data_store.get_test_run_query_storage(&id).await?;

        // Create the TestRunQuery and add it to the TestRun.
        let test_run_query_obj = TestRunQuery::new(definition, output_storage).await?;

        test_run.queries.insert(query_id, test_run_query_obj);

        Ok(id)
    }

    pub async fn add_test_reaction(
        &self,
        test_run_id: &TestRunId,
        mut test_run_reaction: TestRunReactionConfig,
    ) -> anyhow::Result<TestRunReactionId> {
        log::trace!("Adding TestRunReaction from {:?}", test_run_reaction);

        // If the TestRunHost is in an Error state, return an error.
        if let TestRunHostStatus::Error(msg) = &self.get_status().await? {
            anyhow::bail!("TestRunHost is in an Error state: {}", msg);
        };

        // Set the test run IDs from the parent TestRun
        test_run_reaction.test_id = Some(test_run_id.test_id.clone());
        test_run_reaction.test_repo_id = Some(test_run_id.test_repo_id.clone());
        test_run_reaction.test_run_id = Some(test_run_id.test_run_id.clone());

        let reaction_id = test_run_reaction.test_reaction_id.clone();
        let id = TestRunReactionId::new(test_run_id, &reaction_id);

        let mut test_runs_lock = self.test_runs.write().await;
        let test_run = test_runs_lock
            .get_mut(test_run_id)
            .ok_or_else(|| anyhow::anyhow!("TestRun not found: {:?}", test_run_id))?;

        if test_run.reactions.contains_key(&reaction_id) {
            anyhow::bail!(
                "TestRun already contains TestRunReaction with ID: {}",
                reaction_id
            );
        }

        // Get the TestRepoStorage that is associated with the Repo for the TestRunReaction
        let repo = self
            .data_store
            .get_test_repo_storage(test_run_reaction.test_repo_id.as_ref().unwrap())
            .await?;
        repo.add_remote_test(test_run_reaction.test_id.as_ref().unwrap(), false)
            .await?;

        // Get the test definition and extract the reaction definition
        let test_definition = self
            .data_store
            .get_test_definition(
                test_run_reaction.test_repo_id.as_ref().unwrap(),
                test_run_reaction.test_id.as_ref().unwrap(),
            )
            .await?;

        let test_reaction_definition = test_definition.get_test_reaction(&reaction_id)?;

        let reaction_handler_definition = test_reaction_definition
            .output_handler
            .clone()
            .ok_or_else(|| {
                anyhow::anyhow!("No reaction handler defined for reaction {}", reaction_id)
            })?;

        // Get output_loggers from the config
        let output_loggers = test_run_reaction.output_loggers.clone();

        let definition = TestRunReactionDefinition::new(
            test_run_reaction,
            test_reaction_definition.clone(),
            reaction_handler_definition,
            output_loggers,
        )?;
        log::trace!("TestRunReactionDefinition: {:?}", &definition);

        // Get the OUTPUT storage for the new TestRunReaction.
        // This is where the TestRunReaction will write the output to.
        let output_storage = self.data_store.get_test_run_reaction_storage(&id).await?;

        // Create the TestRunReaction and add it to the TestRun.
        let test_run_reaction_obj = TestRunReaction::new(definition, output_storage).await?;

        test_run
            .reactions
            .insert(reaction_id, test_run_reaction_obj);

        Ok(id)
    }

    pub async fn add_test_source(
        &self,
        test_run_id: &TestRunId,
        mut test_run_config: TestRunSourceConfig,
    ) -> anyhow::Result<TestRunSourceId> {
        log::trace!("Adding TestRunSource from {:?}", test_run_config);

        // If the TestRunHost is in an Error state, return an error.
        if let TestRunHostStatus::Error(msg) = &self.get_status().await? {
            anyhow::bail!("TestRunHost is in an Error state: {}", msg);
        };

        // Set the test run IDs from the parent TestRun
        test_run_config.test_id = Some(test_run_id.test_id.clone());
        test_run_config.test_repo_id = Some(test_run_id.test_repo_id.clone());
        test_run_config.test_run_id = Some(test_run_id.test_run_id.clone());

        let source_id = test_run_config.test_source_id.clone();
        let id = TestRunSourceId::new(test_run_id, &source_id);

        let mut test_runs_lock = self.test_runs.write().await;
        let test_run = test_runs_lock
            .get_mut(test_run_id)
            .ok_or_else(|| anyhow::anyhow!("TestRun not found: {:?}", test_run_id))?;

        if test_run.sources.contains_key(&source_id) {
            anyhow::bail!(
                "TestRun already contains TestRunSource with ID: {}",
                source_id
            );
        }

        // Get the TestRepoStorage that is associated with the Repo for the TestRunSource
        let repo = self
            .data_store
            .get_test_repo_storage(test_run_config.test_repo_id.as_ref().unwrap())
            .await?;
        repo.add_remote_test(test_run_config.test_id.as_ref().unwrap(), false)
            .await?;
        let test_source_definition = self
            .data_store
            .get_test_source_definition_for_test_run_source(&id)
            .await?;

        // Get the INPUT Test Data storage for the TestRunSource.
        // This is where the TestRunSource will read the Test Data from.
        let input_storage = self
            .data_store
            .get_test_source_storage_for_test_run_source(&id)
            .await?;

        // Get the OUTPUT storage for the new TestRunSource.
        // This is where the TestRunSource will write the output to.
        let output_storage = self.data_store.get_test_run_source_storage(&id).await?;

        // Create the TestRunSource and add it to the TestRun.
        let test_run_source = create_test_run_source(
            &test_run_config,
            &test_source_definition,
            input_storage,
            output_storage,
        )
        .await?;
        test_run.sources.insert(source_id, test_run_source);

        Ok(id)
    }

    pub async fn contains_test_source(&self, test_run_source_id: &str) -> anyhow::Result<bool> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        if let Some(test_run) = test_runs.get(&test_run_source_id.test_run_id) {
            Ok(test_run
                .sources
                .contains_key(&test_run_source_id.test_source_id))
        } else {
            Ok(false)
        }
    }

    pub async fn get_status(&self) -> anyhow::Result<TestRunHostStatus> {
        Ok(self.status.read().await.clone())
    }

    pub async fn get_source_bootstrap_data(
        &self,
        test_run_source_id: &str,
        node_labels: &HashSet<String>,
        rel_labels: &HashSet<String>,
    ) -> anyhow::Result<BootstrapData> {
        log::debug!(
            "Source ID: {}, Node Labels: {:?}, Rel Labels: {:?}",
            test_run_source_id,
            node_labels,
            rel_labels
        );

        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => source.get_bootstrap_data(node_labels, rel_labels).await,
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn get_test_query_ids(&self) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::new();
        let test_runs = self.test_runs.read().await;
        for test_run in test_runs.values() {
            for query_id in test_run.queries.keys() {
                ids.push(format!("{}.{}", test_run.id, query_id));
            }
        }
        Ok(ids)
    }

    pub async fn get_test_query_state(
        &self,
        test_run_query_id: &str,
    ) -> anyhow::Result<TestRunQueryState> {
        let test_run_query_id = TestRunQueryId::try_from(test_run_query_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_query_id.test_run_id) {
            Some(test_run) => match test_run.queries.get(&test_run_query_id.test_query_id) {
                Some(query) => query.get_state().await,
                None => anyhow::bail!("TestRunQuery not found: {:?}", test_run_query_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_query_id.test_run_id),
        }
    }

    pub async fn get_test_query_result_logger_output(
        &self,
        test_run_query_id: &str,
    ) -> anyhow::Result<Vec<ResultStreamLoggerResult>> {
        let test_run_query_id = TestRunQueryId::try_from(test_run_query_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_query_id.test_run_id) {
            Some(test_run) => match test_run.queries.get(&test_run_query_id.test_query_id) {
                Some(query) => Ok(query
                    .get_query_result_observer_state()
                    .await?
                    .logger_results),
                None => anyhow::bail!("TestRunQuery not found: {:?}", test_run_query_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_query_id.test_run_id),
        }
    }

    pub async fn get_test_source_ids(&self) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::new();
        let test_runs = self.test_runs.read().await;
        for test_run in test_runs.values() {
            for source_id in test_run.sources.keys() {
                ids.push(format!("{}.{}", test_run.id, source_id));
            }
        }
        Ok(ids)
    }

    pub async fn get_test_source_state(
        &self,
        test_run_source_id: &str,
    ) -> anyhow::Result<TestRunSourceState> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => source.get_state().await,
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    async fn set_status(&self, status: TestRunHostStatus) {
        let mut write_lock = self.status.write().await;
        *write_lock = status.clone();
    }

    pub async fn test_query_pause(
        &self,
        test_run_query_id: &str,
    ) -> anyhow::Result<QueryResultObserverCommandResponse> {
        let test_run_query_id = TestRunQueryId::try_from(test_run_query_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_query_id.test_run_id) {
            Some(test_run) => match test_run.queries.get(&test_run_query_id.test_query_id) {
                Some(query) => query.pause_query_result_observer().await,
                None => anyhow::bail!("TestRunQuery not found: {:?}", test_run_query_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_query_id.test_run_id),
        }
    }

    pub async fn test_query_reset(
        &self,
        test_run_query_id: &str,
    ) -> anyhow::Result<QueryResultObserverCommandResponse> {
        let test_run_query_id = TestRunQueryId::try_from(test_run_query_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_query_id.test_run_id) {
            Some(test_run) => match test_run.queries.get(&test_run_query_id.test_query_id) {
                Some(query) => query.reset_query_result_observer().await,
                None => anyhow::bail!("TestRunQuery not found: {:?}", test_run_query_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_query_id.test_run_id),
        }
    }

    pub async fn test_query_start(
        &self,
        test_run_query_id: &str,
    ) -> anyhow::Result<QueryResultObserverCommandResponse> {
        let test_run_query_id = TestRunQueryId::try_from(test_run_query_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_query_id.test_run_id) {
            Some(test_run) => match test_run.queries.get(&test_run_query_id.test_query_id) {
                Some(query) => query.start_query_result_observer().await,
                None => anyhow::bail!("TestRunQuery not found: {:?}", test_run_query_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_query_id.test_run_id),
        }
    }

    pub async fn test_query_stop(
        &self,
        test_run_query_id: &str,
    ) -> anyhow::Result<QueryResultObserverCommandResponse> {
        let test_run_query_id = TestRunQueryId::try_from(test_run_query_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_query_id.test_run_id) {
            Some(test_run) => match test_run.queries.get(&test_run_query_id.test_query_id) {
                Some(query) => query.stop_query_result_observer().await,
                None => anyhow::bail!("TestRunQuery not found: {:?}", test_run_query_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_query_id.test_run_id),
        }
    }

    pub async fn get_test_reaction_ids(&self) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::new();
        let test_runs = self.test_runs.read().await;
        for test_run in test_runs.values() {
            for reaction_id in test_run.reactions.keys() {
                ids.push(format!("{}.{}", test_run.id, reaction_id));
            }
        }
        Ok(ids)
    }

    pub async fn get_test_reaction_state(
        &self,
        test_run_reaction_id: &str,
    ) -> anyhow::Result<TestRunReactionState> {
        let test_run_reaction_id = TestRunReactionId::try_from(test_run_reaction_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_reaction_id.test_run_id) {
            Some(test_run) => match test_run
                .reactions
                .get(&test_run_reaction_id.test_reaction_id)
            {
                Some(reaction) => reaction.get_state().await,
                None => anyhow::bail!("TestRunReaction not found: {:?}", test_run_reaction_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_reaction_id.test_run_id),
        }
    }

    pub async fn test_reaction_pause(
        &self,
        test_run_reaction_id: &str,
    ) -> anyhow::Result<ReactionObserverCommandResponse> {
        let test_run_reaction_id = TestRunReactionId::try_from(test_run_reaction_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_reaction_id.test_run_id) {
            Some(test_run) => match test_run
                .reactions
                .get(&test_run_reaction_id.test_reaction_id)
            {
                Some(reaction) => reaction.pause_reaction_observer().await,
                None => anyhow::bail!("TestRunReaction not found: {:?}", test_run_reaction_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_reaction_id.test_run_id),
        }
    }

    pub async fn test_reaction_reset(
        &self,
        test_run_reaction_id: &str,
    ) -> anyhow::Result<ReactionObserverCommandResponse> {
        let test_run_reaction_id = TestRunReactionId::try_from(test_run_reaction_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_reaction_id.test_run_id) {
            Some(test_run) => match test_run
                .reactions
                .get(&test_run_reaction_id.test_reaction_id)
            {
                Some(reaction) => reaction.reset_reaction_observer().await,
                None => anyhow::bail!("TestRunReaction not found: {:?}", test_run_reaction_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_reaction_id.test_run_id),
        }
    }

    pub async fn test_reaction_start(
        &self,
        test_run_reaction_id: &str,
    ) -> anyhow::Result<ReactionObserverCommandResponse> {
        let test_run_reaction_id = TestRunReactionId::try_from(test_run_reaction_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_reaction_id.test_run_id) {
            Some(test_run) => match test_run
                .reactions
                .get(&test_run_reaction_id.test_reaction_id)
            {
                Some(reaction) => reaction.start_reaction_observer().await,
                None => anyhow::bail!("TestRunReaction not found: {:?}", test_run_reaction_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_reaction_id.test_run_id),
        }
    }

    pub async fn test_reaction_stop(
        &self,
        test_run_reaction_id: &str,
    ) -> anyhow::Result<ReactionObserverCommandResponse> {
        let test_run_reaction_id = TestRunReactionId::try_from(test_run_reaction_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_reaction_id.test_run_id) {
            Some(test_run) => match test_run
                .reactions
                .get(&test_run_reaction_id.test_reaction_id)
            {
                Some(reaction) => reaction.stop_reaction_observer().await,
                None => anyhow::bail!("TestRunReaction not found: {:?}", test_run_reaction_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_reaction_id.test_run_id),
        }
    }

    pub async fn test_source_pause(
        &self,
        test_run_source_id: &str,
    ) -> anyhow::Result<SourceChangeGeneratorCommandResponse> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => source.pause_source_change_generator().await,
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn test_source_reset(
        &self,
        test_run_source_id: &str,
    ) -> anyhow::Result<SourceChangeGeneratorCommandResponse> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => source.reset_source_change_generator().await,
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn test_source_skip(
        &self,
        test_run_source_id: &str,
        skips: u64,
        spacing_mode: Option<SpacingMode>,
    ) -> anyhow::Result<SourceChangeGeneratorCommandResponse> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => {
                    source
                        .skip_source_change_generator(skips, spacing_mode)
                        .await
                }
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn test_source_start(
        &self,
        test_run_source_id: &str,
    ) -> anyhow::Result<SourceChangeGeneratorCommandResponse> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => source.start_source_change_generator().await,
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn test_source_step(
        &self,
        test_run_source_id: &str,
        steps: u64,
        spacing_mode: Option<SpacingMode>,
    ) -> anyhow::Result<SourceChangeGeneratorCommandResponse> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => {
                    source
                        .step_source_change_generator(steps, spacing_mode)
                        .await
                }
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn test_source_stop(
        &self,
        test_run_source_id: &str,
    ) -> anyhow::Result<SourceChangeGeneratorCommandResponse> {
        let test_run_source_id = TestRunSourceId::try_from(test_run_source_id)?;
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_source_id.test_run_id) {
            Some(test_run) => match test_run.sources.get(&test_run_source_id.test_source_id) {
                Some(source) => source.stop_source_change_generator().await,
                None => anyhow::bail!("TestRunSource not found: {:?}", test_run_source_id),
            },
            None => anyhow::bail!("TestRun not found: {:?}", test_run_source_id.test_run_id),
        }
    }

    pub async fn add_test_drasi_server(
        &self,
        test_run_id: &TestRunId,
        mut test_run_drasi_server: TestRunDrasiServerConfig,
    ) -> anyhow::Result<TestRunDrasiServerId> {
        log::trace!("Adding TestRunDrasiServer from {:?}", test_run_drasi_server);

        // If the TestRunHost is in an Error state, return an error.
        if let TestRunHostStatus::Error(msg) = &self.get_status().await? {
            anyhow::bail!("TestRunHost is in an Error state: {}", msg);
        };

        // Set the test run IDs from the parent TestRun
        test_run_drasi_server.test_id = Some(test_run_id.test_id.clone());
        test_run_drasi_server.test_repo_id = Some(test_run_id.test_repo_id.clone());
        test_run_drasi_server.test_run_id = Some(test_run_id.test_run_id.clone());

        let server_id = test_run_drasi_server.test_drasi_server_id.clone();
        let id = TestRunDrasiServerId::new(test_run_id, &server_id);

        let mut test_runs_lock = self.test_runs.write().await;
        let test_run = test_runs_lock
            .get_mut(test_run_id)
            .ok_or_else(|| anyhow::anyhow!("TestRun not found: {:?}", test_run_id))?;

        if test_run.drasi_servers.contains_key(&server_id) {
            anyhow::bail!(
                "TestRun already contains TestRunDrasiServer with ID: {}",
                server_id
            );
        }

        // Get the test definition and extract the drasi server definition
        // Note: Local tests are already loaded when the repository is initialized,
        // so we don't need to call add_remote_test here
        let test_definition = self
            .data_store
            .get_test_definition(
                test_run_drasi_server.test_repo_id.as_ref().unwrap(),
                test_run_drasi_server.test_id.as_ref().unwrap(),
            )
            .await?;

        let test_drasi_server_definition = test_definition
            .drasi_servers
            .iter()
            .find(|s| s.id == server_id)
            .ok_or_else(|| anyhow::anyhow!("Drasi server definition not found: {}", server_id))?
            .clone();

        let definition =
            TestRunDrasiServerDefinition::new(test_run_drasi_server, test_drasi_server_definition)?;
        log::trace!("TestRunDrasiServerDefinition: {:?}", &definition);

        // Get the OUTPUT storage for the new TestRunDrasiServer.
        let output_storage = self
            .data_store
            .get_test_run_drasi_server_storage(&id)
            .await?;

        // Create the TestRunDrasiServer and add it to the TestRun.
        let test_run_drasi_server_obj = TestRunDrasiServer::new(definition, output_storage).await?;

        test_run
            .drasi_servers
            .insert(server_id, test_run_drasi_server_obj);

        Ok(id)
    }

    pub async fn get_test_drasi_server(
        &self,
        test_run_drasi_server_id: &TestRunDrasiServerId,
    ) -> anyhow::Result<Option<TestRunDrasiServerState>> {
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_drasi_server_id.test_run_id) {
            Some(test_run) => match test_run
                .drasi_servers
                .get(&test_run_drasi_server_id.test_drasi_server_id)
            {
                Some(server) => Ok(Some(server.get_state().await)),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub async fn remove_test_drasi_server(
        &self,
        test_run_drasi_server_id: &TestRunDrasiServerId,
    ) -> anyhow::Result<()> {
        let mut test_runs_lock = self.test_runs.write().await;
        match test_runs_lock.get_mut(&test_run_drasi_server_id.test_run_id) {
            Some(test_run) => {
                if let Some(server) = test_run
                    .drasi_servers
                    .remove(&test_run_drasi_server_id.test_drasi_server_id)
                {
                    // Stop the server if it's running
                    if matches!(
                        server.get_state().await,
                        TestRunDrasiServerState::Running { .. }
                    ) {
                        server
                            .stop(Some("Removing from TestRun".to_string()))
                            .await?;
                    }
                    Ok(())
                } else {
                    anyhow::bail!(
                        "TestRunDrasiServer not found: {:?}",
                        test_run_drasi_server_id
                    );
                }
            }
            None => anyhow::bail!(
                "TestRun not found: {:?}",
                test_run_drasi_server_id.test_run_id
            ),
        }
    }

    pub async fn get_drasi_server_endpoint(
        &self,
        test_run_drasi_server_id: &TestRunDrasiServerId,
    ) -> anyhow::Result<Option<String>> {
        let test_runs = self.test_runs.read().await;
        match test_runs.get(&test_run_drasi_server_id.test_run_id) {
            Some(test_run) => match test_run
                .drasi_servers
                .get(&test_run_drasi_server_id.test_drasi_server_id)
            {
                Some(server) => Ok(server.get_api_endpoint().await),
                None => Ok(None),
            },
            None => Ok(None),
        }
    }

    pub async fn get_test_drasi_server_ids(&self) -> anyhow::Result<Vec<String>> {
        let mut ids = Vec::new();
        let test_runs = self.test_runs.read().await;
        for test_run in test_runs.values() {
            for server_id in test_run.drasi_servers.keys() {
                ids.push(format!("{}.{}", test_run.id, server_id));
            }
        }
        Ok(ids)
    }

    // New TestRun lifecycle management methods
    pub async fn get_test_run_ids(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .test_runs
            .read()
            .await
            .keys()
            .map(|id| id.to_string())
            .collect())
    }

    pub async fn get_test_run_status(
        &self,
        test_run_id: &TestRunId,
    ) -> anyhow::Result<TestRunStatus> {
        let test_runs = self.test_runs.read().await;
        match test_runs.get(test_run_id) {
            Some(test_run) => Ok(test_run.status.clone()),
            None => anyhow::bail!("TestRun not found: {:?}", test_run_id),
        }
    }

    pub async fn start_test_run(&self, test_run_id: &TestRunId) -> anyhow::Result<()> {
        let mut test_runs = self.test_runs.write().await;
        match test_runs.get_mut(test_run_id) {
            Some(test_run) => {
                // Start drasi servers first
                for server in test_run.drasi_servers.values() {
                    if matches!(
                        server.get_state().await,
                        TestRunDrasiServerState::Uninitialized { .. }
                    ) {
                        server.start().await?;
                    }
                }

                // Start sources
                for source in test_run.sources.values() {
                    let state = source.get_state().await?;
                    if state.start_mode == SourceStartMode::Auto {
                        source.start_source_change_generator().await?;
                    }
                }

                // Start queries
                for query in test_run.queries.values() {
                    query.start_query_result_observer().await?;
                }

                // Start reactions
                for reaction in test_run.reactions.values() {
                    if reaction.start_immediately {
                        reaction.start_reaction_observer().await?;
                    }
                }

                test_run.status = TestRunStatus::Running;
                Ok(())
            }
            None => anyhow::bail!("TestRun not found: {:?}", test_run_id),
        }
    }

    pub async fn stop_test_run(&self, test_run_id: &TestRunId) -> anyhow::Result<()> {
        let mut test_runs = self.test_runs.write().await;
        match test_runs.get_mut(test_run_id) {
            Some(test_run) => {
                // Stop reactions first
                for reaction in test_run.reactions.values() {
                    reaction.stop_reaction_observer().await?;
                }

                // Stop queries
                for query in test_run.queries.values() {
                    query.stop_query_result_observer().await?;
                }

                // Stop sources
                for source in test_run.sources.values() {
                    source.stop_source_change_generator().await?;
                }

                // Stop drasi servers
                for server in test_run.drasi_servers.values() {
                    if matches!(
                        server.get_state().await,
                        TestRunDrasiServerState::Running { .. }
                    ) {
                        server.stop(Some("Stopping TestRun".to_string())).await?;
                    }
                }

                test_run.status = TestRunStatus::Stopped;
                Ok(())
            }
            None => anyhow::bail!("TestRun not found: {:?}", test_run_id),
        }
    }

    pub async fn delete_test_run(&self, test_run_id: &TestRunId) -> anyhow::Result<()> {
        // First stop the test run if it's running
        let status = self.get_test_run_status(test_run_id).await?;
        if status == TestRunStatus::Running {
            self.stop_test_run(test_run_id).await?;
        }

        // Remove the test run
        let mut test_runs = self.test_runs.write().await;
        test_runs
            .remove(test_run_id)
            .ok_or_else(|| anyhow::anyhow!("TestRun not found: {:?}", test_run_id))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use test_data_store::TestDataStore;

    use crate::{TestRunHost, TestRunHostConfig, TestRunHostStatus};

    #[tokio::test]
    async fn test_new_test_run_host() -> anyhow::Result<()> {
        let data_store = Arc::new(TestDataStore::new_temp(None).await?);

        let test_run_host_config = TestRunHostConfig::default();
        let test_run_host = TestRunHost::new(test_run_host_config, data_store.clone())
            .await
            .unwrap();

        assert_eq!(
            test_run_host.get_status().await?,
            TestRunHostStatus::Running
        );

        Ok(())
    }
}
