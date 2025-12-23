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

pub mod llm;
pub mod mcp;
pub mod normalized;
pub mod session;
