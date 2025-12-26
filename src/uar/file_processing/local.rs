//! Local file processing provider.
//!
//! This is a fallback provider that handles text-based files locally
//! without requiring external API calls. For more complex document
//! formats, consider using Kreuzberg or another external provider.

use super::provider::{FileProcessor, ProcessingError, ProcessingResult};
use async_trait::async_trait;
use std::path::Path;

/// Local file processor for text-based files.
///
/// This provider handles simple text formats locally:
/// - Plain text (.txt)
/// - Markdown (.md, .markdown)
/// - JSON (.json)
/// - CSV (.csv)
/// - XML (.xml)
///
/// For binary formats like PDF, DOCX, etc., this provider
/// falls back to returning an error suggesting to use an
/// external provider.
#[derive(Debug, Default)]
pub struct LocalProvider;

impl LocalProvider {
    /// Create a new local file processor.
    pub fn new() -> Self {
        Self
    }

    /// Check if the provider is configured (always true for local).
    pub fn is_configured(&self) -> bool {
        true
    }
}

#[async_trait]
impl FileProcessor for LocalProvider {
    async fn process(&self, path: &Path) -> Result<ProcessingResult, ProcessingError> {
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // Check if we support this type
        if !self.supports_mime_type(&mime_type) {
            return Err(ProcessingError::UnsupportedType(format!(
                "Local provider cannot process {}: use an external provider (Unstructured.io or Mistral OCR)",
                mime_type
            )));
        }

        // Read the file as text
        let content = tokio::fs::read_to_string(path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::InvalidData {
                ProcessingError::UnsupportedType(format!(
                    "File is not valid UTF-8 text: {}",
                    path.display()
                ))
            } else {
                ProcessingError::IoError(e)
            }
        })?;

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
            "text/plain"
                | "text/markdown"
                | "text/csv"
                | "text/xml"
                | "application/json"
                | "application/xml"
        )
    }

    fn provider_name(&self) -> &'static str {
        "Local"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_supports_text() {
        let provider = LocalProvider::new();
        assert!(provider.supports_mime_type("text/plain"));
        assert!(provider.supports_mime_type("text/markdown"));
    }

    #[test]
    fn test_does_not_support_pdf() {
        let provider = LocalProvider::new();
        assert!(!provider.supports_mime_type("application/pdf"));
    }

    #[tokio::test]
    async fn test_process_text_file() {
        let provider = LocalProvider::new();

        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        writeln!(temp_file, "Hello, World!").unwrap();

        let result = provider.process(temp_file.path()).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("Hello, World!"));
    }

    #[tokio::test]
    async fn test_process_markdown_file() {
        let provider = LocalProvider::new();

        let mut temp_file = NamedTempFile::with_suffix(".md").unwrap();
        writeln!(temp_file, "# Heading\n\nParagraph text.").unwrap();

        let result = provider.process(temp_file.path()).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.content.contains("# Heading"));
    }

    #[test]
    fn test_always_configured() {
        let provider = LocalProvider::new();
        assert!(provider.is_configured());
    }
}
