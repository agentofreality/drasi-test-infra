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

use serde::Serialize;
#[allow(unused_imports)]
use serde_json::json;
use utoipa::{OpenApi, ToSchema};

use crate::web_api::{
    drasi_server_queries, drasi_server_reactions, drasi_server_sources, drasi_servers, queries,
    reactions, repo, sources, test_runs, DataCollectorStateResponse, TestDataStoreStateResponse,
    TestRunHostStateResponse, TestServiceStateResponse,
};

/// Standard error response for all API endpoints
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({"error": "Resource not found", "details": "TestSource with ID source-123 not found"}))]
pub struct ErrorResponse {
    /// Error message
    pub error: String,
    /// Additional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// Source state response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": {"test_id": "test-123", "run_id": "run-456", "source_id": "source-789"},
    "source_change_generator": {
        "state": {"current_index": 0, "total_changes": 100},
        "status": "Running"
    },
    "start_mode": "StartImmediately"
}))]
pub struct SourceStateResponse {
    /// Source identifier
    pub id: serde_json::Value,
    /// Source change generator state
    pub source_change_generator: SourceChangeGeneratorState,
    /// Start mode configuration
    pub start_mode: String,
}

/// Source change generator state
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "state": {"current_index": 0, "total_changes": 100, "elapsed_time": "00:02:30"},
    "status": "Running"
}))]
pub struct SourceChangeGeneratorState {
    /// Generator-specific state data
    pub state: serde_json::Value,
    /// Current status
    pub status: String,
}

/// Query state response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": {"test_id": "test-123", "run_id": "run-456", "query_id": "query-789"},
    "query_observer": {
        "status": "Running",
        "stream_status": "Active",
        "error_message": null,
        "result_summary": {
            "record_count": 150,
            "elapsed_time": "00:01:45"
        }
    },
    "start_immediately": true
}))]
pub struct QueryStateResponse {
    /// Query identifier
    pub id: serde_json::Value,
    /// Query observer state
    pub query_observer: QueryObserverState,
    /// Whether to start immediately
    pub start_immediately: bool,
}

/// Query observer state
#[derive(Debug, Serialize, ToSchema)]
pub struct QueryObserverState {
    /// Observer status
    pub status: String,
    /// Stream status
    pub stream_status: String,
    /// Error message if any
    pub error_message: Option<String>,
    /// Result summary
    pub result_summary: serde_json::Value,
    /// Observer settings
    pub settings: serde_json::Value,
}

/// Reaction state response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "id": {"test_id": "test-123", "run_id": "run-456", "query_id": "reaction-789"},
    "reaction_observer": {
        "status": "Running",
        "stream_status": "Active",
        "error_message": null,
        "result_summary": {
            "invocation_count": 25,
            "elapsed_time": "00:02:15"
        }
    },
    "start_immediately": true
}))]
pub struct ReactionStateResponse {
    /// Reaction identifier
    pub id: serde_json::Value,
    /// Reaction observer state
    pub reaction_observer: ReactionObserverState,
    /// Whether to start immediately
    pub start_immediately: bool,
}

/// Reaction observer state
#[derive(Debug, Serialize, ToSchema)]
pub struct ReactionObserverState {
    /// Observer status
    pub status: String,
    /// Stream status
    pub stream_status: String,
    /// Error message if any
    pub error_message: Option<String>,
    /// Result summary
    pub result_summary: serde_json::Value,
    /// Observer settings
    pub settings: serde_json::Value,
}

/// Source bootstrap response
#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "nodes_created": 100,
    "relations_created": 250,
    "elapsed_time": "00:00:05"
}))]
pub struct SourceBootstrapResponseBody {
    /// Number of nodes created
    pub nodes_created: u64,
    /// Number of relations created
    pub relations_created: u64,
    /// Time taken to bootstrap
    pub elapsed_time: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::web_api::get_service_info_handler,
        // Drasi Server endpoints
        drasi_servers::create_drasi_server,
        drasi_servers::list_drasi_servers,
        drasi_servers::get_drasi_server,
        drasi_servers::delete_drasi_server,
        drasi_servers::get_drasi_server_status,
        // Drasi Server Sources endpoints
        drasi_server_sources::list_sources,
        drasi_server_sources::get_source,
        drasi_server_sources::create_source,
        drasi_server_sources::update_source,
        drasi_server_sources::delete_source,
        drasi_server_sources::start_source,
        drasi_server_sources::stop_source,
        // Drasi Server Queries endpoints
        drasi_server_queries::list_queries,
        drasi_server_queries::get_query,
        drasi_server_queries::create_query,
        drasi_server_queries::update_query,
        drasi_server_queries::delete_query,
        drasi_server_queries::start_query,
        drasi_server_queries::stop_query,
        drasi_server_queries::get_query_results,
        // Drasi Server Reactions endpoints
        drasi_server_reactions::list_reactions,
        drasi_server_reactions::get_reaction,
        drasi_server_reactions::create_reaction,
        drasi_server_reactions::update_reaction,
        drasi_server_reactions::delete_reaction,
        drasi_server_reactions::start_reaction,
        drasi_server_reactions::stop_reaction,
        // Source endpoints
        sources::get_source_list_handler,
        sources::get_source_handler,
        sources::post_source_handler,
        sources::source_bootstrap_handler,
        sources::source_change_generator_pause_handler,
        sources::source_change_generator_reset_handler,
        sources::source_change_generator_skip_handler,
        sources::source_change_generator_start_handler,
        sources::source_change_generator_step_handler,
        sources::source_change_generator_stop_handler,
        // Query endpoints
        queries::get_query_list_handler,
        queries::get_query_handler,
        queries::post_query_handler,
        queries::get_query_result_profile_handler,
        queries::query_observer_pause_handler,
        queries::query_observer_reset_handler,
        queries::query_observer_start_handler,
        queries::query_observer_stop_handler,
        // Reaction endpoints
        reactions::get_reaction_list_handler,
        reactions::get_reaction_handler,
        reactions::post_reaction_handler,
        reactions::reaction_observer_pause_handler,
        reactions::reaction_observer_reset_handler,
        reactions::reaction_observer_start_handler,
        reactions::reaction_observer_stop_handler,
        // Repository endpoints
        repo::get_test_repo_list_handler,
        repo::get_test_repo_handler,
        repo::post_test_repo_handler,
        repo::get_test_repo_test_list_handler,
        repo::get_test_repo_test_handler,
        repo::post_test_repo_test_handler,
        repo::get_test_repo_test_source_list_handler,
        repo::get_test_repo_test_source_handler,
        repo::post_test_repo_test_source_handler,
        // Test Run endpoints
        test_runs::create_test_run,
        test_runs::list_test_runs,
        test_runs::get_test_run,
        test_runs::delete_test_run,
        test_runs::start_test_run,
        test_runs::stop_test_run,
        // Test Run Source endpoints
        test_runs::list_test_run_sources,
        test_runs::create_test_run_source,
        test_runs::get_test_run_source,
        test_runs::delete_test_run_source,
        test_runs::start_test_run_source,
        test_runs::stop_test_run_source,
        test_runs::pause_test_run_source,
        test_runs::reset_test_run_source,
        // Test Run Query endpoints
        test_runs::list_test_run_queries,
        test_runs::create_test_run_query,
        test_runs::get_test_run_query,
        test_runs::delete_test_run_query,
        test_runs::start_test_run_query,
        test_runs::stop_test_run_query,
        test_runs::pause_test_run_query,
        test_runs::reset_test_run_query,
        // Test Run Reaction endpoints
        test_runs::list_test_run_reactions,
        test_runs::create_test_run_reaction,
        test_runs::get_test_run_reaction,
        test_runs::delete_test_run_reaction,
        test_runs::start_test_run_reaction,
        test_runs::stop_test_run_reaction,
        test_runs::pause_test_run_reaction,
        test_runs::reset_test_run_reaction,
        // Test Run Drasi Server endpoints
        test_runs::list_test_run_drasi_servers,
        test_runs::create_test_run_drasi_server,
        test_runs::get_test_run_drasi_server,
        test_runs::delete_test_run_drasi_server,
    ),
    components(
        schemas(
            // Common schemas
            ErrorResponse,
            // Service state schemas
            TestServiceStateResponse,
            TestDataStoreStateResponse,
            TestRunHostStateResponse,
            DataCollectorStateResponse,
            // Source schemas
            SourceStateResponse,
            SourceChangeGeneratorState,
            SourceBootstrapResponseBody,
            sources::TestSkipConfig,
            sources::TestStepConfig,
            sources::SourceBootstrapRequestBody,
            sources::Node,
            sources::Relation,
            // Query schemas
            QueryStateResponse,
            QueryObserverState,
            // Reaction schemas
            ReactionStateResponse,
            ReactionObserverState,
            // Repository schemas
            repo::TestRepoResponse,
            repo::TestPostBody,
            repo::TestResponse,
            repo::TestSourcePostBody,
            repo::TestSourceResponse,
            // Drasi Server schemas
            drasi_servers::DrasiServerCreatedResponse,
            drasi_servers::DrasiServerInfo,
            drasi_servers::DrasiServerDetails,
            drasi_servers::DrasiServerStatus,
            test_run_host::drasi_servers::TestRunDrasiServerConfig,
            test_run_host::drasi_servers::TestRunDrasiServerState,
            // Drasi Server Component schemas
            test_run_host::api_models::ComponentStatus,
            test_run_host::api_models::StatusResponse,
            test_run_host::api_models::DrasiServerError,
            // Drasi Server Source schemas
            test_run_host::api_models::SourceInfo,
            test_run_host::api_models::SourceDetails,
            test_run_host::api_models::CreateSourceRequest,
            test_run_host::api_models::UpdateSourceRequest,
            test_run_host::api_models::SourceCreatedResponse,
            // Drasi Server Query schemas
            test_run_host::api_models::QueryInfo,
            test_run_host::api_models::QueryDetails,
            test_run_host::api_models::CreateQueryRequest,
            test_run_host::api_models::UpdateQueryRequest,
            test_run_host::api_models::QueryCreatedResponse,
            // Drasi Server Reaction schemas
            test_run_host::api_models::ReactionInfo,
            test_run_host::api_models::ReactionDetails,
            test_run_host::api_models::CreateReactionRequest,
            test_run_host::api_models::UpdateReactionRequest,
            test_run_host::api_models::ReactionCreatedResponse,
            // Test Run schemas
            test_runs::TestRunCreatedResponse,
            test_runs::TestRunInfo,
        )
    ),
    tags(
        (name = "service", description = "Test Service general information"),
        (name = "test-runs", description = "Test Run management API - hierarchical structure for organizing test components"),
        (name = "drasi-servers", description = "Drasi Server management API"),
        (name = "drasi-server-sources", description = "Drasi Server Sources management API"),
        (name = "drasi-server-queries", description = "Drasi Server Queries management API"),
        (name = "drasi-server-reactions", description = "Drasi Server Reactions management API"),
        (name = "sources", description = "Test source management API"),
        (name = "queries", description = "Test query management API"),
        (name = "reactions", description = "Test reaction management API"),
        (name = "repos", description = "Test repository management API")
    ),
    info(
        title = "Drasi Test Service API",
        version = "0.1.0",
        description = "REST API for controlling the Drasi Test Service and managing test resources",
        contact(
            name = "Drasi Team",
            url = "https://github.com/drasi-project"
        )
    )
)]
pub struct ApiDoc;
