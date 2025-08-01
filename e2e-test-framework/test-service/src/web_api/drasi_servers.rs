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

use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use test_data_store::test_run_storage::TestRunDrasiServerId;
use test_run_host::{
    drasi_servers::{TestRunDrasiServerConfig, TestRunDrasiServerState},
    TestRunHost,
};
use utoipa::ToSchema;

pub use test_run_host::drasi_servers::api_models::DrasiServerError;

/// Create a new Drasi Server for a test run
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers",
    request_body = TestRunDrasiServerConfig,
    responses(
        (status = 200, description = "Drasi Server created successfully", body = DrasiServerCreatedResponse),
        (status = 400, description = "Invalid configuration"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-servers"
)]
pub async fn create_drasi_server(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Json(config): Json<TestRunDrasiServerConfig>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match test_run_host.add_test_drasi_server(config).await {
        Ok(id) => Ok(Json(DrasiServerCreatedResponse { id: id.to_string() })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// List all Drasi Servers
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers",
    responses(
        (status = 200, description = "List of Drasi Servers", body = Vec<DrasiServerInfo>),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-servers"
)]
pub async fn list_drasi_servers(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    match test_run_host.get_test_drasi_server_ids().await {
        Ok(ids) => {
            let mut servers = Vec::new();
            for id_str in ids {
                if let Ok(server_id) = TestRunDrasiServerId::try_from(id_str.as_str()) {
                    if let Ok(Some(state)) = test_run_host.get_test_drasi_server(&server_id).await {
                        let endpoint = test_run_host
                            .get_drasi_server_endpoint(&server_id)
                            .await
                            .unwrap_or(None);

                        servers.push(DrasiServerInfo {
                            id: id_str,
                            state,
                            endpoint,
                        });
                    }
                }
            }
            Ok(Json(servers))
        }
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// Get details of a specific Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{id}",
    params(
        ("id" = String, Path, description = "Drasi Server ID")
    ),
    responses(
        (status = 200, description = "Drasi Server details", body = DrasiServerDetails),
        (status = 404, description = "Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-servers"
)]
pub async fn get_drasi_server(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let server_id = match TestRunDrasiServerId::try_from(id.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    match test_run_host.get_test_drasi_server(&server_id).await {
        Ok(Some(state)) => {
            let endpoint = test_run_host
                .get_drasi_server_endpoint(&server_id)
                .await
                .unwrap_or(None);

            Ok(Json(DrasiServerDetails {
                id: server_id.to_string(),
                state,
                endpoint,
            }))
        }
        Ok(None) => Err((StatusCode::NOT_FOUND, "Drasi Server not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

/// Stop and remove a Drasi Server
#[utoipa::path(
    delete,
    path = "/test_run_host/drasi_servers/{id}",
    params(
        ("id" = String, Path, description = "Drasi Server ID")
    ),
    responses(
        (status = 204, description = "Drasi Server removed successfully"),
        (status = 404, description = "Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-servers"
)]
pub async fn delete_drasi_server(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let server_id = match TestRunDrasiServerId::try_from(id.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    match test_run_host.remove_test_drasi_server(&server_id).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((StatusCode::NOT_FOUND, e.to_string()))
            } else {
                Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            }
        }
    }
}

/// Get the status of a specific Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{id}/status",
    params(
        ("id" = String, Path, description = "Drasi Server ID")
    ),
    responses(
        (status = 200, description = "Drasi Server status", body = DrasiServerStatus),
        (status = 404, description = "Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-servers"
)]
pub async fn get_drasi_server_status(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let server_id = match TestRunDrasiServerId::try_from(id.as_str()) {
        Ok(id) => id,
        Err(e) => return Err((StatusCode::BAD_REQUEST, e.to_string())),
    };

    match test_run_host.get_test_drasi_server(&server_id).await {
        Ok(Some(state)) => Ok(Json(DrasiServerStatus {
            id: server_id.to_string(),
            state,
        })),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Drasi Server not found".to_string())),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DrasiServerCreatedResponse {
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DrasiServerInfo {
    pub id: String,
    pub state: TestRunDrasiServerState,
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DrasiServerDetails {
    pub id: String,
    pub state: TestRunDrasiServerState,
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DrasiServerStatus {
    pub id: String,
    pub state: TestRunDrasiServerState,
}

pub fn get_drasi_servers_routes() -> axum::Router {
    use axum::routing::{delete, get, post};

    axum::Router::new()
        .route("/drasi_servers", post(create_drasi_server))
        .route("/drasi_servers", get(list_drasi_servers))
        .route("/drasi_servers/:id", get(get_drasi_server))
        .route("/drasi_servers/:id", delete(delete_drasi_server))
        .route("/drasi_servers/:id/status", get(get_drasi_server_status))
}
