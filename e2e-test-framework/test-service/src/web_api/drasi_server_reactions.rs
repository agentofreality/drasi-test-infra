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
use test_run_host::drasi_servers::api_models::{
    CreateReactionRequest, UpdateReactionRequest,
};

#[allow(dead_code)]
#[derive(OpenApi)]
#[openapi(paths(list_reactions, get_reaction, create_reaction, update_reaction, delete_reaction, start_reaction, stop_reaction))]
pub struct DrasiServerReactionsApi;

/// List all reactions on a Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/reactions",
    responses(
        (status = 200, description = "List of reactions", body = Vec<ReactionInfo>),
        (status = 404, description = "Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn list_reactions(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(server_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host.list_drasi_server_reactions(&server_id).await {
        Ok(reactions) => Ok(Json(reactions)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(DrasiServerError {
                    error: "ListReactionsFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: None,
                }),
            ))
        }
    }
}

/// Get details of a specific reaction
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/reactions/{reaction_id}",
    responses(
        (status = 200, description = "Reaction details", body = ReactionDetails),
        (status = 404, description = "Reaction or server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn get_reaction(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, reaction_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .get_drasi_server_reaction(&server_id, &reaction_id)
        .await
    {
        Ok(reaction) => Ok(Json(reaction)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(DrasiServerError {
                    error: "GetReactionFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(reaction_id),
                }),
            ))
        }
    }
}

/// Create a new reaction
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/reactions",
    request_body = CreateReactionRequest,
    responses(
        (status = 201, description = "Reaction created successfully", body = ReactionCreatedResponse),
        (status = 400, description = "Invalid configuration"),
        (status = 404, description = "Drasi Server not found"),
        (status = 409, description = "Reaction already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn create_reaction(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(server_id): Path<String>,
    Json(request): Json<CreateReactionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    let reaction_name = request.name.clone();
    match test_run_host
        .create_drasi_server_reaction(&server_id, request)
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
                    error: "CreateReactionFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(reaction_name),
                }),
            ))
        }
    }
}

/// Update an existing reaction
#[utoipa::path(
    put,
    path = "/test_run_host/drasi_servers/{server_id}/reactions/{reaction_id}",
    request_body = UpdateReactionRequest,
    responses(
        (status = 200, description = "Reaction updated successfully", body = ReactionDetails),
        (status = 400, description = "Invalid configuration"),
        (status = 404, description = "Reaction or server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn update_reaction(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, reaction_id)): Path<(String, String)>,
    Json(request): Json<UpdateReactionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .update_drasi_server_reaction(&server_id, &reaction_id, request)
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
                    error: "UpdateReactionFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(reaction_id),
                }),
            ))
        }
    }
}

/// Delete a reaction
#[utoipa::path(
    delete,
    path = "/test_run_host/drasi_servers/{server_id}/reactions/{reaction_id}",
    responses(
        (status = 204, description = "Reaction deleted successfully"),
        (status = 404, description = "Reaction or server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn delete_reaction(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, reaction_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .delete_drasi_server_reaction(&server_id, &reaction_id)
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
                    error: "DeleteReactionFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(reaction_id),
                }),
            ))
        }
    }
}

/// Start a reaction
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/reactions/{reaction_id}/start",
    responses(
        (status = 200, description = "Reaction started successfully", body = StatusResponse),
        (status = 404, description = "Reaction or server not found"),
        (status = 409, description = "Reaction already running"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn start_reaction(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, reaction_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .start_drasi_server_reaction(&server_id, &reaction_id)
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
                    error: "StartReactionFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(reaction_id),
                }),
            ))
        }
    }
}

/// Stop a reaction
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/reactions/{reaction_id}/stop",
    responses(
        (status = 200, description = "Reaction stopped successfully", body = StatusResponse),
        (status = 404, description = "Reaction or server not found"),
        (status = 409, description = "Reaction not running"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-reactions"
)]
pub async fn stop_reaction(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, reaction_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .stop_drasi_server_reaction(&server_id, &reaction_id)
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
                    error: "StopReactionFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(reaction_id),
                }),
            ))
        }
    }
}

pub fn get_drasi_server_reactions_routes() -> axum::Router {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/drasi_servers/:server_id/reactions", get(list_reactions))
        .route("/drasi_servers/:server_id/reactions", post(create_reaction))
        .route(
            "/drasi_servers/:server_id/reactions/:reaction_id",
            get(get_reaction),
        )
        .route(
            "/drasi_servers/:server_id/reactions/:reaction_id",
            put(update_reaction),
        )
        .route(
            "/drasi_servers/:server_id/reactions/:reaction_id",
            delete(delete_reaction),
        )
        .route(
            "/drasi_servers/:server_id/reactions/:reaction_id/start",
            post(start_reaction),
        )
        .route(
            "/drasi_servers/:server_id/reactions/:reaction_id/stop",
            post(stop_reaction),
        )
}
