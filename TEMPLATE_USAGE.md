# Template Usage Guide

This project is a GitHub template for creating agentic AI applications with Rust, Axum, Leptos, HTMX, and Web Components.

## Quick Start

### Option 1: GitHub "Use this template"

1. Click "Use this template" on GitHub
2. The `template-cleanup.yml` workflow will automatically run on first push
3. Edit `.github/workflows/template-cleanup.yml` inputs before first push if needed

### Option 2: cargo-generate

```bash
cargo generate --git https://github.com/Prometheus-AGS/axum-leptos-htmx-wc
```

### Option 3: Bootstrap Script

```bash
git clone https://github.com/Prometheus-AGS/axum-leptos-htmx-wc my-project
cd my-project
./bootstrap.sh
```

---

## Template Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `project-name` | `axum-leptos-htmx-wc` | Project name (kebab-case) |
| `crate_name` | `axum_leptos_htmx_wc` | Rust crate name (auto-derived) |
| `github_org` | `Prometheus-AGS` | GitHub organization or username |
| `author_name` | `Developer` | Author name for package metadata |
| `enable_tauri` | `true` | Include Tauri desktop support |
| `enable_docker` | `true` | Include Docker configuration |
| `include_sdks` | `true` | Include SDK packages |

---

## Files Modified During Initialization

- `Cargo.toml` - Package name, binary name, repository URL
- `package.json` - Package name, repository URL
- `src-tauri/Cargo.toml` - Tauri crate name
- `src-tauri/tauri.conf.json` - Product name, identifier
- `docker-compose.*.yaml` - Network and volume names
- `sdks/*/` - SDK package names

---

## SDK Packages

The template includes client SDK scaffolding:

| Language | Location | Package Name |
|----------|----------|--------------|
| Rust | `sdks/rust/` | `{project-name}-sdk` |
| TypeScript | `sdks/typescript/` | `@{org}/{project-name}-sdk` |
| Python | `sdks/python/` | `{project-name}-sdk` |

### Rust SDK Features

- `http-client` (default): HTTP client for remote API calls
- `embedded`: Embed the full runtime as a library

---

## Post-Initialization Steps

1. Review `README.md` and update project description
2. Update `LICENSE` files if needed
3. Run `bun install` then `cargo build`
4. Test with `cargo run`
5. Commit changes: `git commit -m "chore: initialize from template"`
