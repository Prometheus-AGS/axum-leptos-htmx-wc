# Design: Opencode Assessment Implementation

## Tauri Packaging

- **Sidecars**: Configure Tauri to bundle MCP servers as sidecars.
- **Dynamic Path Resolution**: Update `src/mcp/config.rs` to resolve sidecar paths at runtime using `tauri::path::resource_dir`.

## Accessibility (A11y)

- **ARIA**: Add `role`, `aria-label`, `aria-live`, etc., to all Web Components.
- **Keyboard**: Ensure all interactive elements are focusable and reachable via Tab.
- **Focus Management**: Implement focus traps for dialogs.

## Storage Health

- **UI Integration**: Place the `storage-health` component in the settings or sidebar.
- **Thresholds**: Implement visual warnings (yellow/red) when storage usage exceeds 80%/90%.

## Error Boundaries

- **Component Level**: Wrap major UI sections in error boundaries that catch and display fallback UI.
- **Global Handler**: Enhance `window.onerror` and `unhandledrejection` to provide user-friendly feedback.

## Tool Analytics

- **Middleware/Interceptor**: Add a layer in `src/uar/telemetry` to record tool execution metrics (name, duration, success/failure).
- **Storage**: Store metrics in a way that can be queried (e.g., via `tracing` or a dedicated metrics collector).

## Offline Mode

- **Service Worker**: Register a service worker to cache static assets and provide a fallback for offline access.
- **PGlite**: Ensure PGlite is initialized and available even when offline.

## SSE Replay

- **Server-side**: Store recent events in a buffer (e.g., in-memory or Redis) and support `Last-Event-ID` header.
- **Client-side**: Send `Last-Event-ID` on reconnection.
