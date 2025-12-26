use super::types::*;
use crate::AppState;
use crate::uar::security::claims::UserContext;
use crate::uar::{defaults, domain::events::NormalizedEvent};
use axum::{
    extract::{Extension, Json, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, Sse},
    },
};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub async fn list_models(State(_state): State<AppState>) -> impl IntoResponse {
    // In a real implementation, we would list all active agents from persistence
    // For now, we return the default/orchestrator agents
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let models = vec![
        ModelCard {
            id: "default".to_string(),
            object: "model".to_string(),
            created: now,
            owned_by: "uar".to_string(),
        },
        ModelCard {
            id: "orchestrator".to_string(),
            object: "model".to_string(),
            created: now,
            owned_by: "uar".to_string(),
        },
    ];

    Json(ModelList {
        object: "list".to_string(),
        data: models,
    })
}

pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(user_context): axum::Extension<UserContext>,
    Json(req): Json<ChatCompletionRequest>,
) -> impl IntoResponse {
    let run_manager = &state.run_manager;
    let conversation_id = Uuid::new_v4().to_string();
    let created = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Extract last user message
    let last_message = req
        .messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    // Map model ID to agent
    // Simple mapping for now
    let agent = if req.model == "orchestrator" {
        defaults::orchestrator_agent()
    } else {
        match run_manager.persistence.as_ref() {
            Some(p) => {
                // Try to load dynamic agent
                match futures::executor::block_on(p.load_agent_by_name(&req.model)) {
                    Ok(Some(a)) => a,
                    _ => defaults::default_agent(),
                }
            }
            None => defaults::default_agent(),
        }
    };

    // Start Run
    // Note: We are creating a NEW session for each request for simplicity in this adapter
    // because OpenAI API is stateless (except for message history passed in request).
    // Ideally we would map thread_id if UAR supported it in context, but UAR sessions are ID-based.
    // We'll create an ephemeral session ID here.
    let run_id = run_manager
        .start_run(
            agent,
            last_message.clone(),
            Some(conversation_id.clone()),
            Some(user_context.user_id),
        )
        .await;

    // Subscribe to events
    let mut rx = match run_manager.subscribe(&run_id).await {
        Some(rx) => rx,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to subscribe to run",
            )
                .into_response();
        }
    };

    // Convert to SSE stream
    let stream = async_stream::stream! {
        let id = Uuid::new_v4().to_string();

        // Yield initial role chunk
         let initial_chunk = ChatCompletionChunk {
            id: id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: req.model.clone(),
            choices: vec![ChatCompletionChunkChoice {
                index: 0,
                delta: ChatCompletionChunkDelta {
                    role: Some("assistant".to_string()),
                    content: Some("".to_string()),
                },
                finish_reason: None,
            }],
        };
        yield Ok::<_, std::convert::Infallible>(Event::default().json_data(initial_chunk).unwrap());

        while let Ok(event) = rx.recv().await {
            match event {
                NormalizedEvent::ChatDelta { text_delta, .. } => {
                    let chunk = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created,
                        model: req.model.clone(),
                        choices: vec![ChatCompletionChunkChoice {
                            index: 0,
                            delta: ChatCompletionChunkDelta {
                                role: None,
                                content: Some(text_delta),
                            },
                            finish_reason: None,
                        }],
                    };
                    yield Ok(Event::default().json_data(chunk).unwrap());
                }
                NormalizedEvent::RunDone { .. } => {
                     let chunk = ChatCompletionChunk {
                        id: id.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created,
                        model: req.model.clone(),
                        choices: vec![ChatCompletionChunkChoice {
                            index: 0,
                            delta: ChatCompletionChunkDelta::default(),
                            finish_reason: Some("stop".to_string()),
                        }],
                    };
                    yield Ok(Event::default().json_data(chunk).unwrap());
                    yield Ok(Event::default().data("[DONE]"));
                    break;
                }
                NormalizedEvent::Error { message, .. } => {
                    tracing::error!("Error in chat completion: {}", message);
                    break;
                }
                _ => {}
            }
        }
    };

    Sse::new(stream)
        .keep_alive(axum::response::sse::KeepAlive::default())
        .into_response()
}
