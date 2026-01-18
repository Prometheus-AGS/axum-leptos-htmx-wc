<!-- OPENSPEC:START -->
# OpenSpec Instructions

These instructions are for AI assistants working in this project.

Always open `@/openspec/AGENTS.md` when the request:
- Mentions planning or proposals (words like proposal, spec, change, plan)
- Introduces new capabilities, breaking changes, architecture shifts, or big performance/security work
- Sounds ambiguous and you need the authoritative spec before coding

Use `@/openspec/AGENTS.md` to learn:
- How to create and apply change proposals
- Spec format and conventions
- Project structure and guidelines

Keep this managed block so 'openspec update' can refresh the instructions.

<!-- OPENSPEC:END -->

# AGENTS.md - Repository Guidelines

## Build, Lint & Test

- **Rust**: `cargo build`, `cargo clippy --all-targets --all-features`, `cargo fmt`
- **Web**: `bun install`, `bun run build`, `bun run lint`, `bun run format`
- **Test All**: `cargo test`, `bun test web/tests`
- **Single Test**: `cargo test <test_name>`, `bun test <file_pattern>`
- **Clean Build**: ZERO warnings/errors allowed. Fix warnings immediately or use `#[expect(lint, reason="...")]`.

## Code Style & Conventions

- **Rust**: 4-space indent. `snake_case` (fn/mod), `CamelCase` (types). Use `anyhow` for app errors, `tracing` for structured logs.
- **TypeScript**: 2-space indent, semicolons, TS 5.9.3. Prefer Web Components in `web/components/`.
- **Imports**: No glob re-exports (`pub use foo::*`). Use `#[doc(inline)]` for public re-exports.
- **Documentation**: Public items must have `///` docs with `# Examples`, `# Errors`, and `# Panics` sections.

## Architecture & UI

- **Structure**: `src/` (Axum/Leptos SSR), `web/` (TS/Web Components), `static/` (Bundled assets).
- **UI Reference**: `docs/htmx/` for Material 3 Flat 2.0 patterns (borderless, token-based theming).
- **Config**: `.env` (see `.env.example`), `mcp.json` for MCP tools.
