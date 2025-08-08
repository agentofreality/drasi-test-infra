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

use std::{collections::HashSet, sync::Arc};

use axum::{
    extract::{Extension, Path},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};
use test_data_store::{
    scripts::{NodeRecord, RelationRecord},
    test_repo_storage::models::SpacingMode,
};
use test_run_host::{
    sources::{bootstrap_data_generators::BootstrapData, TestRunSourceConfig},
    TestRunHost, TestRunHostStatus,
};
use utoipa::ToSchema;

use super::TestServiceWebApiError;

pub fn get_sources_routes() -> Router {
    Router::new()
        .route(
            "/sources",
            get(get_source_list_handler).post(post_source_handler),
        )
        .route("/sources/:id", get(get_source_handler))
        .route("/sources/:id/bootstrap", post(source_bootstrap_handler))
        .route(
            "/sources/:id/pause",
            post(source_change_generator_pause_handler),
        )
        .route(
            "/sources/:id/reset",
            post(source_change_generator_reset_handler),
        )
        .route(
            "/sources/:id/skip",
            post(source_change_generator_skip_handler),
        )
        .route(
            "/sources/:id/start",
            post(source_change_generator_start_handler),
        )
        .route(
            "/sources/:id/step",
            post(source_change_generator_step_handler),
        )
        .route(
            "/sources/:id/stop",
            post(source_change_generator_stop_handler),
        )
}

#[utoipa::path(
    get,
    path = "/test_run_host/sources",
    tag = "sources",
    responses(
        (status = 200, description = "List of source IDs", body = Vec<String>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_source_list_handler(
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - get_source_list");

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let sources = test_run_host.get_test_source_ids().await?;
    Ok(Json(sources).into_response())
}

#[utoipa::path(
    get,
    path = "/test_run_host/sources/{id}",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    responses(
        (status = 200, description = "Source state information", body = SourceStateResponse),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_source_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - get_source: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let source_state = test_run_host.get_test_source_state(&id).await?;
    Ok(Json(source_state).into_response())
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "num_skips": 10,
    "spacing_mode": "Recorded"
}))]
pub struct TestSkipConfig {
    /// Number of events to skip
    #[serde(default)]
    pub num_skips: u64,
    /// Spacing mode for skipping events
    pub spacing_mode: Option<SpacingMode>,
}

impl Default for TestSkipConfig {
    fn default() -> Self {
        TestSkipConfig {
            num_skips: 1,
            spacing_mode: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "nodeLabels": ["Person", "Company"],
    "relLabels": ["WORKS_FOR", "KNOWS"]
}))]
pub struct SourceBootstrapRequestBody {
    /// Labels of nodes to include in bootstrap data
    #[serde(rename = "nodeLabels")]
    pub node_labels: Vec<String>,
    /// Labels of relationships to include in bootstrap data
    #[serde(rename = "relLabels")]
    pub rel_labels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SourceBootstrapResponseBody {
    pub nodes: Vec<Node>,
    pub rels: Vec<Relation>,
}

impl SourceBootstrapResponseBody {
    pub fn new(data: BootstrapData) -> Self {
        let mut body = Self {
            nodes: Vec::new(),
            rels: Vec::new(),
        };

        for (_, nodes) in data.nodes {
            body.nodes
                .extend(nodes.iter().map(Node::from_script_record));
        }
        for (_, rels) in data.rels {
            body.rels
                .extend(rels.iter().map(Relation::from_script_record));
        }

        body
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "person-123",
    "labels": ["Person", "Employee"],
    "properties": {"name": "John Doe", "age": 30, "department": "Engineering"}
}))]
pub struct Node {
    /// Unique identifier for the node
    #[serde(default)]
    pub id: String,
    /// Labels associated with the node
    #[serde(default)]
    pub labels: Vec<String>,
    /// Properties of the node as a JSON object
    #[serde(serialize_with = "serialize_properties")]
    pub properties: Value,
}

impl Node {
    fn from_script_record(record: &NodeRecord) -> Self {
        Self {
            id: record.id.clone(),
            labels: record.labels.clone(),
            properties: record.properties.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "rel-456",
    "labels": ["REPORTS_TO"],
    "startId": "person-123",
    "startLabel": "Person",
    "endId": "person-789",
    "endLabel": "Person",
    "properties": {"since": "2023-01-01", "direct": true}
}))]
pub struct Relation {
    /// Unique identifier for the relationship
    #[serde(default)]
    pub id: String,
    /// Labels/types of the relationship
    #[serde(default)]
    pub labels: Vec<String>,
    /// ID of the start node
    #[serde(default, rename = "startId")]
    pub start_id: String,
    /// Optional label of the start node
    #[serde(skip_serializing_if = "Option::is_none", rename = "startLabel")]
    pub start_label: Option<String>,
    /// ID of the end node
    #[serde(default, rename = "endId")]
    pub end_id: String,
    /// Optional label of the end node
    #[serde(skip_serializing_if = "Option::is_none", rename = "endLabel")]
    pub end_label: Option<String>,
    /// Properties of the relationship as a JSON object
    #[serde(serialize_with = "serialize_properties")]
    pub properties: Value,
}

impl Relation {
    fn from_script_record(record: &RelationRecord) -> Self {
        Self {
            id: record.id.clone(),
            labels: record.labels.clone(),
            start_id: record.start_id.clone(),
            start_label: record.start_label.clone(),
            end_id: record.end_id.clone(),
            end_label: record.end_label.clone(),
            properties: record.properties.clone(),
        }
    }
}

fn serialize_properties<S>(value: &serde_json::Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        // If properties is Null, serialize it as an empty object `{}`.
        Value::Null => {
            let empty_obj = json!({});
            empty_obj.serialize(serializer)
        }
        // Otherwise, serialize the value as-is.
        _ => value.serialize(serializer),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/bootstrap",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    request_body = SourceBootstrapRequestBody,
    responses(
        (status = 200, description = "Bootstrap data retrieved successfully", body = SourceBootstrapResponseBody),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_bootstrap_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
    body: Json<SourceBootstrapRequestBody>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_bootstrap");

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let bootstrap_body = body.0;

    let node_labels: HashSet<String> = bootstrap_body.node_labels.into_iter().collect();
    let rel_labels: HashSet<String> = bootstrap_body.rel_labels.into_iter().collect();
    log::debug!(
        "Source: {:?}, Node Labels: {:?}, Rel Labels: {:?}",
        id,
        node_labels,
        rel_labels
    );

    let response = test_run_host
        .get_source_bootstrap_data(&id, &node_labels, &rel_labels)
        .await;
    match response {
        Ok(data) => Ok(Json(SourceBootstrapResponseBody::new(data)).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/pause",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    responses(
        (status = 200, description = "Source paused successfully", body = SourceChangeGeneratorState),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_change_generator_pause_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_change_generator_pause: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_source_pause(&id).await;
    match response {
        Ok(source) => Ok(Json(source.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/reset",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    responses(
        (status = 200, description = "Source reset successfully", body = SourceChangeGeneratorState),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_change_generator_reset_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_change_generator_reset: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_source_reset(&id).await;
    match response {
        Ok(source) => Ok(Json(source.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/skip",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    request_body = Option<TestSkipConfig>,
    responses(
        (status = 200, description = "Source skipped successfully", body = SourceChangeGeneratorState),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_change_generator_skip_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
    body: Json<Option<TestSkipConfig>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_change_generator_skip: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let skips_body = body.0.unwrap_or_default();

    let response = test_run_host
        .test_source_skip(&id, skips_body.num_skips, skips_body.spacing_mode)
        .await;
    match response {
        Ok(source) => Ok(Json(source.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/start",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    responses(
        (status = 200, description = "Source started successfully", body = SourceChangeGeneratorState),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_change_generator_start_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_change_generator_start: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_source_start(&id).await;
    match response {
        Ok(source) => Ok(Json(source.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "num_steps": 5,
    "spacing_mode": "Live"
}))]
pub struct TestStepConfig {
    /// Number of steps to execute
    #[serde(default)]
    pub num_steps: u64,
    /// Spacing mode for stepping through events
    pub spacing_mode: Option<SpacingMode>,
}

impl Default for TestStepConfig {
    fn default() -> Self {
        TestStepConfig {
            num_steps: 1,
            spacing_mode: None,
        }
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/step",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    request_body = Option<TestStepConfig>,
    responses(
        (status = 200, description = "Source stepped successfully", body = SourceChangeGeneratorState),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_change_generator_step_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
    body: Json<Option<TestStepConfig>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_change_generator_step: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let steps_body = body.0.unwrap_or_default();

    let response = test_run_host
        .test_source_step(&id, steps_body.num_steps, steps_body.spacing_mode)
        .await;
    match response {
        Ok(source) => Ok(Json(source.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources/{id}/stop",
    tag = "sources",
    params(
        ("id" = String, Path, description = "Source identifier")
    ),
    responses(
        (status = 200, description = "Source stopped successfully", body = SourceChangeGeneratorState),
        (status = 404, description = "Source not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn source_change_generator_stop_handler(
    Path(id): Path<String>,
    test_run_host: Extension<Arc<TestRunHost>>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - source_change_generator_stop: {}", id);

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let response = test_run_host.test_source_stop(&id).await;
    match response {
        Ok(source) => Ok(Json(source.state).into_response()),
        Err(e) => Err(TestServiceWebApiError::AnyhowError(e)),
    }
}

#[utoipa::path(
    post,
    path = "/test_run_host/sources",
    tag = "sources",
    request_body = test_run_host::sources::TestRunSourceConfig,
    responses(
        (status = 200, description = "Source created successfully", body = SourceStateResponse),
        (status = 400, description = "Invalid request body", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn post_source_handler(
    test_run_host: Extension<Arc<TestRunHost>>,
    body: Json<TestRunSourceConfig>,
) -> anyhow::Result<impl IntoResponse, TestServiceWebApiError> {
    log::info!("Processing call - post_source");

    // If the TestRunHost is an Error state, return an error and a description of the error.
    if let TestRunHostStatus::Error(msg) = &test_run_host.get_status().await? {
        return Err(TestServiceWebApiError::TestRunHostError(msg.to_string()));
    }

    let source_config = body.0;

    // Extract TestRunId from the config
    let test_run_id = match test_data_store::test_run_storage::TestRunId::try_from(&source_config) {
        Ok(id) => id,
        Err(e) => return Err(TestServiceWebApiError::AnyhowError(anyhow::anyhow!(e))),
    };

    match test_run_host
        .add_test_source(&test_run_id, source_config)
        .await
    {
        Ok(id) => match test_run_host.get_test_source_state(&id.to_string()).await {
            Ok(source) => Ok(Json(source).into_response()),
            Err(_) => Err(TestServiceWebApiError::NotFound(
                "TestRunSource".to_string(),
                id.to_string(),
            )),
        },
        Err(e) => {
            let msg = format!("Error creating Source: {}", e);
            log::error!("{}", &msg);
            Err(TestServiceWebApiError::AnyhowError(e))
        }
    }
}
