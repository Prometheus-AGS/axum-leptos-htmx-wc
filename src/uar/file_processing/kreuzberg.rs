//! Kreuzberg file processing provider.
//!
//! Uses the Kreuzberg Rust core for high-performance document extraction.
//! Supports 56+ file formats including PDF, Office documents, images, and more.

use async_trait::async_trait;
use std::path::Path;

use crate::config::KreuzbergConfig;

use super::provider::{ExtractedImage, FileProcessor, ProcessingError, ProcessingResult};

/// Kreuzberg-based file processor using the native Rust core.
///
/// Provides high-performance document extraction with OCR support.
/// Supports PDFs, Office documents, images, HTML, XML, archives, and more.
#[derive(Debug)]
pub struct KreuzbergProvider {
    config: KreuzbergConfig,
}

impl KreuzbergProvider {
    /// Create a new Kreuzberg provider with the given configuration.
    pub fn new(config: KreuzbergConfig) -> Self {
        Self { config }
    }

    /// Create a provider with default configuration.
    pub fn with_defaults() -> Self {
        Self {
            config: KreuzbergConfig::default(),
        }
    }

    /// Build the Kreuzberg ExtractionConfig from our config.
    fn build_extraction_config(&self) -> kreuzberg::ExtractionConfig {
        let mut config = kreuzberg::ExtractionConfig::default();

        // Configure OCR settings
        if self.config.ocr_enabled {
            let ocr_config = kreuzberg::OcrConfig {
                backend: self.config.ocr_backend.clone(),
                language: self.config.ocr_language.clone(),
                tesseract_config: None,
            };
            config.ocr = Some(ocr_config);
        }

        // Force OCR setting
        config.force_ocr = self.config.force_ocr;

        config
    }
}

#[async_trait]
impl FileProcessor for KreuzbergProvider {
    async fn process(&self, path: &Path) -> Result<ProcessingResult, ProcessingError> {
        let extraction_config = self.build_extraction_config();

        // Use kreuzberg's synchronous API wrapped in spawn_blocking
        // for async compatibility
        let path_buf = path.to_path_buf();
        let result = tokio::task::spawn_blocking(move || {
            kreuzberg::extract_file_sync(path_buf, None, &extraction_config)
        })
        .await
        .map_err(|e| ProcessingError::ProviderError(format!("Task join error: {}", e)))?
        .map_err(|e| ProcessingError::ProviderError(format!("Kreuzberg error: {}", e)))?;

        // Extract MIME type from the file
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Convert tables to a simple format if present
        let mut content = result.content.clone();

        // Append tables in markdown format if extracted
        if !result.tables.is_empty() {
            content.push_str("\n\n## Extracted Tables\n\n");
            for (i, table) in result.tables.iter().enumerate() {
                content.push_str(&format!("### Table {}\n\n", i + 1));
                // Tables are already in structured format from kreuzberg
                content.push_str(&format!("{:?}\n\n", table));
            }
        }

        // Convert metadata to JSON
        let metadata = if self.config.extract_metadata {
            Some(serde_json::json!({
                "title": result.metadata.title,
                "subject": result.metadata.subject,
                "authors": result.metadata.authors,
                "keywords": result.metadata.keywords,
                "language": result.metadata.language,
                "pages": result.metadata.pages,
            }))
        } else {
            None
        };

        // Extract any embedded images (if available)
        let images: Vec<ExtractedImage> = result
            .images
            .unwrap_or_default()
            .into_iter()
            .map(|img| ExtractedImage {
                data: img.data,
                mime_type: format!("image/{}", img.format),
                description: None,
            })
            .collect();

        Ok(ProcessingResult {
            content,
            mime_type,
            metadata,
            images,
        })
    }

    fn supports_mime_type(&self, mime_type: &str) -> bool {
        // Kreuzberg supports 56+ file formats
        matches!(
            mime_type,
            // PDFs
            "application/pdf"
            // Microsoft Office
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/vnd.ms-powerpoint"
            | "application/vnd.openxmlformats-officedocument.presentationml.presentation"
            // OpenDocument
            | "application/vnd.oasis.opendocument.text"
            | "application/vnd.oasis.opendocument.spreadsheet"
            | "application/vnd.oasis.opendocument.presentation"
            // Images (OCR)
            | "image/png"
            | "image/jpeg"
            | "image/jpg"
            | "image/gif"
            | "image/webp"
            | "image/bmp"
            | "image/tiff"
            // Text formats
            | "text/plain"
            | "text/markdown"
            | "text/html"
            | "text/xml"
            | "text/csv"
            | "text/rtf"
            | "application/json"
            | "application/xml"
            // Archives (kreuzberg extracts contents)
            | "application/zip"
            | "application/x-tar"
            | "application/gzip"
            // Email
            | "message/rfc822"
            | "application/vnd.ms-outlook"
            // Code files
            | "text/x-python"
            | "text/x-rust"
            | "text/javascript"
            | "text/typescript"
            | "text/x-java"
            | "text/x-c"
            | "text/x-c++"
        )
    }

    fn provider_name(&self) -> &'static str {
        "Kreuzberg"
    }
}

/// Process bytes directly without writing to disk.
/// Useful for processing uploaded file data.
#[allow(dead_code)]
pub async fn process_bytes(
    data: &[u8],
    mime_type: &str,
    config: &KreuzbergConfig,
) -> Result<ProcessingResult, ProcessingError> {
    let mut extraction_config = kreuzberg::ExtractionConfig::default();

    // Configure OCR
    if config.ocr_enabled {
        let ocr_config = kreuzberg::OcrConfig {
            backend: config.ocr_backend.clone(),
            language: config.ocr_language.clone(),
            tesseract_config: None,
        };
        extraction_config.ocr = Some(ocr_config);
    }

    extraction_config.force_ocr = config.force_ocr;

    let data_owned = data.to_vec();
    let mime_for_extraction = mime_type.to_string();
    let mime_for_result = mime_type.to_string();

    let result = tokio::task::spawn_blocking(move || {
        kreuzberg::extract_bytes_sync(&data_owned, &mime_for_extraction, &extraction_config)
    })
    .await
    .map_err(|e| ProcessingError::ProviderError(format!("Task join error: {}", e)))?
    .map_err(|e| ProcessingError::ProviderError(format!("Kreuzberg error: {}", e)))?;

    let mut content = result.content;

    // Append tables
    if !result.tables.is_empty() {
        content.push_str("\n\n## Extracted Tables\n\n");
        for (i, table) in result.tables.iter().enumerate() {
            content.push_str(&format!("### Table {}\n\n{:?}\n\n", i + 1, table));
        }
    }

    let metadata = if config.extract_metadata {
        Some(serde_json::json!({
            "title": result.metadata.title,
            "subject": result.metadata.subject,
            "authors": result.metadata.authors,
            "keywords": result.metadata.keywords,
            "language": result.metadata.language,
            "pages": result.metadata.pages,
        }))
    } else {
        None
    };

    let images: Vec<ExtractedImage> = result
        .images
        .unwrap_or_default()
        .into_iter()
        .map(|img| ExtractedImage {
            data: img.data,
            mime_type: format!("image/{}", img.format),
            description: None,
        })
        .collect();

    Ok(ProcessingResult {
        content,
        mime_type: mime_for_result,
        metadata,
        images,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kreuzberg_provider_creation() {
        let provider = KreuzbergProvider::with_defaults();
        assert_eq!(provider.provider_name(), "Kreuzberg");
    }

    #[test]
    fn test_mime_type_support() {
        let provider = KreuzbergProvider::with_defaults();

        // PDFs should be supported
        assert!(provider.supports_mime_type("application/pdf"));

        // Images should be supported (for OCR)
        assert!(provider.supports_mime_type("image/png"));
        assert!(provider.supports_mime_type("image/jpeg"));

        // Office documents
        assert!(provider.supports_mime_type(
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
        ));

        // Text files
        assert!(provider.supports_mime_type("text/plain"));
        assert!(provider.supports_mime_type("text/markdown"));
    }

    #[test]
    fn test_custom_config() {
        let config = KreuzbergConfig {
            ocr_enabled: true,
            force_ocr: true,
            ocr_backend: "tesseract".to_string(),
            ocr_language: "eng+deu".to_string(),
            pdf_dpi: 150,
            extract_tables: true,
            extract_metadata: true,
            output_format: "markdown".to_string(),
        };

        let provider = KreuzbergProvider::new(config);
        assert_eq!(provider.provider_name(), "Kreuzberg");
    }
}
