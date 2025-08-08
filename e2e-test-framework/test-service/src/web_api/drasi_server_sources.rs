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
use test_run_host::TestRunHost;
use utoipa::OpenApi;

use crate::web_api::drasi_servers::DrasiServerError;
use test_run_host::drasi_servers::api_models::{CreateSourceRequest, UpdateSourceRequest};

#[allow(dead_code)]
#[derive(OpenApi)]
#[openapi(paths(
    list_sources,
    get_source,
    create_source,
    update_source,
    delete_source,
    start_source,
    stop_source
))]
pub struct DrasiServerSourcesApi;

/// List all sources on a Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/sources",
    responses(
        (status = 200, description = "List of sources", body = Vec<SourceInfo>),
        (status = 404, description = "Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn list_sources(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(server_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host.list_drasi_server_sources(&server_id).await {
        Ok(sources) => Ok(Json(sources)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "ListSourcesFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: None,
                }),
            ))
        }
    }
}

/// Get details of a specific source
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/sources/{source_id}",
    responses(
        (status = 200, description = "Source details", body = SourceDetails),
        (status = 404, description = "Source or server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn get_source(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, source_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .get_drasi_server_source(&server_id, &source_id)
        .await
    {
        Ok(source) => Ok(Json(source)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "GetSourceFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(source_id),
                }),
            ))
        }
    }
}

/// Create a new source
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/sources",
    request_body = CreateSourceRequest,
    responses(
        (status = 201, description = "Source created successfully", body = SourceCreatedResponse),
        (status = 400, description = "Invalid configuration"),
        (status = 404, description = "Drasi Server not found"),
        (status = 409, description = "Source already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn create_source(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(server_id): Path<String>,
    Json(request): Json<CreateSourceRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    let source_name = request.name.clone();
    match test_run_host
        .create_drasi_server_source(&server_id, request)
        .await
    {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else if e.to_string().contains("invalid") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "CreateSourceFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(source_name),
                }),
            ))
        }
    }
}

/// Update an existing source
#[utoipa::path(
    put,
    path = "/test_run_host/drasi_servers/{server_id}/sources/{source_id}",
    request_body = UpdateSourceRequest,
    responses(
        (status = 200, description = "Source updated successfully", body = SourceDetails),
        (status = 400, description = "Invalid configuration"),
        (status = 404, description = "Source or server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn update_source(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, source_id)): Path<(String, String)>,
    Json(request): Json<UpdateSourceRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .update_drasi_server_source(&server_id, &source_id, request)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("invalid") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "UpdateSourceFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(source_id),
                }),
            ))
        }
    }
}

/// Delete a source
#[utoipa::path(
    delete,
    path = "/test_run_host/drasi_servers/{server_id}/sources/{source_id}",
    responses(
        (status = 204, description = "Source deleted successfully"),
        (status = 404, description = "Source or server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn delete_source(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, source_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .delete_drasi_server_source(&server_id, &source_id)
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "DeleteSourceFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(source_id),
                }),
            ))
        }
    }
}

/// Start a source
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/sources/{source_id}/start",
    responses(
        (status = 200, description = "Source started successfully", body = StatusResponse),
        (status = 404, description = "Source or server not found"),
        (status = 409, description = "Source already running"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn start_source(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, source_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .start_drasi_server_source(&server_id, &source_id)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("already running") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "StartSourceFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(source_id),
                }),
            ))
        }
    }
}

/// Stop a source
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/sources/{source_id}/stop",
    responses(
        (status = 200, description = "Source stopped successfully", body = StatusResponse),
        (status = 404, description = "Source or server not found"),
        (status = 409, description = "Source not running"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-sources"
)]
pub async fn stop_source(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, source_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .stop_drasi_server_source(&server_id, &source_id)
        .await
    {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("not running") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "StopSourceFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(source_id),
                }),
            ))
        }
    }
}

pub fn get_drasi_server_sources_routes() -> axum::Router {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/drasi_servers/:server_id/sources", get(list_sources))
        .route("/drasi_servers/:server_id/sources", post(create_source))
        .route(
            "/drasi_servers/:server_id/sources/:source_id",
            get(get_source),
        )
        .route(
            "/drasi_servers/:server_id/sources/:source_id",
            put(update_source),
        )
        .route(
            "/drasi_servers/:server_id/sources/:source_id",
            delete(delete_source),
        )
        .route(
            "/drasi_servers/:server_id/sources/:source_id/start",
            post(start_source),
        )
        .route(
            "/drasi_servers/:server_id/sources/:source_id/stop",
            post(stop_source),
        )
}
