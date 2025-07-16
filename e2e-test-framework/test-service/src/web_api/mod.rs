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

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use thiserror::Error;
use tokio::{select, signal};
use utoipa::{OpenApi, ToSchema};

use data_collector::DataCollector;
use queries::get_queries_routes;
use repo::get_test_repo_routes;
use sources::get_sources_routes;
use test_data_store::TestDataStore;
use test_run_host::TestRunHost;
use utoipa_swagger_ui::SwaggerUi;

use crate::openapi::ApiDoc;

pub mod queries;
pub mod repo;
pub mod sources;

#[derive(Debug, Error)]
pub enum TestServiceWebApiError {
    #[error("Anyhow Error: {0}")]
    AnyhowError(anyhow::Error),
    #[error("{0} with ID {1} not found")]
    NotFound(String, String),
    #[error("Serde Error: {0}")]
    SerdeJsonError(serde_json::Error),
    #[error("TestRunHost is in an Error state: {0}")]
    TestRunHostError(String),
    #[error("NotReady: {0}")]
    NotReady(String),
    #[error("IO Error: {0}")]
    IOError(std::io::Error),
}

impl From<anyhow::Error> for TestServiceWebApiError {
    fn from(error: anyhow::Error) -> Self {
        TestServiceWebApiError::AnyhowError(error)
    }
}

impl From<serde_json::Error> for TestServiceWebApiError {
    fn from(error: serde_json::Error) -> Self {
        TestServiceWebApiError::SerdeJsonError(error)
    }
}

impl From<std::io::Error> for TestServiceWebApiError {
    fn from(error: std::io::Error) -> Self {
        TestServiceWebApiError::IOError(error)
    }
}

impl IntoResponse for TestServiceWebApiError {
    fn into_response(self) -> Response {
        match self {
            TestServiceWebApiError::AnyhowError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response()
            }
            TestServiceWebApiError::NotFound(kind, id) => (
                StatusCode::NOT_FOUND,
                Json(format!("{} with ID {} not found", kind, id)),
            )
                .into_response(),
            TestServiceWebApiError::SerdeJsonError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response()
            }
            TestServiceWebApiError::TestRunHostError(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(msg)).into_response()
            }
            TestServiceWebApiError::NotReady(msg) => {
                (StatusCode::SERVICE_UNAVAILABLE, Json(msg)).into_response()
            }
            TestServiceWebApiError::IOError(e) => {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_string())).into_response()
            }
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "data_collector": {
        "status": "Running",
        "data_collection_ids": ["collection-1", "collection-2"]
    },
    "data_store": {
        "path": "/data/test-data-store",
        "test_repo_ids": ["repo-1", "repo-2"]
    },
    "test_run_host": {
        "status": "Active",
        "test_run_query_ids": ["query-1", "query-2"],
        "test_run_source_ids": ["source-1", "source-2"]
    }
}))]
pub struct TestServiceStateResponse {
    /// Data collector state information
    pub data_collector: DataCollectorStateResponse,
    /// Test data store state information
    pub data_store: TestDataStoreStateResponse,
    /// Test run host state information
    pub test_run_host: TestRunHostStateResponse,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "path": "/data/test-data-store",
    "test_repo_ids": ["population-test", "performance-test"]
}))]
pub struct TestDataStoreStateResponse {
    /// File system path where test data is stored
    pub path: String,
    /// List of available test repository IDs
    pub test_repo_ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "status": "Active",
    "test_run_query_ids": ["query-1", "query-2"],
    "test_run_source_ids": ["source-1", "source-2"]
}))]
pub struct TestRunHostStateResponse {
    /// Current status of the test run host
    pub status: String,
    /// List of active test run query IDs
    pub test_run_query_ids: Vec<String>,
    /// List of active test run source IDs
    pub test_run_source_ids: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
#[schema(example = json!({
    "status": "Running",
    "data_collection_ids": ["collection-123", "collection-456"]
}))]
pub struct DataCollectorStateResponse {
    /// Current status of the data collector
    pub status: String,
    /// List of active data collection IDs
    pub data_collection_ids: Vec<String>,
}

pub(crate) async fn start_web_api(
    port: u16,
    test_data_store: Arc<TestDataStore>,
    test_run_host: Arc<TestRunHost>,
    data_collector: Arc<DataCollector>,
) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    // Create the main API router
    let api_router = Router::new()
        .route("/", get(get_service_info_handler))
        .nest("/test_repos", get_test_repo_routes())
        .nest("/test_run_host", get_queries_routes())
        .nest("/test_run_host", get_sources_routes());

    // Create the complete application with Swagger UI
    let app = api_router
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(axum::extract::Extension(data_collector))
        .layer(axum::extract::Extension(test_data_store))
        .layer(axum::extract::Extension(test_run_host));

    log::info!("Test Service Web API listening on http://{}", addr);
    log::info!("API Documentation available at http://{}/docs", addr);
    log::info!("OpenAPI JSON specification available at http://{}/api-docs/openapi.json", addr);

    let server = axum::Server::bind(&addr).serve(app.into_make_service());

    // Graceful shutdown when receiving `Ctrl+C` or SIGTERM
    let graceful = server.with_graceful_shutdown(shutdown_signal());

    log::info!("Press CTRL-C to stop the server...");

    if let Err(err) = graceful.await {
        eprintln!("Server error: {}", err);
    }
}

async fn shutdown_signal() {
    // Create two tasks: one for Ctrl+C and another for SIGTERM
    // Either will trigger a shutdown
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        log::info!("Received Ctrl+C, shutting down...");
    };

    // Listen for SIGTERM (Docker stop signal)
    let sigterm = async {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to listen for SIGTERM");
            sigterm.recv().await;
            log::info!("Received SIGTERM, shutting down...");
        }
        #[cfg(not(unix))]
        futures::future::pending::<()>().await; // Fallback for non-Unix systems
    };

    // Wait for either signal
    select! {
        _ = ctrl_c => {},
        _ = sigterm => {},
    }

    log::info!("Cleaning up resources...");
    // TODO: Perform cleanup here...
    log::info!("Resources cleaned up.");
}

#[utoipa::path(
    get,
    path = "/",
    tag = "service",
    responses(
        (status = 200, description = "Service information retrieved successfully", body = TestServiceStateResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
async fn get_service_info_handler(
    data_collector: Extension<Arc<DataCollector>>,
    test_data_store: Extension<Arc<TestDataStore>>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - service_info");

    Ok(Json(TestServiceStateResponse {
        data_store: TestDataStoreStateResponse {
            path: test_data_store
                .get_data_store_path()
                .await?
                .to_string_lossy()
                .to_string(),
            test_repo_ids: test_data_store.get_test_repo_ids().await?,
        },
        test_run_host: TestRunHostStateResponse {
            status: test_run_host.get_status().await?.to_string(),
            test_run_query_ids: test_run_host.get_test_query_ids().await?,
            test_run_source_ids: test_run_host.get_test_source_ids().await?,
        },
        data_collector: DataCollectorStateResponse {
            status: data_collector.get_status().await?.to_string(),
            data_collection_ids: data_collector.get_data_collection_ids().await?,
        },
    }))
}
