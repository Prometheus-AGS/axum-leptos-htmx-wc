//! Project initialization TUI tool.
//!
//! This tool provides an interactive terminal interface for initializing
//! a new project from the template. It prompts for project details and
//! performs find-and-replace operations across project files.

use anyhow::{Context, Result};
use console::{Emoji, style};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Confirm, Text};
use regex::Regex;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

// Emojis for visual feedback
static ROCKET: Emoji<'_, '_> = Emoji("🚀 ", "");
static CHECK: Emoji<'_, '_> = Emoji("✅ ", "[OK] ");
static GEAR: Emoji<'_, '_> = Emoji("⚙️  ", "");
static BROOM: Emoji<'_, '_> = Emoji("🧹 ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨ ", "");

/// Configuration collected from user prompts.
#[derive(Debug, Clone)]
struct ProjectConfig {
    /// Project name in kebab-case
    project_name: String,
    /// Project description
    description: String,
    /// Author name
    author_name: String,
    /// Author email
    author_email: String,
    /// GitHub organization or username
    github_org: String,
    /// Include Tauri desktop support
    enable_tauri: bool,
    /// Include Docker configuration
    enable_docker: bool,
    /// Include SDK scaffolding
    include_sdks: bool,
}

impl ProjectConfig {
    /// Derive the crate name from project name (kebab to snake case).
    fn crate_name(&self) -> String {
        self.project_name.replace('-', "_")
    }
}

/// Files to process for replacements.
const FILES_TO_PROCESS: &[&str] = &[
    "Cargo.toml",
    "package.json",
    "README.md",
    "src-tauri/Cargo.toml",
    "src-tauri/tauri.conf.json",
    "docker-compose.dev.yaml",
    "docker-compose.prod.yaml",
    "docker-compose.test.yaml",
    "sdks/rust/Cargo.toml",
    "sdks/rust/src/lib.rs",
    "sdks/rust/README.md",
    "sdks/typescript/package.json",
    "sdks/typescript/README.md",
    "sdks/python/pyproject.toml",
    "sdks/python/README.md",
    "sdks/python/src/axum_leptos_htmx_wc_sdk/__init__.py",
    "sdks/python/src/axum_leptos_htmx_wc_sdk/client.py",
    "TEMPLATE_USAGE.md",
];

fn main() -> Result<()> {
    println!();
    println!(
        "{} {}",
        ROCKET,
        style("Project Initialization").bold().cyan()
    );
    println!();

    // Collect configuration from user
    let config = collect_config()?;

    // Confirm before proceeding
    println!();
    println!("{}", style("Configuration:").bold());
    println!("  Project Name: {}", style(&config.project_name).green());
    println!("  Crate Name:   {}", style(config.crate_name()).green());
    println!("  Description:  {}", style(&config.description).dim());
    println!(
        "  Author:       {} <{}>",
        &config.author_name, &config.author_email
    );
    println!("  GitHub Org:   {}", style(&config.github_org).blue());
    println!(
        "  Tauri:        {}",
        if config.enable_tauri { "Yes" } else { "No" }
    );
    println!(
        "  Docker:       {}",
        if config.enable_docker { "Yes" } else { "No" }
    );
    println!(
        "  SDKs:         {}",
        if config.include_sdks { "Yes" } else { "No" }
    );
    println!();

    let proceed = Confirm::new("Proceed with initialization?")
        .with_default(true)
        .prompt()?;

    if !proceed {
        println!("{}", style("Aborted.").yellow());
        return Ok(());
    }

    // Perform replacements
    println!();
    println!("{} {}", GEAR, style("Applying replacements...").bold());
    apply_replacements(&config)?;

    // Remove optional components if disabled
    if !config.enable_tauri {
        println!("{} Removing Tauri support...", BROOM);
        remove_directory("src-tauri")?;
    }

    if !config.enable_docker {
        println!("{} Removing Docker configuration...", BROOM);
        remove_files(&[
            "Dockerfile",
            "docker-compose.dev.yaml",
            "docker-compose.prod.yaml",
            "docker-compose.test.yaml",
        ])?;
    }

    if !config.include_sdks {
        println!("{} Removing SDK scaffolding...", BROOM);
        remove_directory("sdks")?;
    }

    // Cleanup template files
    println!(
        "{} {}",
        BROOM,
        style("Cleaning up template files...").bold()
    );
    cleanup_template_files()?;

    // Success message
    println!();
    println!(
        "{} {}",
        SPARKLES,
        style("Project initialized successfully!").bold().green()
    );
    println!();
    println!("Next steps:");
    println!("  1. Review the changes: {}", style("git diff").cyan());
    println!("  2. Install dependencies: {}", style("bun install").cyan());
    println!("  3. Build the project: {}", style("cargo build").cyan());
    println!("  4. Run the dev server: {}", style("cargo run").cyan());
    println!();

    Ok(())
}

/// Collect project configuration from user prompts.
fn collect_config() -> Result<ProjectConfig> {
    let project_name = Text::new("Project name (kebab-case)?")
        .with_default("my-app")
        .with_validator(validate_kebab_case)
        .with_help_message("Use lowercase letters, numbers, and hyphens")
        .prompt()?;

    let description = Text::new("Project description?")
        .with_default("An agentic AI application with tool-first LLM interaction")
        .prompt()?;

    let author_name = Text::new("Author name?")
        .with_default("Developer")
        .prompt()?;

    let author_email = Text::new("Author email?")
        .with_default("dev@example.com")
        .prompt()?;

    let github_org = Text::new("GitHub organization/username?")
        .with_default("my-org")
        .prompt()?;

    let enable_tauri = Confirm::new("Include Tauri desktop support?")
        .with_default(true)
        .prompt()?;

    let enable_docker = Confirm::new("Include Docker configuration?")
        .with_default(true)
        .prompt()?;

    let include_sdks = Confirm::new("Include SDK scaffolding (Rust/TS/Python)?")
        .with_default(true)
        .prompt()?;

    Ok(ProjectConfig {
        project_name,
        description,
        author_name,
        author_email,
        github_org,
        enable_tauri,
        enable_docker,
        include_sdks,
    })
}

/// Validate that input is kebab-case.
fn validate_kebab_case(
    input: &str,
) -> Result<inquire::validator::Validation, Box<dyn std::error::Error + Send + Sync>> {
    let kebab_re = Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();
    if kebab_re.is_match(input) {
        Ok(inquire::validator::Validation::Valid)
    } else {
        Ok(inquire::validator::Validation::Invalid(
            "Must be kebab-case (lowercase letters, numbers, hyphens)".into(),
        ))
    }
}

/// Apply find-and-replace operations to project files.
fn apply_replacements(config: &ProjectConfig) -> Result<()> {
    let crate_name = config.crate_name();

    let pb = ProgressBar::new(FILES_TO_PROCESS.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("  {bar:40.cyan/blue} {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );

    for file_path in FILES_TO_PROCESS {
        pb.set_message(format!("{file_path}"));

        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path)
                .with_context(|| format!("Failed to read {file_path}"))?;

            let updated = content
                // Project name (kebab-case)
                .replace("axum-leptos-htmx-wc", &config.project_name)
                // Crate name (snake_case)
                .replace("axum_leptos_htmx_wc", &crate_name)
                // GitHub organization
                .replace("Prometheus-AGS", &config.github_org)
                // Author info
                .replace(
                    "Developer <dev@example.com>",
                    &format!("{} <{}>", config.author_name, config.author_email),
                );

            if updated != content {
                fs::write(file_path, updated)
                    .with_context(|| format!("Failed to write {file_path}"))?;
            }
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done");

    // Also process Rust files in sdks
    if config.include_sdks && Path::new("sdks/rust/src").exists() {
        process_rust_sdk_files(config)?;
    }

    Ok(())
}

/// Process Rust SDK source files for crate name replacements.
fn process_rust_sdk_files(config: &ProjectConfig) -> Result<()> {
    let crate_name = config.crate_name();

    for entry in WalkDir::new("sdks/rust/src")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        let updated = content.replace("axum_leptos_htmx_wc", &crate_name);

        if updated != content {
            fs::write(path, updated)?;
        }
    }

    Ok(())
}

/// Remove a directory recursively.
fn remove_directory(path: &str) -> Result<()> {
    if Path::new(path).exists() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory: {path}"))?;
        println!("  {} Removed {}", CHECK, path);
    }
    Ok(())
}

/// Remove specified files.
fn remove_files(files: &[&str]) -> Result<()> {
    for file in files {
        if Path::new(file).exists() {
            fs::remove_file(file).with_context(|| format!("Failed to remove file: {file}"))?;
            println!("  {} Removed {}", CHECK, file);
        }
    }
    Ok(())
}

/// Clean up template-specific files after initialization.
fn cleanup_template_files() -> Result<()> {
    // Remove cargo-generate config
    remove_files(&["cargo-generate.toml"])?;

    // Remove bootstrap script
    remove_files(&["bootstrap.sh"])?;

    // Remove template cleanup workflow
    remove_files(&[".github/workflows/template-cleanup.yml"])?;

    // Ask about removing this tool
    let remove_self = Confirm::new("Remove this initialization tool (tools/project-init)?")
        .with_default(true)
        .prompt()
        .unwrap_or(true);

    if remove_self {
        remove_directory("tools/project-init")?;

        // Also remove tools directory if empty
        if Path::new("tools").exists() {
            if fs::read_dir("tools")?.next().is_none() {
                fs::remove_dir("tools")?;
            }
        }
    }

    Ok(())
}
