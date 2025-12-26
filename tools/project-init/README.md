# Project Init TUI

Interactive terminal tool for initializing projects from the template.

## Usage

From the project root:

```bash
cd tools/project-init
cargo run
```

Or build and run directly:

```bash
cargo build --release -p project-init
./target/release/project-init
```

## Features

- **Interactive prompts** - Guided project setup with validation
- **Smart defaults** - Sensible defaults for all options
- **Selective components** - Choose which features to include (Tauri, Docker, SDKs)
- **Progress feedback** - Visual progress bar during replacements
- **Self-cleanup** - Removes itself after initialization (optional)

## What it does

1. Prompts for project name, description, author info, etc.
2. Replaces template placeholders in project files
3. Optionally removes Tauri, Docker, or SDK scaffolding
4. Cleans up template-specific files (cargo-generate.toml, bootstrap.sh, etc.)
5. Offers to remove itself after initialization
