# Proposal: Implement Opencode Assessment Suggestions

Implement all suggestions from `docs/OPENCODE_ASSESSMENT.md` to achieve S-Tier production readiness, ensuring zero errors/warnings and complete implementations.

## Problem

The `OPENCODE_ASSESSMENT.md` identified several areas for improvement to reach full production maturity, including Tauri packaging, accessibility, storage monitoring, error handling, analytics, offline support, and SSE reliability.

## Proposed Changes

1. **Tauri Packaging**: Formalize MCP server packaging as sidecars.
2. **A11y**: Implement ARIA labels, keyboard navigation, and screen reader support.
3. **Storage Health**: Enhance the existing `storage-health` component and integrate it into the UI.
4. **Error Boundaries**: Implement granular error boundaries in the frontend.
5. **Tool Analytics**: Add server-side tracking for MCP tool usage.
6. **Offline Mode**: Implement Service Workers and PGlite-backed offline reading.
7. **SSE Replay**: Implement `Last-Event-ID` support for SSE reconnections.

## Impact

- **Tauri**: Improved portability and reduced runtime dependencies.
- **A11y**: Better inclusivity and compliance.
- **UX**: Improved reliability (SSE replay, error boundaries) and transparency (storage health).
- **Observability**: Better insights into tool performance.
- **Resilience**: Offline support for conversation history.
