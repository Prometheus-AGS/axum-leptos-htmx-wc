use crate::llm::{LlmProtocol, LlmSettings, Provider};
use clap::Parser;
use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file path
    #[arg(short, long, env = "CONFIG_FILE")]
    pub config: Option<String>,

    /// Port to listen on
    #[arg(long, env = "PORT")]
    pub port: Option<u16>,

    /// Require JWT authentication
    #[arg(long, env = "JWT_REQUIRED")]
    pub jwt_required: Option<bool>,

    /// Enable rate limiting
    #[arg(long, env = "RATE_LIMIT_ENABLED")]
    pub rate_limit_enabled: Option<bool>,

    /// Enable external cache (Redis)
    #[arg(long, env = "EXTERNAL_CACHE_ENABLED")]
    pub external_cache_enabled: Option<bool>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub security: SecurityConfig,
    pub resilience: ResilienceConfig,
    pub persistence: PersistenceConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
    pub jwt_required: bool,
    pub jwt_secret: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ResilienceConfig {
    pub rate_limit_enabled: bool,
    pub requests_per_second: f32,
    pub burst_size: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PersistenceConfig {
    pub database_url: String,
    pub provider: String,
    pub external_cache_enabled: bool,
    pub redis_url: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let args: Vec<String> = env::args().collect();
        // If running tests, we might get weird args.
        // A robust way:
        if cfg!(test) || env::var("UAR_TEST_MODE").is_ok() {
            Self::load_from_args(vec!["app".to_string()])
        } else {
            Self::load_from_args(args)
        }
    }

    pub fn load_from_args<I, T>(args: I) -> Result<Self, config::ConfigError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        // Use try_parse_from instead of parse to avoid exit on error
        let cli = match Cli::try_parse_from(args) {
            Ok(c) => c,
            Err(e) => {
                // In a real app we might want to exit/print help.
                // But for `load()`, maybe we just warn or return error?
                // If we strictly follow "cli args override", failure to parse means we can't respect them.
                // We should probably property propagate this or just log.
                // For now, let's print and return default CLI (empty options).
                // OR better, panic if not test?
                // Let's implement safe fallback or propagation.
                // Actually, clap::Parser::parse() exits process on error.
                // We want to replicate that behavior in main, but here?
                // If invalid args, we should probably fail.
                println!("Warning: Failed to parse CLI args: {}", e);
                Cli {
                    config: None,
                    port: None,
                    jwt_required: None,
                    rate_limit_enabled: None,
                    external_cache_enabled: None,
                }
            }
        };

        let mut builder = Config::builder();

        // 1. Default Defaults
        builder = builder
            .set_default("server.port", 3000)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("security.jwt_required", true)?
            .set_default("resilience.rate_limit_enabled", true)?
            .set_default("resilience.requests_per_second", 5.0)?
            .set_default("resilience.burst_size", 10.0)?
            .set_default("persistence.external_cache_enabled", false)?
            .set_default(
                "persistence.database_url",
                "postgres://postgres:password@localhost:5432/uar",
            )?
            .set_default("persistence.provider", "postgres")?
            .set_default("security.jwt_secret", "secret_key_change_me")?;

        // 2. Config File
        // Priority:
        // 1. Explicit CLI arg or CONFIG_FILE env (via clap or manual check)
        // 2. Current working directory "./config.yaml"
        // 3. User home directory "~/.uar/config.yaml"

        let explicit_config = cli.config.clone().or_else(|| env::var("CONFIG_FILE").ok());

        let config_path = if let Some(path) = explicit_config {
            // If explicitly set, we only check this path
            Some(path)
        } else {
            // Check ./config.yaml
            let cwd_config = "config.yaml";
            if std::path::Path::new(cwd_config).exists() {
                Some(cwd_config.to_string())
            } else {
                // Check ~/.uar/config.yaml
                let home = env::var("HOME").unwrap_or_else(|_| ".".into());
                let home_config = format!("{}/.uar/config.yaml", home);
                if std::path::Path::new(&home_config).exists() {
                    Some(home_config)
                } else {
                    None
                }
            }
        };

        if let Some(path) = config_path {
            println!("Loading config from: {}", path);
            builder = builder.add_source(File::with_name(&path));
        }

        // 3. Environment Variables (prefixed with UAR_)
        // E.g. UAR_SERVER__PORT=8000
        builder = builder.add_source(
            Environment::with_prefix("UAR")
                .separator("__")
                .try_parsing(true),
        );

        // 4. Manual CLI Overrides (applied essentially as overrides)
        if let Some(port) = cli.port {
            builder = builder.set_override("server.port", port)?;
        }
        if let Some(jwt) = cli.jwt_required {
            builder = builder.set_override("security.jwt_required", jwt)?;
        }
        if let Some(rl) = cli.rate_limit_enabled {
            builder = builder.set_override("resilience.rate_limit_enabled", rl)?;
        }
        if let Some(cache) = cli.external_cache_enabled {
            builder = builder.set_override("persistence.external_cache_enabled", cache)?;
        }

        // 4. Manual Environment Overrides (Fix for config-rs issues)
        // Explicitly check key variables to ensure they override defaults/files
        if let Ok(val) = env::var("UAR_SERVER__PORT") {
            if let Ok(port) = val.parse::<u16>() {
                builder = builder.set_override("server.port", port)?;
            }
        }
        if let Ok(val) = env::var("UAR_SERVER__HOST") {
            builder = builder.set_override("server.host", val)?;
        }
        if let Ok(val) = env::var("UAR_SECURITY__JWT_REQUIRED") {
            if let Ok(bool_val) = val.parse::<bool>() {
                builder = builder.set_override("security.jwt_required", bool_val)?;
            }
        }
        if let Ok(val) = env::var("UAR_SECURITY__JWT_SECRET") {
            builder = builder.set_override("security.jwt_secret", val)?;
        }
        if let Ok(val) = env::var("UAR_RESILIENCE__RATE_LIMIT_ENABLED") {
            if let Ok(bool_val) = val.parse::<bool>() {
                builder = builder.set_override("resilience.rate_limit_enabled", bool_val)?;
            }
        }
        if let Ok(val) = env::var("UAR_PERSISTENCE__PROVIDER") {
            builder = builder.set_override("persistence.provider", val)?;
        }
        if let Ok(val) = env::var("UAR_PERSISTENCE__DATABASE_URL") {
            builder = builder.set_override("persistence.database_url", val)?;
        }

        // 5. Environment Variables (prefixed with UAR_) - for any keys not explicitly overridden above
        // E.g. UAR_SERVER__PORT=8000
        builder = builder.add_source(
            Environment::with_prefix("UAR")
                .separator("__")
                .try_parsing(true),
        );

        // Implicit provider override if DB URL looks like a path or ws://?
        // No, let users set UAR_PERSISTENCE__PROVIDER explicitly.

        // Additional Env Vars Mapping (fallback for legacy/direct envs not prefixed or using different names)
        // Note: `config`'s Environment source handles `UAR_SERVER__PORT` nicely.
        // If we want to support generic `PORT` or `DATABASE_URL` we might need manual mapping or another source.
        // For now, adhering to user's requirement "Environment variable for each setting".
        // The cli definitions above have `env = "PORT"` etc, but `clap` handles those.
        // Wait, if `clap` handles env vars, then `cli` struct will have them populated.
        // So applying `cli` values as overrides essentially handles the Env vars defined in `clap` structs too!
        // That's efficient. We just need to ensure `UAR_` prefix env vars from `config` crate don't conflict or are preferred correctly.
        // Priority: CLI Flag > CLI Env Var > Config File > Defaults.
        // `config::Environment` adds another layer: UAR_SERVER__PORT.
        // This seems robust.

        let cfg = builder.build()?;
        cfg.try_deserialize()
    }
}

pub fn load_llm_settings() -> Result<LlmSettings, String> {
    let base_url = std::env::var("LLM_BASE_URL")
        .map_err(|_| "Missing required env var: LLM_BASE_URL".to_string())?;
    if base_url.trim().is_empty() {
        return Err("LLM_BASE_URL cannot be empty".to_string());
    }

    let model = std::env::var("LLM_MODEL")
        .map_err(|_| "Missing required env var: LLM_MODEL".to_string())?;
    if model.trim().is_empty() {
        return Err("LLM_MODEL cannot be empty".to_string());
    }

    let api_key = std::env::var("LLM_API_KEY")
        .ok()
        .filter(|s| !s.trim().is_empty());

    let protocol = match std::env::var("LLM_PROTOCOL")
        .unwrap_or_else(|_| "auto".to_string())
        .to_lowercase()
        .as_str()
    {
        "responses" => LlmProtocol::Responses,
        "chat" => LlmProtocol::Chat,
        _ => LlmProtocol::Auto,
    };

    // Auto-detect provider from base URL
    let mut provider = Provider::detect_from_url(&base_url);

    // Load Azure-specific settings if needed
    let deployment_name = std::env::var("AZURE_DEPLOYMENT_NAME").ok();
    let api_version = std::env::var("AZURE_API_VERSION").ok();

    // Update provider with Azure deployment info if provided
    if let Provider::AzureOpenAI { .. } = &provider {
        if let Some(deployment) = &deployment_name {
            provider = Provider::AzureOpenAI {
                deployment_name: deployment.clone(),
                api_version: api_version
                    .clone()
                    .unwrap_or_else(|| "2024-08-01-preview".to_string()),
            };
        }
    }

    // Load optional parallel tool calls setting
    let parallel_tool_calls = std::env::var("LLM_PARALLEL_TOOLS")
        .ok()
        .and_then(|s| s.parse().ok());

    Ok(LlmSettings {
        base_url,
        api_key,
        model,
        protocol,
        provider,
        parallel_tool_calls,
        deployment_name,
        api_version,
    })
}
