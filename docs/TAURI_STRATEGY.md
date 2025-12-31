# Tauri Integration Strategy

## 1. Overview
This document outlines the strategy for productizing the `axum-leptos-htmx-wc` application for Tauri, specifically addressing Server-Sent Events (SSE) compatibility and MCP server packaging.

## 2. Server Architecture: Localhost vs Custom Protocol
The application currently uses `EventSource` for critical streaming capabilities. Tauri's custom protocol (`tauri://`) does not strictly support `EventSource` in all WebViews due to origin policies.

**Decision**: Adopt a **Localhost Server** strategy.
- The Rust backend will spawn a local HTTP server on a random (or configured) port.
- The Tauri WebView will be pointed to `http://127.0.0.1:{port}` rather than loading assets from the custom protocol.
- This ensures 100% compatibility with SSE, valid origins for CORS, and standard browser behavior for all web APIs.

### Implementation Details
- **Main Process**: Finds an available port, starts Axum server.
- **WebView**: Initialized with the localhost URL.
- **Security**: The server should bind to `127.0.0.1` (loopback) only.

## 3. MCP Server Packaging
The current `mcp.json` relies on `npx` for some servers (e.g., `time-server`). This introduces a runtime dependency on Node.js/npm, which is brittle for a self-contained desktop app.

**Decision**: **Pre-packaged Binaries**.
- All default MCP servers must be shipped as binaries or embedded within the main binary path.
- `mcp.json` should refer to paths relative to the executable or a known resource directory.

### Packaging Pipeline
1. **Build Step**: Download/compile MCP servers (e.g. `mcp-time-server`) to a `bin/` or `resources/` directory.
2. **Tauri Config**: Include these binaries as sidecars or resources.
3. **Runtime**: The application resolves the path to these binaries and updates the MCP registry configuration dynamically (or via relative path expansion).

## 4. Client Lifecycle
- The `ChatStream` component is lifecycle-aware.
- `disconnectedCallback` invokes `view.destroy()` to clean up scroll listeners.
- This prevents memory leaks in long-running desktop sessions where tabs/panes might be closed and reopened.
