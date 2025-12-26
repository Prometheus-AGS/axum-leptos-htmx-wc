//! Core trait and types for file processing providers.

use async_trait::async_trait;
use std::path::Path;

/// Result of file processing.
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    /// Extracted text content (Markdown format preferred).
    pub content: String,
    /// Original MIME type of the processed file.
    pub mime_type: String,
    /// Extracted metadata (title, author, page count, etc.).
    pub metadata: Option<serde_json::Value>,
    /// Extracted images (for documents with embedded images).
    pub images: Vec<ExtractedImage>,
}

/// An image extracted from a document.
#[derive(Debug, Clone)]
pub struct ExtractedImage {
    /// Raw image data.
    pub data: Vec<u8>,
    /// MIME type of the image (e.g., "image/png").
    pub mime_type: String,
    /// Optional description or alt text.
    pub description: Option<String>,
}

/// Errors that can occur during file processing.
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    /// The file type is not supported by this provider.
    #[error("Unsupported file type: {0}")]
    UnsupportedType(String),

    /// The provider is not properly configured.
    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    /// An error occurred in the provider's API or processing.
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// An I/O error occurred while reading the file.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// HTTP request error.
    #[error("HTTP error: {0}")]
    HttpError(String),
}

/// Trait for file processing providers.
///
/// Implementors of this trait can extract text content from various
/// document formats. Each provider may support different file types
/// and have different requirements (API keys, network access, etc.).
#[async_trait]
pub trait FileProcessor: Send + Sync + std::fmt::Debug {
    /// Process a file and extract text content.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the file to process
    ///
    /// # Returns
    ///
    /// A [`ProcessingResult`] containing the extracted content, or an error.
    async fn process(&self, path: &Path) -> Result<ProcessingResult, ProcessingError>;

    /// Check if this processor supports the given MIME type.
    ///
    /// # Arguments
    ///
    /// * `mime_type` - The MIME type to check (e.g., "application/pdf")
    ///
    /// # Returns
    ///
    /// `true` if this processor can handle files of this type.
    fn supports_mime_type(&self, mime_type: &str) -> bool;

    /// Get the provider name for logging and debugging.
    fn provider_name(&self) -> &'static str;
}
