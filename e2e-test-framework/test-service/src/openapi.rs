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
    queries, repo, sources, DataCollectorStateResponse, TestDataStoreStateResponse,
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
            // Repository schemas
            repo::TestRepoResponse,
            repo::TestPostBody,
            repo::TestResponse,
            repo::TestSourcePostBody,
            repo::TestSourceResponse,
        )
    ),
    tags(
        (name = "service", description = "Test Service general information"),
        (name = "sources", description = "Test source management API"),
        (name = "queries", description = "Test query management API"),
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
