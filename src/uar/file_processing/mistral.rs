//! Mistral OCR file processing provider.
//!
//! Uses Mistral's document AI API for OCR and document processing.
//! Supports PDF documents and images.

use super::provider::{FileProcessor, ProcessingError, ProcessingResult};
use crate::config::MistralConfig;
use async_trait::async_trait;
use base64::{Engine, engine::general_purpose::STANDARD};
use std::path::Path;

/// File processor using Mistral OCR API.
#[derive(Debug)]
pub struct MistralProvider {
    client: reqwest::Client,
    config: MistralConfig,
}

impl MistralProvider {
    /// Mistral OCR API endpoint.
    const API_URL: &'static str = "https://api.mistral.ai/v1/ocr";

    /// Create a new Mistral OCR provider with the given configuration.
    pub fn new(config: MistralConfig) -> Self {
        Self {
            client: reqwest::Client::new(),
            config,
        }
    }

    /// Check if the provider is properly configured for use.
    pub fn is_configured(&self) -> bool {
        self.config.api_key.is_some()
    }
}

#[async_trait]
impl FileProcessor for MistralProvider {
    async fn process(&self, path: &Path) -> Result<ProcessingResult, ProcessingError> {
        let api_key = self.config.api_key.as_ref().ok_or_else(|| {
            ProcessingError::ProviderNotConfigured("Mistral API key not configured".to_string())
        })?;

        let file_bytes = tokio::fs::read(path).await?;
        let base64_data = STANDARD.encode(&file_bytes);

        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Build the appropriate data URL based on MIME type
        let data_url = format!("data:{};base64,{}", mime_type, base64_data);

        // Determine document type for API
        let (doc_type, doc_value) = if mime_type == "application/pdf" {
            ("document_url", data_url)
        } else {
            ("image_url", data_url)
        };

        let body = serde_json::json!({
            "model": self.config.ocr_model,
            "document": {
                "type": doc_type,
                doc_type: doc_value
            },
            "table_format": "markdown"
        });

        let response = self
            .client
            .post(Self::API_URL)
            .bearer_auth(api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| ProcessingError::HttpError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ProcessingError::ProviderError(format!(
                "Mistral OCR error ({}): {}",
                status, error_text
            )));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ProcessingError::ProviderError(e.to_string()))?;

        // Extract markdown content from response
        let content = result
            .get("pages")
            .and_then(|p| p.as_array())
            .map(|pages| {
                pages
                    .iter()
                    .filter_map(|page| page.get("markdown").and_then(|m| m.as_str()))
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n")
            })
            .unwrap_or_default();

        Ok(ProcessingResult {
            content,
            mime_type,
            metadata: None,
            images: vec![],
        })
    }

    fn supports_mime_type(&self, mime_type: &str) -> bool {
        matches!(
            mime_type,
            "application/pdf"
                | "image/png"
                | "image/jpeg"
                | "image/gif"
                | "image/webp"
                | "image/tiff"
        )
    }

    fn provider_name(&self) -> &'static str {
        "Mistral OCR"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_pdf() {
        let config = MistralConfig::default();
        let provider = MistralProvider::new(config);
        assert!(provider.supports_mime_type("application/pdf"));
    }

    #[test]
    fn test_supports_images() {
        let config = MistralConfig::default();
        let provider = MistralProvider::new(config);
        assert!(provider.supports_mime_type("image/png"));
        assert!(provider.supports_mime_type("image/jpeg"));
    }

    #[test]
    fn test_not_configured_without_key() {
        let config = MistralConfig::default();
        let provider = MistralProvider::new(config);
        assert!(!provider.is_configured());
    }

    #[test]
    fn test_configured_with_key() {
        let config = MistralConfig {
            api_key: Some("test-key".to_string()),
            ocr_model: "mistral-ocr-latest".to_string(),
        };
        let provider = MistralProvider::new(config);
        assert!(provider.is_configured());
    }
}
