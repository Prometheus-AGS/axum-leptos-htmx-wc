//! File processing abstraction for document text extraction.
//!
//! This module provides a unified interface for extracting text content from
//! various document formats (PDF, DOCX, images, etc.) using configurable providers.
//!
//! # Providers
//!
//! - [`UnstructuredProvider`] - Unstructured.io (hosted or self-hosted)
//! - [`MistralProvider`] - Mistral OCR API
//! - [`KreuzbergProvider`] - Kreuzberg Rust core (high-performance local processing)
//! - [`LocalProvider`] - Simple local processing (fallback, text files only)
//!
//! # Usage
//!
//! ```rust,ignore
//! use crate::uar::file_processing::{FileProcessorFactory, FileProcessor};
//!
//! let processor = FileProcessorFactory::create(&config)?;
//! let result = processor.process(Path::new("document.pdf")).await?;
//! println!("Extracted: {}", result.content);
//! ```

mod factory;
mod kreuzberg;
mod local;
mod mistral;
mod provider;
mod unstructured;

pub use factory::FileProcessorFactory;
pub use kreuzberg::KreuzbergProvider;
pub use local::LocalProvider;
pub use mistral::MistralProvider;
pub use provider::{ExtractedImage, FileProcessor, ProcessingError, ProcessingResult};
pub use unstructured::UnstructuredProvider;
