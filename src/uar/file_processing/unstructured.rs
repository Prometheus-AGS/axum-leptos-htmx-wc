//! Unstructured.io file processing provider.
//!
//! Supports both hosted (api.unstructured.io) and self-hosted deployments.
//! This provider can extract text from a wide variety of document formats
//! including PDF, DOCX, XLSX, PPTX, HTML, and more.

use super::provider::{FileProcessor, ProcessingError, ProcessingResult};
use crate::config::UnstructuredConfig;
use async_trait::async_trait;
use std::path::Path;

/// File processor using Unstructured.io API.
#[derive(Debug)]
pub struct UnstructuredProvider {
    client: reqwest::Client,
    config: UnstructuredConfig,
}

impl UnstructuredProvider {
    /// Create a new Unstructured.io provider with the given configuration.
    pub fn new(config: UnstructuredConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// Check if the provider is properly configured for use.
    pub fn is_configured(&self) -> bool {
        // Self-hosted doesn't require API key, hosted does
        // We consider it configured if we have either an API key OR a non-default URL
        self.config.api_key.is_some()
            || self.config.api_url != "https://api.unstructured.io/general/v0/general"
    }
}

#[async_trait]
impl FileProcessor for UnstructuredProvider {
    async fn process(&self, path: &Path) -> Result<ProcessingResult, ProcessingError> {
        let file_bytes = tokio::fs::read(path).await?;
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("document")
            .to_string();

        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name)
            .mime_str(&mime_type)
            .map_err(|e| ProcessingError::ProviderError(e.to_string()))?;

        let form = reqwest::multipart::Form::new()
            .part("files", part)
            .text("output_format", "text/markdown");

        let mut request = self.client.post(&self.config.api_url).multipart(form);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("unstructured-api-key", api_key);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ProcessingError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProcessingError::ProviderError(format!(
                "Unstructured API error ({}): {}",
                status, error_text
            )));
        }

        // Parse the response (array of elements)
        let elements: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| ProcessingError::ProviderError(e.to_string()))?;

        // Concatenate all text elements
        let content = elements
            .iter()
            .filter_map(|e| e.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join("\n\n");

        // Extract metadata if available
        let metadata = elements.first().and_then(|e| e.get("metadata").cloned());

        Ok(ProcessingResult {
            content,
            mime_type,
            metadata,
            images: vec![],
        })
    }

    fn supports_mime_type(&self, mime_type: &str) -> bool {
        // Unstructured supports a wide range of formats
        matches!(
            mime_type,
            "application/pdf"
                | "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                | "application/vnd.ms-powerpoint"
                | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                | "text/html"
                | "application/xml"
                | "text/xml"
                | "message/rfc822"
                | "application/epub+zip"
                | "text/plain"
                | "text/markdown"
                | "text/csv"
                | "application/rtf"
                | "image/png"
                | "image/jpeg"
                | "image/gif"
                | "image/webp"
                | "image/tiff"
        )
    }

    fn provider_name(&self) -> &'static str {
        "Unstructured.io"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_pdf() {
        let config = UnstructuredConfig::default();
        let provider = UnstructuredProvider::new(config);
        assert!(provider.supports_mime_type("application/pdf"));
    }

    #[test]
    fn test_supports_docx() {
        let config = UnstructuredConfig::default();
        let provider = UnstructuredProvider::new(config);
        assert!(provider.supports_mime_type(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ));
    }

    #[test]
    fn test_is_configured_with_api_key() {
        let config = UnstructuredConfig {
            api_url: "https://api.unstructured.io/general/v0/general".to_string(),
            api_key: Some("test-key".to_string()),
        };
        let provider = UnstructuredProvider::new(config);
        assert!(provider.is_configured());
    }

    #[test]
    fn test_is_configured_self_hosted() {
        let config = UnstructuredConfig {
            api_url: "http://localhost:8000/general/v0/general".to_string(),
            api_key: None,
        };
        let provider = UnstructuredProvider::new(config);
        assert!(provider.is_configured());
    }
}
