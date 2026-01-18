use crate::uar::{
    api::sse::build_sse_response,
    domain::artifact::AgentArtifact,
    runtime::manager::RunManager,
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

pub fn build_router() -> Router<Arc<RunManager>> {
    Router::new()
        .route("/runs", post(create_run))
        .route("/runs/{id}/stream", get(stream_run))
}

#[derive(Deserialize)]
struct CreateRunRequest {
    artifact: AgentArtifact,
    input: String,
    session_id: Option<String>,
}

#[derive(serde::Serialize)]
struct CreateRunResponse {
    run_id: String,
    stream_url: String,
}

#[derive(Deserialize)]
struct StreamParams {
    last_event_id: Option<u64>,
}

async fn create_run(
    State(manager): State<Arc<RunManager>>,
    Json(req): Json<CreateRunRequest>,
) -> Json<CreateRunResponse> {
    let run_id = manager
        .start_run(req.artifact, req.input, req.session_id, None)
        .await;
    Json(CreateRunResponse {
        run_id: run_id.clone(),
        stream_url: format!("/api/uar/runs/{run_id}/stream"),
    })
}

async fn stream_run(
    State(manager): State<Arc<RunManager>>,
    Path(run_id): Path<String>,
    Query(params): Query<StreamParams>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let Some(rx) = manager.subscribe(&run_id).await else {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    };

    let last_event_id = params.last_event_id.or_else(|| {
        headers
            .get("last-event-id")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
    });

    let replay = manager
        .history_since(&run_id, last_event_id)
        .await
        .unwrap_or_default();
    let replay_max_id = replay.last().map_or(0, |event| event.id);

    // Convert Broadcast Receiver to Stream
    let live_stream = BroadcastStream::new(rx)
        .filter_map(Result::ok)
        .filter(move |event| event.id > replay_max_id);

    let stream = tokio_stream::iter(replay).chain(live_stream);

    build_sse_response(stream).into_response()
}
