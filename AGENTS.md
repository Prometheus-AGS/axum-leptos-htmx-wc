# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Axum server and LLM orchestration. Entry point is `src/main.rs` with supporting modules in `src/llm/`, `src/mcp/`, and `src/normalized.rs`.
- `web/`: Web Component client code in TypeScript (see `web/app.ts`).
- `static/`: Static assets served at `/static` via Axum (bundle minified JS here; avoid CDN assets).
- `docs/coding-standards/`: Rust coding standards reference (imported guidelines).
- `mcp.json`: MCP server/tool configuration. `.env.example` documents required env vars.

## Build, Test, and Development Commands
- `cargo run`: Run the server locally (binds to `127.0.0.1:3000`). Ensure `.env` is configured first.
- `cargo build`: Compile the Rust application.
- `cargo fmt`: Format Rust code with rustfmt defaults.
- `cargo clippy --all-targets --all-features`: Lint Rust code for warnings and best practices.
- `cargo test`: Run tests (none are currently defined, but use this for future suites).
- `bun install`: Install web dependencies for TypeScript tooling.
- `bun run build`: Build the TypeScript bundle into `static/` (keep outputs local to avoid CDN use).

## Coding Style & Naming Conventions
- Rust: follow rustfmt defaults (4-space indentation). Use `snake_case` for modules/functions, `CamelCase` for types/traits, and `SCREAMING_SNAKE_CASE` for constants.
- TypeScript: keep formatting consistent with `web/app.ts` (2-space indentation, semicolons) and target TypeScript 5.9.3.
- API and architecture guidance: reference `docs/coding-standards/README.md` for Rust-specific conventions and documentation expectations.

## Testing Guidelines
- No test framework is configured yet. When adding tests, prefer module tests with `#[cfg(test)]` or integration tests in `tests/`.
- Use descriptive test names that mirror behavior (e.g., `streams_tool_results`).
- Always run `cargo test` before opening a PR.

## Commit & Pull Request Guidelines
- There is no existing commit history in this repo, so no enforced convention. Use short, imperative subjects (e.g., "Add SSE tool events") and include scope when helpful.
- PRs should include: a brief summary, how to run/verify changes, and any config updates (env vars or `mcp.json`). Include UI screenshots/GIFs when `web/` or `static/` output changes.

## Configuration & Secrets
- Copy `.env.example` to `.env` and set `LLM_BASE_URL`, `LLM_MODEL`, `LLM_PROTOCOL`, and `LLM_API_KEY` as needed.
- MCP tools are discovered from `mcp.json` and may require `TAVILY_API_KEY` (do not commit secrets).
