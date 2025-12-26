//! Rust SDK for axum-leptos-htmx-wc
//!
//! This SDK provides two usage modes:
//!
//! # HTTP Client (default feature)
//!
//! Use this mode to interact with a remote server via REST API:
//!
//! ```rust,no_run
//! use axum_leptos_htmx_wc_sdk::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("http://localhost:3000")?;
//!     
//!     // Chat API
//!     let response = client.chat().send("Hello!").await?;
//!     println!("Stream URL: {}", response.stream_url);
//!     
//!     // Knowledge Base API
//!     let kbs = client.knowledge().list().await?;
//!     for kb in kbs {
//!         println!("KB: {} ({})", kb.name, kb.id);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Embedded Runtime (feature = "embedded")
//!
//! Use this mode to embed the full runtime in your Rust application:
//!
//! ```rust,ignore
//! use axum_leptos_htmx_wc_sdk::Runtime;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let runtime = Runtime::builder()
//!         .config_path("config.yaml")
//!         .build()
//!         .await?;
//!     
//!     // Start the full HTTP server + background workers
//!     runtime.start().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;

#[cfg(feature = "http-client")]
pub mod client;

#[cfg(feature = "embedded")]
pub mod runtime;

// Re-exports
pub use error::Error;
pub use types::*;

#[cfg(feature = "http-client")]
pub use client::Client;

#[cfg(feature = "embedded")]
pub use runtime::Runtime;
