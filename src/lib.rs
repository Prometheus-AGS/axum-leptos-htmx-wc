//! Axum + Leptos + HTMX + Web Components
//!
//! An agentic streaming LLM application that supports tool-first interaction,
//! streams rich typed model output, and remains HTML-first and inspectable.
//!
//! # Architecture
//!
//! - **Server**: Axum-based HTTP server with SSE streaming
//! - **LLM Orchestration**: Protocol-agnostic driver for Chat Completions and Responses APIs
//! - **MCP Client**: Dynamic tool discovery and execution via Model Context Protocol
//! - **UI**: Leptos SSR + HTMX + Web Components + Alpine.js
//!
//! # Modules
//!
//! - [`llm`]: LLM driver traits and implementations
//! - [`mcp`]: MCP client configuration and registry
//! - [`normalized`]: Unified streaming event model
//! - [`session`]: Conversation and session management

// Allow pedantic clippy warnings that don't add value for this codebase
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::cargo_common_metadata)]
#![allow(clippy::multiple_crate_versions)]
#![allow(clippy::default_trait_access)]
#![allow(clippy::unused_async)]

pub mod llm;
pub mod mcp;
pub mod normalized;
pub mod session;
