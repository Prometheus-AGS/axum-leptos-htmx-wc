//! Embeddable runtime for the SDK.
//!
//! This module allows embedding the full server runtime in another Rust application.
//! Enable with `features = ["embedded"]`.

#[cfg(feature = "embedded")]
use axum_leptos_htmx_wc::{AppState, config::AppConfig, server};
use std::sync::Arc;

use crate::error::{Error, Result};

/// Embeddable runtime that replicates the standalone server behavior.
///
/// # Example
///
/// ```rust,ignore
/// use axum_leptos_htmx_wc_sdk::Runtime;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let runtime = Runtime::builder()
///         .config_path("config.yaml")
///         .build()
///         .await?;
///
///     // Option 1: Start the full HTTP server
///     runtime.start().await?;
///
///     // Option 2: Access components directly
///     if let Some(orchestrator) = runtime.orchestrator() {
///         // Use the orchestrator for LLM calls
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Runtime {
    #[cfg(feature = "embedded")]
    config: Arc<AppConfig>,
    #[cfg(feature = "embedded")]
    state: Option<AppState>,
}

impl Runtime {
    /// Create a new runtime builder.
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::default()
    }

    /// Start the runtime (HTTP server + background workers).
    ///
    /// This blocks until the server is shut down.
    #[cfg(feature = "embedded")]
    pub async fn start(&self) -> Result<()> {
        let settings = self.config.llm.clone();
        server::start_server(self.config.clone(), settings)
            .await
            .map_err(|e| Error::Runtime(e.to_string()))
    }

    /// Get a reference to the application configuration.
    #[cfg(feature = "embedded")]
    pub fn config(&self) -> &AppConfig {
        &self.config
    }

    /// Get a reference to the internal AppState for direct API access.
    #[cfg(feature = "embedded")]
    pub fn state(&self) -> Option<&AppState> {
        self.state.as_ref()
    }

    /// Access the LLM orchestrator directly (for programmatic LLM calls).
    #[cfg(feature = "embedded")]
    pub fn orchestrator(
        &self,
    ) -> Option<Arc<axum_leptos_htmx_wc::llm::orchestrator::Orchestrator>> {
        self.state.as_ref().map(|s| s.orchestrator.clone())
    }

    /// Access the run manager directly.
    #[cfg(feature = "embedded")]
    pub fn run_manager(
        &self,
    ) -> Option<Arc<axum_leptos_htmx_wc::uar::runtime::manager::RunManager>> {
        self.state.as_ref().map(|s| s.run_manager.clone())
    }

    /// Access the vector matcher for embedding operations.
    #[cfg(feature = "embedded")]
    pub fn vector_matcher(
        &self,
    ) -> Option<Arc<axum_leptos_htmx_wc::uar::runtime::matching::VectorMatcher>> {
        self.state.as_ref().map(|s| s.vector_matcher.clone())
    }

    /// Access the persistence layer.
    #[cfg(feature = "embedded")]
    pub fn persistence(
        &self,
    ) -> Option<Arc<dyn axum_leptos_htmx_wc::uar::persistence::PersistenceLayer>> {
        self.state.as_ref().and_then(|s| s.persistence.clone())
    }
}

/// Builder for creating a Runtime.
#[derive(Default)]
pub struct RuntimeBuilder {
    config_path: Option<String>,
    #[cfg(feature = "embedded")]
    config: Option<AppConfig>,
}

impl RuntimeBuilder {
    /// Set the path to the configuration file.
    pub fn config_path(mut self, path: impl Into<String>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Provide a configuration directly (instead of loading from file).
    #[cfg(feature = "embedded")]
    pub fn config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the runtime.
    #[cfg(feature = "embedded")]
    pub async fn build(self) -> Result<Runtime> {
        let config = match self.config {
            Some(c) => c,
            None => {
                // Load config (implementation depends on your config loading)
                AppConfig::load().map_err(|e| Error::Config(e.to_string()))?
            }
        };

        Ok(Runtime {
            config: Arc::new(config),
            state: None,
        })
    }

    /// Build the runtime (stub for when embedded feature is disabled).
    #[cfg(not(feature = "embedded"))]
    pub async fn build(self) -> Result<Runtime> {
        Err(Error::Config(
            "Embedded runtime requires the 'embedded' feature".to_string(),
        ))
    }
}
