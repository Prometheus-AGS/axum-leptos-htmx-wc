//! Model Context Protocol (MCP) client implementation.
//!
//! This module provides an MCP client that connects to stdio and HTTP-based
//! MCP servers for tool discovery and execution.
//!
//! # Configuration
//!
//! MCP servers are configured via `mcp.json`:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "time": {
//!       "command": "npx",
//!       "args": ["-y", "@mcpcentral/mcp-time"]
//!     },
//!     "tavily": {
//!       "url": "https://mcp.tavily.com/mcp/",
//!       "env": { "TAVILY_API_KEY": "${TAVILY_API_KEY}" }
//!     }
//!   }
//! }
//! ```
//!
//! # Tool Namespacing
//!
//! Tools are namespaced by server name: `server_name::tool_name`
//! (e.g., `time::now`, `tavily::search`).

pub mod config;
pub mod registry;
