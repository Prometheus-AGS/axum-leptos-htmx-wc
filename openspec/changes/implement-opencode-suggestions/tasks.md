# Tasks: Implement Opencode Assessment Suggestions

## Phase 1: Foundation & Cleanup

- [x] Fix existing lint warnings in `web/main.ts` and `web/components/storage-health/storage-health.ts`.
- [x] Sort imports in all modified files.

## Phase 2: P0 - Critical Impact

### Tauri Packaging

- [x] Configure sidecars in `src-tauri/tauri.conf.json`.
- [x] Implement sidecar path resolution in `src/mcp/config.rs`.
- [x] Update MCP server execution logic to use sidecars when running in Tauri.

### A11y Audit & Implementation

- [x] Audit `web/components/` for accessibility gaps.
- [x] Add ARIA labels and roles to all components.
- [x] Implement keyboard navigation for sidebar and settings.
- [x] Add focus management for dialogs.

## Phase 3: P1 - Medium Impact

### Storage Health

- [x] Integrate `storage-health` component into the main UI (e.g., sidebar footer).
- [x] Add visual warnings for high storage usage.

### Error Boundaries

- [x] Create a generic `ErrorBoundary` base class or utility for Web Components.
- [x] Wrap `chat-stream` and `conversation-sidebar` in error boundaries.

### Tool Analytics

- [x] Implement tool execution tracking in `src/uar/telemetry`.
- [x] Add metrics for latency and success rates.

## Phase 4: P2 - Low Impact / Polish

### Offline Mode

- [x] Implement and register a Service Worker.
- [x] Configure caching for static assets.
- [x] Ensure PGlite works offline.

### SSE Replay

- [x] Implement `Last-Event-ID` handling in `src/uar/api/sse.rs`.
- [x] Implement event buffering on the server (via DB-backed history replay).
- [x] Update `web/utils/sse.ts` to send `Last-Event-ID` on reconnect.

## Phase 5: Verification

- [x] Run `cargo clippy` and ensure zero warnings.
- [x] Run `bun run lint` and ensure zero warnings.
- [x] Run all tests (`cargo test`, `bun test`).
- [x] Verify Tauri build with sidecars.
