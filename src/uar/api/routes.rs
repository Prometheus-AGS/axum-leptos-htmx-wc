use crate::uar::{
    api::sse::build_sse_response,
    domain::{artifact::AgentArtifact, events::NormalizedEvent},
    runtime::manager::RunManager,
};
use axum::{
    Json, Router,
    extract::{Path, State},
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

async fn create_run(
    State(manager): State<Arc<RunManager>>,
    Json(req): Json<CreateRunRequest>,
) -> Json<CreateRunResponse> {
    let run_id = manager
        .start_run(req.artifact, req.input, req.session_id, None)
        .await;
    Json(CreateRunResponse {
        run_id: run_id.clone(),
        stream_url: format!("/api/uar/runs/{}/stream", run_id),
    })
}

async fn stream_run(
    State(manager): State<Arc<RunManager>>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let rx = match manager.subscribe(&run_id).await {
        Some(rx) => rx,
        None => return axum::http::StatusCode::NOT_FOUND.into_response(),
    };

    // Convert Broadcast Receiver to Stream
    let stream = BroadcastStream::new(rx).filter_map(|res: Result<NormalizedEvent, _>| res.ok());

    build_sse_response(stream).into_response()
}
