use crate::AppState;
use axum::{
    Json,
    extract::{Multipart, State},
    response::IntoResponse,
};
use serde_json::json;
use std::env;
use tokio::fs;
use uuid::Uuid;

pub async fn ingest_handler(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut uploaded_files: Vec<String> = Vec::new();

    // Default KB ID if not provided (assume "default" or query param)
    // For MVP, hardcode "default".
    let kb_id = "default";

    while let Ok(Some(field)) = multipart.next_field().await {
        let _name = field.name().unwrap_or("unknown").to_string();
        let file_name = field.file_name().unwrap_or("unknown.txt").to_string();
        let _content_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        if let Ok(data) = field.bytes().await {
            // Save to temp file
            let temp_dir = env::temp_dir();
            let temp_path = temp_dir.join(format!("uar_ingest_{}_{}", Uuid::new_v4(), file_name));

            if let Err(e) = fs::write(&temp_path, &data).await {
                return Json(json!({ "error": format!("Failed to write temp file: {}", e) }))
                    .into_response();
            }

            // Ingest using service
            if let Some(ingest_service) = &state.ingest_service {
                match ingest_service.ingest_file(&temp_path, kb_id).await {
                    Ok(_) => {
                        uploaded_files.push(file_name);
                    }
                    Err(e) => {
                        let _ = fs::remove_file(&temp_path).await; // Cleanup
                        return Json(json!({ "error": format!("Ingest failed: {}", e) }))
                            .into_response();
                    }
                }
            } else {
                let _ = fs::remove_file(&temp_path).await;
                return Json(json!({ "error": "Ingest service not available" })).into_response();
            }

            // Cleanup
            let _ = fs::remove_file(&temp_path).await;
        }
    }

    Json(json!({
        "status": "success",
        "message": format!("Ingested {} files into KB '{}'", uploaded_files.len(), kb_id),
        "files": uploaded_files
    }))
    .into_response()
}
