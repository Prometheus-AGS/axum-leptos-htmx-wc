//! File Upload API handlers.
//!
//! Handles multipart file uploads for multimodal chat.

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// Maximum file size in bytes (50MB).
const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

/// Maximum total size for all files in a single request (100MB).
const MAX_TOTAL_SIZE: usize = 100 * 1024 * 1024;

/// Maximum number of files per request.
const MAX_FILES: usize = 10;

/// Allowed MIME type prefixes.
const ALLOWED_MIME_PREFIXES: &[&str] = &[
    "image/",
    "text/",
    "application/pdf",
    "application/msword",
    "application/vnd.openxmlformats-officedocument",
    "application/json",
    "application/xml",
];

/// Response for a successfully uploaded file.
#[derive(Debug, Serialize)]
pub struct UploadedFile {
    /// Unique identifier for the uploaded file.
    pub id: String,
    /// Original filename.
    pub filename: String,
    /// MIME type of the file.
    pub content_type: String,
    /// Size in bytes.
    pub size: usize,
    /// Whether the file is an image (for vision models).
    pub is_image: bool,
    /// Base64-encoded data URL for images (for inline display).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_url: Option<String>,
    /// Extracted text content (for non-image files).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_content: Option<String>,
}

/// Response for the file upload endpoint.
#[derive(Debug, Serialize)]
pub struct UploadResponse {
    /// Successfully uploaded files.
    pub files: Vec<UploadedFile>,
    /// Any errors encountered during upload.
    pub errors: Vec<String>,
}

/// Error response for upload failures.
#[derive(Debug, Serialize)]
pub struct UploadError {
    pub error: String,
    pub code: String,
}

/// Upload files for multimodal chat.
///
/// POST /api/upload
///
/// Accepts multipart form data with files. Returns processed files
/// ready for inclusion in chat messages.
#[allow(dead_code)]
pub async fn upload_handler(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<UploadError>)> {
    let mut uploaded_files = Vec::new();
    let mut errors = Vec::new();
    let mut total_size: usize = 0;
    let mut file_count: usize = 0;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(UploadError {
                error: format!("Failed to read multipart field: {}", e),
                code: "MULTIPART_ERROR".to_string(),
            }),
        )
    })? {
        // Check file count limit
        if file_count >= MAX_FILES {
            errors.push(format!("Maximum file count ({}) exceeded", MAX_FILES));
            break;
        }

        let filename = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("file_{}", uuid::Uuid::new_v4()));

        let content_type = field
            .content_type()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Validate MIME type
        let is_allowed = ALLOWED_MIME_PREFIXES
            .iter()
            .any(|prefix| content_type.starts_with(prefix));

        if !is_allowed {
            errors.push(format!(
                "File '{}' has unsupported type: {}",
                filename, content_type
            ));
            continue;
        }

        // Read file data
        let data = field.bytes().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(UploadError {
                    error: format!("Failed to read file '{}': {}", filename, e),
                    code: "READ_ERROR".to_string(),
                }),
            )
        })?;

        let size = data.len();

        // Check individual file size
        if size > MAX_FILE_SIZE {
            errors.push(format!(
                "File '{}' exceeds max size ({}MB > {}MB)",
                filename,
                size / (1024 * 1024),
                MAX_FILE_SIZE / (1024 * 1024)
            ));
            continue;
        }

        // Check total size
        if total_size + size > MAX_TOTAL_SIZE {
            errors.push(format!(
                "Total upload size would exceed limit ({}MB)",
                MAX_TOTAL_SIZE / (1024 * 1024)
            ));
            break;
        }

        total_size += size;
        file_count += 1;

        let is_image = content_type.starts_with("image/");
        let file_id = uuid::Uuid::new_v4().to_string();

        // Process the file
        let (data_url, text_content) = if is_image {
            // For images, create a base64 data URL
            let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);
            let data_url = format!("data:{};base64,{}", content_type, base64);
            (Some(data_url), None)
        } else if content_type.starts_with("text/")
            || content_type == "application/json"
            || content_type == "application/xml"
        {
            // For text files, extract the content
            match String::from_utf8(data.to_vec()) {
                Ok(text) => (None, Some(text)),
                Err(_) => {
                    errors.push(format!("File '{}' is not valid UTF-8 text", filename));
                    continue;
                }
            }
        } else {
            // For binary files (PDFs, Word docs, etc.), we'd need to use file processing
            // For now, just store the file reference
            // TODO: Use FileProcessorFactory to extract text from documents
            (None, None)
        };

        uploaded_files.push(UploadedFile {
            id: file_id,
            filename,
            content_type,
            size,
            is_image,
            data_url,
            text_content,
        });

        tracing::info!(
            file_count = file_count,
            total_size = total_size,
            is_image = is_image,
            "Processed uploaded file"
        );
    }

    Ok(Json(UploadResponse {
        files: uploaded_files,
        errors,
    }))
}

/// Request to process files for a chat message.
#[derive(Debug, Deserialize)]
pub struct ProcessFilesRequest {
    /// File IDs to include in the message.
    pub file_ids: Vec<String>,
    /// Session ID for the chat.
    pub session_id: Option<String>,
}

/// Response with processed files ready for LLM.
#[derive(Debug, Serialize)]
pub struct ProcessedFilesResponse {
    /// Image URLs (data URLs) for vision models.
    pub images: Vec<ImageRef>,
    /// Text content extracted from documents.
    pub text_content: Vec<TextRef>,
}

/// Reference to an image for vision models.
#[derive(Debug, Serialize)]
pub struct ImageRef {
    pub url: String,
    pub detail: String,
}

/// Reference to extracted text content.
#[derive(Debug, Serialize)]
pub struct TextRef {
    pub filename: String,
    pub content: String,
}
