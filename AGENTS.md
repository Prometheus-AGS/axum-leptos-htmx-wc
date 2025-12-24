# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Axum server + Leptos SSR + LLM orchestration. Entry point is `src/main.rs` with supporting modules in `src/llm/`, `src/mcp/`, `src/session/`, `src/ui/`, and `src/normalized.rs` (typed streaming event contract). Shared library code lives in `src/lib.rs`.
- `web/`: Web Component client code in TypeScript. Bundler entry point is `web/main.ts` with supporting code in `web/components/`, `web/stores/`, and `web/utils/`.
- `static/`: Static assets served at `/static` via Axum (bundle/minify JS and CSS here; avoid CDN assets). Outputs include `main.js`, `app.css`, `pglite.wasm`, `pglite.data`, fonts, and vendored JS.
- `scripts/`: Developer utilities (e.g., `scripts/download-fonts.sh` to vendor fonts into `static/fonts`).
- `docs/coding-standards/`: Rust coding standards reference (imported guidelines).
- `mcp.json`: MCP server/tool configuration. `.env.example` documents required env vars.

## Build, Test, and Development Commands
- `cargo run`: Run the server locally (binds to `127.0.0.1:3000`). Ensure `.env` is configured first.
- `cargo build`: Compile the Rust application.
- `cargo fmt`: Format Rust code with rustfmt defaults.
- `cargo clippy --all-targets --all-features`: Lint Rust code for warnings and best practices.
- `cargo test`: Run Rust unit tests and doctests.
- `bun install`: Install web dependencies for TypeScript tooling.
- `bun run build`: Build the client bundle into `static/` (builds TS to `static/main.js`, builds Tailwind to `static/app.css`, and copies PGlite assets).
- `bun run dev`: Watch/rebuild the client bundle while developing.
- `bun run check`: Typecheck (`tsc --noEmit`).
- `bun run lint`: Lint TypeScript (`eslint web/`).
- `bun run format`: Format TypeScript (`prettier --write web/`).
- `bash scripts/download-fonts.sh`: (Re)download and vendor font files into `static/fonts`.

## Clean Build Policy
- **CRITICAL**: The codebase must ALWAYS compile with ZERO errors and ZERO warnings.
- Before completing any task, run `cargo build` and ensure the output shows no warnings.
- If warnings appear (unused imports, dead code, etc.), fix them immediately:
  - Remove unused imports with `cargo fix --allow-dirty --allow-staged`
  - Or manually remove/comment unused code
  - Add `#[allow(dead_code)]` only if the code will be used soon
- Run `cargo clippy --all-targets --all-features` and address all clippy warnings.
- Never leave warnings in the codebase, even if they seem minor.

## Coding Style & Naming Conventions
- Rust: follow rustfmt defaults (4-space indentation). Use `snake_case` for modules/functions, `CamelCase` for types/traits, and `SCREAMING_SNAKE_CASE` for constants.
- TypeScript: keep formatting consistent with `web/main.ts` (2-space indentation, semicolons) and target TypeScript 5.9.3. Prefer `bun run format` for consistent formatting.
- API and architecture guidance: reference `docs/coding-standards/README.md` for Rust-specific conventions and documentation expectations.

## Testing Guidelines
- This repo already includes module unit tests and doctests; keep `cargo test` green.
- When adding tests, prefer module tests with `#[cfg(test)]` or integration tests in `tests/`.
- Use descriptive test names that mirror behavior (e.g., `streams_tool_results`).
- Always run `cargo test` before opening a PR.

## Commit & Pull Request Guidelines
- Use short, imperative subjects (e.g., "Add SSE tool events") and include scope when helpful.
- PRs should include: a brief summary, how to run/verify changes, and any config updates (env vars or `mcp.json`). Include UI screenshots/GIFs when `web/` changes (and ensure corresponding `static/` outputs are updated if they are checked in).

## Configuration & Secrets
- Copy `.env.example` to `.env` and set `LLM_BASE_URL`, `LLM_MODEL`, `LLM_PROTOCOL`, and `LLM_API_KEY` as needed.
- MCP tools are discovered from `mcp.json` and may require `TAVILY_API_KEY` (do not commit secrets).
- Logging is controlled via `RUST_LOG` (e.g., `RUST_LOG=info`), using `tracing_subscriber`'s env filter.

## UI Component Reference Library
- **Location**: `docs/htmx/` directory contains HTMX sample code for shadcn-ui-like components and UI constructs.
- **Purpose**: These are reference implementations to inspire and accelerate the creation of Leptos or Web Components.
- **Usage**: When building new UI components, consult these examples for patterns, styling, and HTMX integration approaches.
- **Design Philosophy**: All examples follow Material 3 Flat 2.0 design (borderless zones, surface containers, token-based theming).

### Available Component Examples

#### Layout & Navigation
- **Sidebar** (`Sidebar-shadcnui-structure-HTMLX-artifact.html`)
  - Collapsible sidebar with navigation items
  - Responsive mobile/desktop behavior
  - User profile section
  - Supports nested navigation groups

- **Navigation Menu** (`Navigation-Menu-shadcnlike-HTMX-single-file.html`)
  - Horizontal navigation bar with dropdowns
  - Mega-menu support for complex navigation
  - Mobile hamburger menu variant
  - Keyboard navigation support

- **Prometheus Shell** (`Prometheus-Flat-Material-3-Zones-HTMX-Shell.html`)
  - Complete application shell layout
  - Header, sidebar, and main content areas
  - Material 3 zone-based design
  - Theme switcher integration

#### Data Display
- **Data Table** (`Data-Table-HTMX-singlefile-SortingFilteringPaginationSelectionRow-Actions.html`)
  - Full-featured data table with sorting, filtering, pagination
  - Row selection (single/multiple)
  - Row actions menu
  - Responsive column hiding
  - Server-side data loading via HTMX

- **Card** (`Isolated-HTMX-shadcnlike-Card-Prototype.html`)
  - Card component with header, content, footer
  - Multiple card variants (default, outline, ghost)
  - Supports images, actions, and badges
  - Composable card sections

#### Form Components
- **Forms** (`HTMX-208-shadcnlike-Forms-Token-Palette-LightDark.html`)
  - Complete form component set
  - Input, textarea, select, checkbox, radio
  - Form validation states (error, success, disabled)
  - Label and helper text patterns
  - Light/dark theme support

- **Field/FieldSet** (`HTMX-Field-Demo-Token-datatheme.html`)
  - Field wrapper with label, input, error message
  - FieldSet for grouping related fields
  - Inline validation feedback
  - HTMX integration for async validation

#### Interactive Components
- **Accordion** (`Accordion-shadcnui-structure-HTMX-HTMLX-artifact.html`)
  - Collapsible content sections
  - Single or multiple open panels
  - Smooth animations
  - Keyboard navigation (arrow keys, home, end)

- **Drawer** (`Drawer-Variations-LeftRightTopBottom-HTMX-HTMLX-Artifact.html`)
  - Slide-out panels from all four sides (left, right, top, bottom)
  - Overlay backdrop with click-to-close
  - Smooth slide animations
  - Responsive sizing

- **Sheet** (`HTMX-Sheet-shadcnlike-Demo.html`)
  - Modal sheet component (similar to drawer)
  - Bottom sheet for mobile
  - Side sheet for desktop
  - Supports forms and complex content

#### UI Elements
- **Buttons** (`HTMX-208-shadcnlike-Button-Grid-All-Variants.html`)
  - Complete button variant set
  - Variants: default, primary, secondary, destructive, outline, ghost, link
  - Sizes: sm, md, lg
  - Icon buttons and button groups
  - Loading states

- **Badges** (`shadcnlike-Badges-HTMX-208-Full-Variant-Set.html`)
  - Badge component for labels and status indicators
  - Variants: default, primary, secondary, destructive, outline
  - Sizes: sm, md, lg
  - Removable badges with close button
  - Pill and rounded styles

#### Content Editors
- **Calendar** (3 variants)
  - `Calendar-shadcnlike-HTMX-single-file.html` - Single month calendar with date selection
  - `Calendar-2month-layout-improved-HTMX-single-file.html` - Two-month side-by-side layout
  - Date range selection support
  - Disabled dates and custom date rendering
  - Keyboard navigation

- **Markdown Editor** (`HTMX-Markdown-Editor-Pluginready-Leptosfriendly.html`)
  - Split-pane markdown editor (source + preview)
  - Syntax highlighting via Highlight.js
  - Mermaid diagram support
  - Live preview with markdown rendering
  - Toolbar with common formatting actions
  - Plugin-ready architecture for extensions

### Common Patterns Across All Examples

1. **Token-Based Theming**
   - All colors defined as CSS custom properties
   - `--primary-hue` as configurable personalization knob
   - Derived surface container levels for zones
   - Light/dark theme support via `data-theme` attribute

2. **Borderless Design**
   - No visible borders on components
   - Separation via background color zones
   - Surface container hierarchy for depth
   - Shadows for elevation when needed

3. **HTMX Integration**
   - All examples include HTMX 2.0.8
   - Use `hx-get`, `hx-post`, `hx-target`, `hx-swap` attributes
   - Server-side rendering patterns
   - Progressive enhancement approach

4. **Responsive Design**
   - Mobile-first breakpoint at 768px
   - Touch-friendly tap targets (44px minimum)
   - Adaptive layouts for mobile/tablet/desktop
   - Responsive typography and spacing

5. **Accessibility**
   - Keyboard navigation support
   - ARIA attributes for screen readers
   - Focus management and visible focus indicators
   - Semantic HTML structure

### Usage Guidelines

When creating new UI components:
1. **Reference First**: Check if a similar component exists in `docs/htmx/`
2. **Extract Patterns**: Copy the token system, layout patterns, and HTMX integration approach
3. **Adapt for Stack**: Convert to Leptos components or Web Components as needed
4. **Maintain Philosophy**: Keep borderless design, token-based theming, and responsive patterns
5. **Test Thoroughly**: Verify light/dark themes, responsive breakpoints, and keyboard navigation
