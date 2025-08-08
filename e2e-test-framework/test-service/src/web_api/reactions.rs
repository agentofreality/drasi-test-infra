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
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use test_run_host::{reactions::TestRunReactionConfig, TestRunHost, TestRunHostStatus};

use super::TestServiceWebApiError;

pub fn get_reactions_routes() -> Router {
    Router::new()
        .route(
            "/reactions",
            get(get_reaction_list_handler).post(post_reaction_handler),
        )
        .route("/reactions/:id", get(get_reaction_handler))
        .route(
            "/reactions/:id/pause",
            post(reaction_observer_pause_handler),
        )
        .route(
            "/reactions/:id/reset",
            post(reaction_observer_reset_handler),
        )
        .route(
            "/reactions/:id/start",
            post(reaction_observer_start_handler),
        )
        .route("/reactions/:id/stop", post(reaction_observer_stop_handler))
}

#[utoipa::path(
    get,
    path = "/test_run_host/reactions",
    tag = "reactions",
    responses(
        (status = 200, description = "List of reaction IDs", body = Vec<String>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_reaction_list_handler(
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - get_reaction_list");

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let reactions = test_run_host.get_test_reaction_ids().await?;
    Ok(Json(reactions).into_response())
}

#[utoipa::path(
    get,
    path = "/test_run_host/reactions/{id}",
    tag = "reactions",
    params(
        ("id" = String, Path, description = "Reaction identifier")
    ),
    responses(
        (status = 200, description = "Reaction state information", body = ReactionStateResponse),
        (status = 404, description = "Reaction not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_reaction_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - get_reaction: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let reaction_state = test_run_host.get_test_reaction_state(&id).await?;
    Ok(Json(reaction_state).into_response())
}

#[utoipa::path(
    post,
    path = "/test_run_host/reactions/{id}/pause",
    tag = "reactions",
    params(
        ("id" = String, Path, description = "Reaction identifier")
    ),
    responses(
        (status = 200, description = "Reaction paused successfully", body = ReactionObserverState),
        (status = 404, description = "Reaction not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn reaction_observer_pause_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - reaction_observer_pause: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_reaction_pause(&id).await;
    match response {
        Ok(reaction) => Ok(Json(reaction.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/reactions/{id}/reset",
    tag = "reactions",
    params(
        ("id" = String, Path, description = "Reaction identifier")
    ),
    responses(
        (status = 200, description = "Reaction reset successfully", body = ReactionObserverState),
        (status = 404, description = "Reaction not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn reaction_observer_reset_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - reaction_observer_reset: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_reaction_reset(&id).await;
    match response {
        Ok(reaction) => Ok(Json(reaction.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/reactions/{id}/start",
    tag = "reactions",
    params(
        ("id" = String, Path, description = "Reaction identifier")
    ),
    responses(
        (status = 200, description = "Reaction started successfully", body = ReactionObserverState),
        (status = 404, description = "Reaction not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn reaction_observer_start_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - reaction_observer_start: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_reaction_start(&id).await;
    match response {
        Ok(reaction) => Ok(Json(reaction.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/reactions/{id}/stop",
    tag = "reactions",
    params(
        ("id" = String, Path, description = "Reaction identifier")
    ),
    responses(
        (status = 200, description = "Reaction stopped successfully", body = ReactionObserverState),
        (status = 404, description = "Reaction not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn reaction_observer_stop_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - reaction_observer_stop: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_reaction_stop(&id).await;
    match response {
        Ok(reaction) => Ok(Json(reaction.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/reactions",
    tag = "reactions",
    request_body = test_run_host::reactions::TestRunReactionConfig,
    responses(
        (status = 200, description = "Reaction created successfully", body = ReactionStateResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn post_reaction_handler(
    test_run_host: Extension<Arc<TestRunHost>>,
    body: Json<TestRunReactionConfig>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - post_reaction");

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let reaction_config = body.0;

    // Extract TestRunId from the config
    let test_run_id = match test_data_store::test_run_storage::TestRunId::try_from(&reaction_config)
    {
        Ok(id) => id,
        Err(e) => return Err(TestServiceWebApiError::AnyhowError(anyhow::anyhow!(e))),
    };

    match test_run_host
        .add_test_reaction(&test_run_id, reaction_config)
        .await
    {
        Ok(id) => match test_run_host.get_test_reaction_state(&id.to_string()).await {
            Ok(reaction) => Ok(Json(reaction).into_response()),
            Err(_) => Err(TestServiceWebApiError::NotFound(
                "TestRunReaction".to_string(),
                id.to_string(),
            )),
        },
        Err(e) => {
            let msg = format!("Error creating Reaction: {}", e);
            log::error!("{}", &msg);
            Err(TestServiceWebApiError::AnyhowError(e))
        }
    }
}
