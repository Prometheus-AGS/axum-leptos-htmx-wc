# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an agentic streaming LLM application that combines Rust (Axum + Leptos) with HTML-first frontend technologies (HTMX, Web Components, Alpine.js). The application is designed to be tool-first, streaming-native, and Tauri-compatible for web/desktop/mobile deployment.

## Development Commands

### Rust Backend
```bash
# Run the server in development mode
cargo run

# Build the Rust application
cargo build --release

# Run tests
cargo test

# Check for linting issues (extensive clippy configuration)
cargo clippy

# Format Rust code
cargo fmt
```

### Frontend Assets
```bash
# Build all frontend assets (TypeScript, CSS, WASM)
bun run build

# Development mode with file watching
bun run dev

# Type check TypeScript without emitting
bun run check

# Lint TypeScript files
bun run lint

# Format frontend code
bun run format
```

### Individual Asset Building
```bash
# Build TypeScript only
bun run build:ts

# Build CSS with Tailwind
bun run build:css

# Copy WASM files from dependencies
bun run copy:wasm
```

## Architecture Overview

### Core Technologies
- **Backend**: Rust with Axum web framework, Leptos for SSR
- **Frontend**: HTMX 2.0.8, Web Components (TypeScript), Alpine.js
- **Streaming**: Server-Sent Events (SSE) with normalized event model
- **Tools**: MCP (Model Context Protocol) via rmcp Rust SDK
- **Styling**: Tailwind CSS (ShadCN-inspired design system)

### Key Architectural Patterns

#### Event-Driven Streaming Architecture
The application uses a normalized event model for LLM interactions that supports:
- Token streaming (`message.delta`)
- Tool call streaming (`tool_call.delta`, `tool_call.complete`)
- Tool results (`tool_result`)
- Error handling (`error`)
- Completion signaling (`done`)

All events are mirrored into AG-UI-style events (`agui.*`) for future compatibility.

#### MCP Tool Integration
- Tools are discovered dynamically from `mcp.json`
- Supports both stdio and HTTP-based MCP servers
- Tools are namespaced automatically (e.g., `time::now`, `tavily::search`)
- Server controls all tool execution (not the model)

#### HTML-First UI Philosophy
- Uses HTMX for navigation and server interaction
- Web Components provide client-side programmability
- Alpine.js handles local UI state only
- Progressive enhancement over heavy SPA frameworks
- Identical UI across web/desktop/mobile via Tauri compatibility

### Key Components

#### Rust Backend (`src/`)
- `main.rs`: Entry point, Axum server configuration
- `lib.rs`: Core application logic and orchestrator
- Session management via `SessionStore`
- LLM orchestration via `Orchestrator`
- MCP tool registry via `McpRegistry`

#### Frontend (`web/components/`)
- `<chat-stream>`: Main streaming chat interface
- `<chat-messages>`: Message container and management
- `<chat-tool-call>`: Tool call visualization
- Other specialized components for code blocks, Mermaid diagrams, etc.

#### Static Assets (`static/`)
- `main.js`: Compiled TypeScript bundle
- `app.css`: Compiled Tailwind CSS
- `*.wasm`, `*.data`: PGLite WebAssembly files

## Configuration Files

### MCP Tools Configuration (`mcp.json`)
Define MCP servers for tool discovery:
```json
{
  "mcpServers": {
    "time": {
      "command": "npx",
      "args": ["-y", "@mcpcentral/mcp-time"]
    },
    "tavily": {
      "url": "https://mcp.tavily.com/mcp/?tavilyApiKey=${TAVILY_API_KEY}",
      "env": {
        "TAVILY_API_KEY": "${TAVILY_API_KEY}"
      }
    }
  }
}
```

### Environment Variables
Set up the following in `.env`:
- `TAVILY_API_KEY`: For web search functionality
- LLM provider API keys (OpenAI, Azure, etc.)

## Important Development Patterns

### Streaming Implementation
- All LLM interactions default to streaming mode
- Events flow: LLM → Orchestrator → SSE → Web Components
- Protocol-agnostic design supports OpenAI Chat Completions, Responses, and compatible backends

### Tool-First Design
- Tools are non-optional and always available
- Server maintains MCP client connections
- Tool calls are deterministic and server-controlled
- Dynamic tool discovery at startup

### Component Architecture
- Web Components consume typed SSE events
- Components have clear lifecycle hooks and boundaries
- Zero framework lock-in approach

### Tauri Compatibility
- No CDN scripts (all assets served locally)
- No API keys in browser
- Same codebase for web/desktop/mobile
- SSE works identically in webview

## Code Quality Standards

The project uses extensive Rust linting (see `Cargo.toml` lints section) including:
- Clippy with pedantic and performance lints
- Custom restriction lints for better code quality
- Structured logging with `tracing`
- `mimalloc` for performance optimization

## Key Files to Understand

- `src/main.rs`: Server setup and routing
- `src/lib.rs`: Core orchestration logic
- `web/main.ts`: Frontend entry point
- `web/components/chat-stream/chat-stream.ts`: Main streaming interface
- `mcp.json`: Tool configuration
- `package.json`: Frontend build scripts
- `Cargo.toml`: Rust dependencies and linting configuration

This architecture represents a modern approach to building AI applications that prioritizes tool use, streaming interactions, and clean separation between server logic and client presentation.