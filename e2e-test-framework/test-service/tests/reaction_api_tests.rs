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

use std::sync::Arc;

use test_data_store::{TestDataStore, test_repo_storage::{
    repo_clients::{TestRepoConfig, CommonTestRepoConfig, LocalStorageTestRepoConfig},
    models::{LocalTestDefinition, TestReactionDefinition, HttpReactionHandlerDefinition}
}};
use test_run_host::{reactions::{TestRunReactionConfig, reaction_observer::ReactionObserverStatus}, TestRunHost, TestRunHostConfig};

#[tokio::test]
async fn test_get_reaction_list_empty() -> anyhow::Result<()> {
    let data_store = Arc::new(TestDataStore::new_temp(None).await?);
    let test_run_host_config = TestRunHostConfig::default();
    let test_run_host = Arc::new(TestRunHost::new(test_run_host_config, data_store.clone()).await?);

    let reaction_ids = test_run_host.get_test_reaction_ids().await?;
    assert_eq!(reaction_ids.len(), 0);

    Ok(())
}

#[tokio::test]
async fn test_add_reaction() -> anyhow::Result<()> {
    let data_store = Arc::new(TestDataStore::new_temp(None).await?);

    // Add a test repository first
    let repo_id = "test-repo";
    let repo_config = TestRepoConfig::LocalStorage {
        common_config: CommonTestRepoConfig {
            id: repo_id.to_string(),
            local_tests: Vec::new(),
        },
        unique_config: LocalStorageTestRepoConfig { source_path: None },
    };
    data_store.add_test_repo(repo_config).await?;
    
    let test_id = "test-001";

    // Create test definition with proper structure
    let test_def = LocalTestDefinition {
        test_id: test_id.to_string(),
        version: 1,
        description: Some("Test with Reaction".to_string()),
        test_folder: Some("test".to_string()),
        sources: vec![],
        queries: vec![],
        reactions: vec![TestReactionDefinition {
            test_reaction_id: "reaction-001".to_string(),
            output_handler: Some(test_data_store::test_repo_storage::models::ReactionHandlerDefinition::Http(
                HttpReactionHandlerDefinition {
                    host: Some("localhost".to_string()),
                    port: Some(8080),
                    path: Some("/callback".to_string()),
                    correlation_header: None,
                }
            )),
            stop_triggers: None,
        }],
    };

    // Add the test definition to repository
    data_store.add_local_test(repo_id, test_def, false).await?;

    let test_run_host_config = TestRunHostConfig::default();
    let test_run_host = Arc::new(TestRunHost::new(test_run_host_config, data_store.clone()).await?);

    let reaction_config = TestRunReactionConfig {
        start_immediately: false,
        test_id: test_id.to_string(),
        test_repo_id: repo_id.to_string(),
        test_run_id: Some("run-001".to_string()),
        test_reaction_id: "reaction-001".to_string(),
        test_run_overrides: None,
        output_loggers: vec![],
    };

    let reaction_id = test_run_host.add_test_reaction(reaction_config).await?;

    let reaction_ids = test_run_host.get_test_reaction_ids().await?;
    assert_eq!(reaction_ids.len(), 1);
    assert!(reaction_ids.contains(&reaction_id.to_string()));

    Ok(())
}

#[tokio::test]
async fn test_reaction_state_transitions() -> anyhow::Result<()> {
    let data_store = Arc::new(TestDataStore::new_temp(None).await?);

    // Add a test repository first
    let repo_id = "test-repo";
    let repo_config = TestRepoConfig::LocalStorage {
        common_config: CommonTestRepoConfig {
            id: repo_id.to_string(),
            local_tests: Vec::new(),
        },
        unique_config: LocalStorageTestRepoConfig { source_path: None },
    };
    data_store.add_test_repo(repo_config).await?;
    
    let test_id = "test-002";

    // Create test definition with proper structure
    let test_def = LocalTestDefinition {
        test_id: test_id.to_string(),
        version: 1,
        description: Some("Test with Reaction".to_string()),
        test_folder: Some("test".to_string()),
        sources: vec![],
        queries: vec![],
        reactions: vec![TestReactionDefinition {
            test_reaction_id: "reaction-002".to_string(),
            output_handler: Some(test_data_store::test_repo_storage::models::ReactionHandlerDefinition::Http(
                HttpReactionHandlerDefinition {
                    host: Some("localhost".to_string()),
                    port: Some(8080),
                    path: Some("/callback".to_string()),
                    correlation_header: None,
                }
            )),
            stop_triggers: None,
        }],
    };

    // Add the test definition to repository
    data_store.add_local_test(repo_id, test_def, false).await?;

    let test_run_host_config = TestRunHostConfig::default();
    let test_run_host = Arc::new(TestRunHost::new(test_run_host_config, data_store.clone()).await?);

    let reaction_config = TestRunReactionConfig {
        start_immediately: false,
        test_id: test_id.to_string(),
        test_repo_id: repo_id.to_string(),
        test_run_id: Some("run-002".to_string()),
        test_reaction_id: "reaction-002".to_string(),
        test_run_overrides: None,
        output_loggers: vec![],
    };

    let reaction_id = test_run_host.add_test_reaction(reaction_config).await?;

    // Test initial state
    let state = test_run_host.get_test_reaction_state(&reaction_id.to_string()).await?;
    assert_eq!(state.reaction_observer.status, ReactionObserverStatus::Stopped);

    // Test start transition
    test_run_host.test_reaction_start(&reaction_id.to_string()).await?;
    let state = test_run_host.get_test_reaction_state(&reaction_id.to_string()).await?;
    assert_eq!(state.reaction_observer.status, ReactionObserverStatus::Running);

    // Test pause transition
    test_run_host.test_reaction_pause(&reaction_id.to_string()).await?;
    let state = test_run_host.get_test_reaction_state(&reaction_id.to_string()).await?;
    assert_eq!(state.reaction_observer.status, ReactionObserverStatus::Paused);

    // Test stop transition
    test_run_host.test_reaction_stop(&reaction_id.to_string()).await?;
    let state = test_run_host.get_test_reaction_state(&reaction_id.to_string()).await?;
    assert_eq!(state.reaction_observer.status, ReactionObserverStatus::Stopped);

    // Test reset from stopped state - should fail
    let result = test_run_host.test_reaction_reset(&reaction_id.to_string()).await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_reaction_with_loggers() -> anyhow::Result<()> {
    let data_store = Arc::new(TestDataStore::new_temp(None).await?);

    // Add a test repository first
    let repo_id = "test-repo";
    let repo_config = TestRepoConfig::LocalStorage {
        common_config: CommonTestRepoConfig {
            id: repo_id.to_string(),
            local_tests: Vec::new(),
        },
        unique_config: LocalStorageTestRepoConfig { source_path: None },
    };
    data_store.add_test_repo(repo_config).await?;
    
    let test_id = "test-003";

    // Create test definition with loggers and stop triggers
    let test_def = LocalTestDefinition {
        test_id: test_id.to_string(),
        version: 1,
        description: Some("Test with Reaction and Loggers".to_string()),
        test_folder: Some("test".to_string()),
        sources: vec![],
        queries: vec![],
        reactions: vec![TestReactionDefinition {
            test_reaction_id: "reaction-003".to_string(),
            output_handler: Some(test_data_store::test_repo_storage::models::ReactionHandlerDefinition::Http(
                HttpReactionHandlerDefinition {
                    host: Some("localhost".to_string()),
                    port: Some(8080),
                    path: Some("/callback".to_string()),
                    correlation_header: None,
                }
            )),
            stop_triggers: Some(vec![
                test_data_store::test_repo_storage::models::StopTriggerDefinition::RecordCount(
                    test_data_store::test_repo_storage::models::RecordCountStopTriggerDefinition {
                        record_count: 10,
                    }
                ),
            ]),
        }],
    };

    // Add the test definition to repository
    data_store.add_local_test(repo_id, test_def, false).await?;

    let test_run_host_config = TestRunHostConfig::default();
    let test_run_host = Arc::new(TestRunHost::new(test_run_host_config, data_store.clone()).await?);

    let reaction_config = TestRunReactionConfig {
        start_immediately: true, // Start immediately
        test_id: test_id.to_string(),
        test_repo_id: repo_id.to_string(),
        test_run_id: Some("run-003".to_string()),
        test_reaction_id: "reaction-003".to_string(),
        test_run_overrides: None,
        output_loggers: vec![
            test_run_host::reactions::output_loggers::OutputLoggerConfig::Console(
                test_run_host::reactions::output_loggers::ConsoleOutputLoggerConfig {
                    date_time_format: Some("%Y-%m-%d %H:%M:%S".to_string()),
                }
            ),
            test_run_host::reactions::output_loggers::OutputLoggerConfig::JsonlFile(
                test_run_host::reactions::output_loggers::JsonlFileOutputLoggerConfig {
                    max_lines_per_file: Some(1000),
                }
            ),
        ],
    };

    let reaction_id = test_run_host.add_test_reaction(reaction_config).await?;

    // Verify it started immediately
    let state = test_run_host.get_test_reaction_state(&reaction_id.to_string()).await?;
    assert!(state.start_immediately);
    assert_eq!(state.reaction_observer.status, ReactionObserverStatus::Running);

    // Stop and check logger results
    test_run_host.test_reaction_stop(&reaction_id.to_string()).await?;
    let state = test_run_host.get_test_reaction_state(&reaction_id.to_string()).await?;
    
    // The reaction should have the configured loggers
    assert_eq!(state.reaction_observer.settings.loggers.len(), 2);
    
    Ok(())
}