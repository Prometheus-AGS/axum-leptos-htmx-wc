//! Session and conversation thread management.
//!
//! This module provides in-memory session storage for managing conversation
//! state across multiple requests. Sessions are identified by UUID and contain
//! the full message history.
//!
//! # Architecture
//!
//! - [`Session`]: Represents a single conversation session
//! - [`SessionStore`]: Thread-safe store for all active sessions
//!
//! # Example
//!
//! ```rust
//! use axum_leptos_htmx_wc::session::{Session, SessionStore};
//!
//! let store = SessionStore::new();
//! let session = store.create();
//! session.add_user_message("Hello!");
//!
//! let messages = session.messages();
//! assert_eq!(messages.len(), 1);
//! ```

mod thread;

pub use thread::{Session, SessionStore};
