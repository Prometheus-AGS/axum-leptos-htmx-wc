//! Factory for creating file processors based on configuration.

use super::kreuzberg::KreuzbergProvider;
use super::local::LocalProvider;
use super::mistral::MistralProvider;
use super::provider::{FileProcessor, ProcessingError};
use super::unstructured::UnstructuredProvider;
use crate::config::{FileProcessingConfig, KreuzbergConfig, MistralConfig, UnstructuredConfig};
use std::sync::Arc;

/// Factory for creating file processors based on configuration.
#[derive(Debug)]
pub struct FileProcessorFactory;

impl FileProcessorFactory {
    /// Create a file processor based on configuration.
    ///
    /// # Provider Selection
    ///
    /// For "auto" mode, returns the first available provider in order:
    /// 1. Kreuzberg (if configured, high-performance local processing)
    /// 2. Unstructured.io (if API key configured or self-hosted URL)
    /// 3. Mistral OCR (if API key configured)
    /// 4. Local (always available, text files only)
    ///
    /// # Arguments
    ///
    /// * `config` - File processing configuration
    /// * `unstructured` - Optional Unstructured.io configuration
    /// * `mistral` - Optional Mistral OCR configuration
    /// * `kreuzberg` - Optional Kreuzberg configuration
    ///
    /// # Returns
    ///
    /// An `Arc<dyn FileProcessor>` that can be used to process files.
    pub fn create(
        config: &FileProcessingConfig,
        unstructured: Option<&UnstructuredConfig>,
        mistral: Option<&MistralConfig>,
        kreuzberg: Option<&KreuzbergConfig>,
    ) -> Result<Arc<dyn FileProcessor>, ProcessingError> {
        match config.provider.as_str() {
            "kreuzberg" => {
                let cfg = kreuzberg.cloned().unwrap_or_default();
                tracing::info!(
                    "Using Kreuzberg for file processing (OCR backend: {})",
                    cfg.ocr_backend
                );
                Ok(Arc::new(KreuzbergProvider::new(cfg)))
            }
            "unstructured" => {
                let cfg = unstructured.ok_or_else(|| {
                    ProcessingError::ProviderNotConfigured(
                        "Unstructured configuration required".to_string(),
                    )
                })?;
                Ok(Arc::new(UnstructuredProvider::new(cfg.clone())))
            }
            "mistral" => {
                let cfg = mistral.ok_or_else(|| {
                    ProcessingError::ProviderNotConfigured(
                        "Mistral configuration required".to_string(),
                    )
                })?;
                if !cfg.api_key.is_some() {
                    return Err(ProcessingError::ProviderNotConfigured(
                        "Mistral API key required".to_string(),
                    ));
                }
                Ok(Arc::new(MistralProvider::new(cfg.clone())))
            }
            "local" => {
                tracing::info!("Using local file processing (text files only)");
                Ok(Arc::new(LocalProvider::new()))
            }
            "auto" | _ => {
                // Try providers in order of preference

                // 1. Kreuzberg (high-performance local processing)
                if let Some(cfg) = kreuzberg {
                    tracing::info!(
                        "Using Kreuzberg for file processing (OCR backend: {})",
                        cfg.ocr_backend
                    );
                    return Ok(Arc::new(KreuzbergProvider::new(cfg.clone())));
                }

                // 2. Unstructured.io
                if let Some(cfg) = unstructured {
                    let provider = UnstructuredProvider::new(cfg.clone());
                    if provider.is_configured() {
                        tracing::info!("Using Unstructured.io for file processing");
                        return Ok(Arc::new(provider));
                    }
                }

                // 3. Mistral OCR
                if let Some(cfg) = mistral {
                    let provider = MistralProvider::new(cfg.clone());
                    if provider.is_configured() {
                        tracing::info!("Using Mistral OCR for file processing");
                        return Ok(Arc::new(provider));
                    }
                }

                // 4. Fall back to local processing
                tracing::info!("Using local file processing (text files only)");
                Ok(Arc::new(LocalProvider::new()))
            }
        }
    }

    /// Create a processor for a specific file, choosing the best provider.
    ///
    /// This method selects a provider based on the file's MIME type,
    /// preferring providers that explicitly support the type.
    pub fn create_for_file(
        path: &std::path::Path,
        config: &FileProcessingConfig,
        unstructured: Option<&UnstructuredConfig>,
        mistral: Option<&MistralConfig>,
        kreuzberg: Option<&KreuzbergConfig>,
    ) -> Result<Arc<dyn FileProcessor>, ProcessingError> {
        let mime_type = mime_guess::from_path(path)
            .first_or_octet_stream()
            .to_string();

        // For complex documents, prefer Kreuzberg (if available)
        if let Some(cfg) = kreuzberg {
            let provider = KreuzbergProvider::new(cfg.clone());
            if provider.supports_mime_type(&mime_type) {
                return Ok(Arc::new(provider));
            }
        }

        // For images, prefer Mistral OCR (if API key available)
        if mime_type.starts_with("image/") {
            if let Some(cfg) = mistral {
                let provider = MistralProvider::new(cfg.clone());
                if provider.is_configured() && provider.supports_mime_type(&mime_type) {
                    return Ok(Arc::new(provider));
                }
            }
        }

        // For PDFs and complex documents, prefer Unstructured
        if matches!(
            mime_type.as_str(),
            "application/pdf"
                | "application/msword"
                | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                | "application/vnd.ms-excel"
                | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        ) {
            if let Some(cfg) = unstructured {
                let provider = UnstructuredProvider::new(cfg.clone());
                if provider.is_configured() && provider.supports_mime_type(&mime_type) {
                    return Ok(Arc::new(provider));
                }
            }
        }

        // Fall back to default provider selection
        Self::create(config, unstructured, mistral, kreuzberg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_local_provider() {
        let config = FileProcessingConfig {
            provider: "local".to_string(),
            ..Default::default()
        };
        let result = FileProcessorFactory::create(&config, None, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "Local");
    }

    #[test]
    fn test_create_kreuzberg_provider() {
        let config = FileProcessingConfig {
            provider: "kreuzberg".to_string(),
            ..Default::default()
        };
        let kreuzberg_config = KreuzbergConfig::default();
        let result = FileProcessorFactory::create(&config, None, None, Some(&kreuzberg_config));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "Kreuzberg");
    }

    #[test]
    fn test_create_auto_fallback_to_local() {
        let config = FileProcessingConfig {
            provider: "auto".to_string(),
            ..Default::default()
        };
        let result = FileProcessorFactory::create(&config, None, None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "Local");
    }

    #[test]
    fn test_create_auto_with_kreuzberg() {
        let config = FileProcessingConfig {
            provider: "auto".to_string(),
            ..Default::default()
        };
        let kreuzberg_config = KreuzbergConfig::default();
        let result = FileProcessorFactory::create(&config, None, None, Some(&kreuzberg_config));
        assert!(result.is_ok());
        // When kreuzberg config is present, auto mode should prefer it
        assert_eq!(result.unwrap().provider_name(), "Kreuzberg");
    }

    #[test]
    fn test_create_unstructured_without_config() {
        let config = FileProcessingConfig {
            provider: "unstructured".to_string(),
            ..Default::default()
        };
        let result = FileProcessorFactory::create(&config, None, None, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_unstructured_with_config() {
        let config = FileProcessingConfig {
            provider: "unstructured".to_string(),
            ..Default::default()
        };
        let unstructured_config = UnstructuredConfig {
            api_url: "http://localhost:8000".to_string(),
            api_key: Some("test-key".to_string()),
        };
        let result = FileProcessorFactory::create(&config, Some(&unstructured_config), None, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider_name(), "Unstructured.io");
    }
}
