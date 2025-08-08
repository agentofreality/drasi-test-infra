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
use test_run_host::drasi_servers::api_models::{CreateQueryRequest, UpdateQueryRequest};

#[allow(dead_code)]
#[derive(OpenApi)]
#[openapi(paths(
    list_queries,
    get_query,
    create_query,
    update_query,
    delete_query,
    start_query,
    stop_query,
    get_query_results
))]
pub struct DrasiServerQueriesApi;

/// List queries on a Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/queries",
    responses(
        (status = 200, description = "List of queries", body = Vec<QueryInfo>),
        (status = 404, description = "Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn list_queries(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(server_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host.list_drasi_server_queries(&server_id).await {
        Ok(queries) => Ok(Json(queries)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "ListQueriesFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: None,
                }),
            ))
        }
    }
}

/// Get a query on a Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/queries/{query_id}",
    responses(
        (status = 200, description = "Query details", body = QueryDetails),
        (status = 404, description = "Query or Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn get_query(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, query_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .get_drasi_server_query(&server_id, &query_id)
        .await
    {
        Ok(query) => Ok(Json(query)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "GetQueryFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(query_id),
                }),
            ))
        }
    }
}

/// Create a query on a Drasi Server
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/queries",
    request_body = CreateQueryRequest,
    responses(
        (status = 201, description = "Query created", body = QueryCreatedResponse),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Drasi Server not found"),
        (status = 409, description = "Query already exists"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn create_query(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path(server_id): Path<String>,
    Json(request): Json<CreateQueryRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .create_drasi_server_query(&server_id, request)
        .await
    {
        Ok(response) => Ok((StatusCode::CREATED, Json(response))),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else if e.to_string().contains("already exists") {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "CreateQueryFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: None,
                }),
            ))
        }
    }
}

/// Update a query on a Drasi Server
#[utoipa::path(
    put,
    path = "/test_run_host/drasi_servers/{server_id}/queries/{query_id}",
    request_body = UpdateQueryRequest,
    responses(
        (status = 200, description = "Query updated", body = QueryDetails),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Query or Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn update_query(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, query_id)): Path<(String, String)>,
    Json(request): Json<UpdateQueryRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .update_drasi_server_query(&server_id, &query_id, request)
        .await
    {
        Ok(query) => Ok(Json(query)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "UpdateQueryFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(query_id),
                }),
            ))
        }
    }
}

/// Delete a query from a Drasi Server
#[utoipa::path(
    delete,
    path = "/test_run_host/drasi_servers/{server_id}/queries/{query_id}",
    responses(
        (status = 204, description = "Query deleted"),
        (status = 404, description = "Query or Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn delete_query(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, query_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .delete_drasi_server_query(&server_id, &query_id)
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
                    error: "DeleteQueryFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(query_id),
                }),
            ))
        }
    }
}

/// Start a query on a Drasi Server
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/queries/{query_id}/start",
    responses(
        (status = 200, description = "Query started", body = StatusResponse),
        (status = 404, description = "Query or Drasi Server not found"),
        (status = 409, description = "Query already running"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn start_query(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, query_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .start_drasi_server_query(&server_id, &query_id)
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
                    error: "StartQueryFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(query_id),
                }),
            ))
        }
    }
}

/// Stop a query on a Drasi Server
#[utoipa::path(
    post,
    path = "/test_run_host/drasi_servers/{server_id}/queries/{query_id}/stop",
    responses(
        (status = 200, description = "Query stopped", body = StatusResponse),
        (status = 404, description = "Query or Drasi Server not found"),
        (status = 409, description = "Query not running"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn stop_query(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, query_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .stop_drasi_server_query(&server_id, &query_id)
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
                    error: "StopQueryFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(query_id),
                }),
            ))
        }
    }
}

/// Get query results from a Drasi Server
#[utoipa::path(
    get,
    path = "/test_run_host/drasi_servers/{server_id}/queries/{query_id}/results",
    responses(
        (status = 200, description = "Query results", body = serde_json::Value),
        (status = 404, description = "Query or Drasi Server not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "drasi-server-queries"
)]
pub async fn get_query_results(
    Extension(test_run_host): Extension<Arc<TestRunHost>>,
    Path((server_id, query_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, (StatusCode, Json<DrasiServerError>)> {
    match test_run_host
        .get_drasi_server_query_results(&server_id, &query_id)
        .await
    {
        Ok(results) => Ok(Json(results)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            Err((
                status,
                Json(DrasiServerError {
                    error: "GetQueryResultsFailed".to_string(),
                    message: e.to_string(),
                    server_id: server_id.clone(),
                    component_id: Some(query_id),
                }),
            ))
        }
    }
}

pub fn get_drasi_server_queries_routes() -> axum::Router {
    use axum::routing::{delete, get, post, put};

    axum::Router::new()
        .route("/drasi_servers/:server_id/queries", get(list_queries))
        .route("/drasi_servers/:server_id/queries", post(create_query))
        .route(
            "/drasi_servers/:server_id/queries/:query_id",
            get(get_query),
        )
        .route(
            "/drasi_servers/:server_id/queries/:query_id",
            put(update_query),
        )
        .route(
            "/drasi_servers/:server_id/queries/:query_id",
            delete(delete_query),
        )
        .route(
            "/drasi_servers/:server_id/queries/:query_id/start",
            post(start_query),
        )
        .route(
            "/drasi_servers/:server_id/queries/:query_id/stop",
            post(stop_query),
        )
        .route(
            "/drasi_servers/:server_id/queries/:query_id/results",
            get(get_query_results),
        )
}
