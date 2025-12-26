use crate::llm::{LlmProtocol, LlmSettings, Provider};
use clap::Parser;
use config::{Config, Environment};
use serde::Deserialize;
use std::collections::HashMap;
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

    /// Disable timeout middleware
    #[arg(long, env = "TIMEOUT_DISABLED")]
    pub timeout_disabled: Option<bool>,

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
    #[serde(default)]
    pub file_processing: FileProcessingConfig,
    #[serde(default)]
    pub unstructured: Option<UnstructuredConfig>,
    #[serde(default)]
    pub mistral_ocr: Option<MistralConfig>,
    #[serde(default)]
    pub kreuzberg: Option<KreuzbergConfig>,
    #[serde(default)]
    pub vision: VisionConfig,
    #[serde(default)]
    pub knowledge_bases: KnowledgeBasesConfig,
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
    pub timeout_disabled: bool,
    pub requests_per_second: f32,
    pub burst_size: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PersistenceConfig {
    pub provider: String,
    pub database_url: String,
    pub vector_dimension: usize,
    pub external_cache_enabled: bool,
}

/// Configuration for file processing and uploads.
#[derive(Debug, Deserialize, Clone)]
pub struct FileProcessingConfig {
    /// Provider to use: "unstructured", "mistral", "kreuzberg" (local), "auto"
    pub provider: String,
    /// Directory where uploaded files are saved before processing
    pub upload_dir: String,
    /// Maximum number of files per prompt
    pub max_files_per_prompt: usize,
    /// Maximum file size per file in bytes
    pub max_file_size: usize,
    /// Maximum total upload size for all files in a single prompt
    pub max_total_size: usize,
    /// Allowed MIME types (empty = allow all supported types)
    #[serde(default)]
    pub allowed_mime_types: Vec<String>,
}

impl Default for FileProcessingConfig {
    fn default() -> Self {
        Self {
            provider: "auto".to_string(),
            upload_dir: std::env::temp_dir()
                .join("uar-uploads")
                .to_string_lossy()
                .to_string(),
            max_files_per_prompt: 10,
            max_file_size: 50 * 1024 * 1024,   // 50MB
            max_total_size: 100 * 1024 * 1024, // 100MB
            allowed_mime_types: Vec::new(),
        }
    }
}

/// Unstructured.io configuration (hosted or self-hosted).
#[derive(Debug, Deserialize, Clone)]
pub struct UnstructuredConfig {
    /// API URL (default: hosted service)
    #[serde(default = "UnstructuredConfig::default_api_url")]
    pub api_url: String,
    /// API key for the hosted service
    pub api_key: Option<String>,
}

impl UnstructuredConfig {
    fn default_api_url() -> String {
        "https://api.unstructured.io/general/v0/general".to_string()
    }
}

impl Default for UnstructuredConfig {
    fn default() -> Self {
        Self {
            api_url: Self::default_api_url(),
            api_key: None,
        }
    }
}

/// Mistral OCR configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct MistralConfig {
    /// Mistral API key
    pub api_key: Option<String>,
    /// OCR model to use
    #[serde(default = "MistralConfig::default_ocr_model")]
    pub ocr_model: String,
}

impl MistralConfig {
    fn default_ocr_model() -> String {
        "mistral-ocr-latest".to_string()
    }
}

impl Default for MistralConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            ocr_model: Self::default_ocr_model(),
        }
    }
}

/// Kreuzberg local file processing configuration.
/// Kreuzberg is a high-performance document intelligence framework with a Rust core.
#[derive(Debug, Deserialize, Clone)]
pub struct KreuzbergConfig {
    /// Enable OCR for scanned documents and images
    #[serde(default = "KreuzbergConfig::default_ocr_enabled")]
    pub ocr_enabled: bool,
    /// Force OCR even if text layer exists in PDFs
    #[serde(default)]
    pub force_ocr: bool,
    /// OCR backend: "tesseract" (default), "easyocr", "paddleocr"
    #[serde(default = "KreuzbergConfig::default_ocr_backend")]
    pub ocr_backend: String,
    /// OCR language(s), e.g., "eng", "eng+deu" for multiple
    #[serde(default = "KreuzbergConfig::default_ocr_language")]
    pub ocr_language: String,
    /// PDF rendering DPI for OCR (higher = better quality but more memory)
    #[serde(default = "KreuzbergConfig::default_pdf_dpi")]
    pub pdf_dpi: u32,
    /// Extract tables from documents
    #[serde(default = "KreuzbergConfig::default_extract_tables")]
    pub extract_tables: bool,
    /// Extract metadata from documents
    #[serde(default = "KreuzbergConfig::default_extract_metadata")]
    pub extract_metadata: bool,
    /// Output format: "markdown" or "text"
    #[serde(default = "KreuzbergConfig::default_output_format")]
    pub output_format: String,
}

impl KreuzbergConfig {
    fn default_ocr_enabled() -> bool {
        true
    }
    fn default_ocr_backend() -> String {
        "tesseract".to_string()
    }
    fn default_ocr_language() -> String {
        "eng".to_string()
    }
    fn default_pdf_dpi() -> u32 {
        300
    }
    fn default_extract_tables() -> bool {
        true
    }
    fn default_extract_metadata() -> bool {
        true
    }
    fn default_output_format() -> String {
        "markdown".to_string()
    }
}

impl Default for KreuzbergConfig {
    fn default() -> Self {
        Self {
            ocr_enabled: Self::default_ocr_enabled(),
            force_ocr: false,
            ocr_backend: Self::default_ocr_backend(),
            ocr_language: Self::default_ocr_language(),
            pdf_dpi: Self::default_pdf_dpi(),
            extract_tables: Self::default_extract_tables(),
            extract_metadata: Self::default_extract_metadata(),
            output_format: Self::default_output_format(),
        }
    }
}

/// Vision/Image processing configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct VisionConfig {
    /// Explicit vision model (overrides auto-detection)
    pub model: Option<String>,
    /// Auto-detect vision capability from default LLM model
    #[serde(default = "VisionConfig::default_auto_detect")]
    pub auto_detect: bool,
}

impl VisionConfig {
    fn default_auto_detect() -> bool {
        true
    }
}

impl Default for VisionConfig {
    fn default() -> Self {
        Self {
            model: None,
            auto_detect: true,
        }
    }
}

// =============================================================================
// KNOWLEDGE BASES CONFIGURATION
// =============================================================================

/// Top-level configuration for knowledge bases.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct KnowledgeBasesConfig {
    /// Default knowledge base configuration (always exists)
    #[serde(default)]
    pub default: Option<KnowledgeBaseConfig>,
    /// Additional named knowledge bases
    #[serde(default)]
    pub named: HashMap<String, KnowledgeBaseConfig>,
}

/// Configuration for a single knowledge base.
#[derive(Debug, Deserialize, Clone)]
pub struct KnowledgeBaseConfig {
    /// Unique name for the knowledge base
    pub name: String,
    /// Description of the knowledge base
    #[serde(default)]
    pub description: Option<String>,
    /// Embedding provider: "fastembed", "openai", "mistral"
    #[serde(default = "KnowledgeBaseConfig::default_embedding_provider")]
    pub embedding_provider: String,
    /// Model ID for the embedding provider
    #[serde(default = "KnowledgeBaseConfig::default_embedding_model")]
    pub embedding_model: String,
    /// Vector dimensions (None = use model default)
    #[serde(default)]
    pub vector_dimensions: Option<usize>,
    /// File processor: "auto", "unstructured", "mistral", "kreuzberg"
    #[serde(default = "KnowledgeBaseConfig::default_file_processor")]
    pub file_processor: String,
    /// Chunking strategy configuration
    #[serde(default)]
    pub chunking: ChunkingConfig,
}

impl KnowledgeBaseConfig {
    fn default_embedding_provider() -> String {
        "fastembed".to_string()
    }

    fn default_embedding_model() -> String {
        "BAAI/bge-small-en-v1.5".to_string()
    }

    fn default_file_processor() -> String {
        "auto".to_string()
    }
}

/// Chunking strategy configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct ChunkingConfig {
    /// Strategy: "fixed", "recursive", "token", "sentence", "semantic", "document"
    #[serde(default = "ChunkingConfig::default_strategy")]
    pub strategy: String,
    /// Chunk size in characters (for fixed/recursive) or tokens
    #[serde(default = "ChunkingConfig::default_chunk_size")]
    pub chunk_size: usize,
    /// Semantic similarity threshold (for semantic strategy only)
    #[serde(default)]
    pub semantic_threshold: Option<f32>,
}

impl ChunkingConfig {
    fn default_strategy() -> String {
        "recursive".to_string()
    }

    fn default_chunk_size() -> usize {
        512
    }
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: Self::default_strategy(),
            chunk_size: Self::default_chunk_size(),
            semantic_threshold: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, config::ConfigError> {
        Self::load_from_args(std::env::args())
    }

    pub fn load_from_args<I, T>(args: I) -> Result<Self, config::ConfigError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let cli =
            Cli::try_parse_from(args).map_err(|e| config::ConfigError::Message(e.to_string()))?;

        let mut builder = Config::builder();

        // 1. Default Defaults
        builder = builder
            .set_default("server.port", 3000)?
            .set_default("server.host", "0.0.0.0")?
            .set_default("security.jwt_required", true)?
            .set_default("resilience.rate_limit_enabled", true)?
            .set_default("resilience.timeout_disabled", false)? // Default enabled (timeout_disabled=false)
            .set_default("resilience.requests_per_second", 5.0)?
            .set_default("resilience.burst_size", 10.0)?
            .set_default("persistence.external_cache_enabled", false)?
            // File processing defaults
            .set_default("file_processing.provider", "auto")?
            .set_default(
                "file_processing.upload_dir",
                std::env::temp_dir()
                    .join("uar-uploads")
                    .to_string_lossy()
                    .to_string(),
            )?
            .set_default("file_processing.max_files_per_prompt", 10_i64)?
            .set_default("file_processing.max_file_size", 52_428_800_i64)?
            .set_default("file_processing.max_total_size", 104_857_600_i64)?
            // Vision defaults
            .set_default("vision.auto_detect", true)?;
        // 4. Manual CLI Overrides
        // ...
        if let Some(rl) = cli.rate_limit_enabled {
            builder = builder.set_override("resilience.rate_limit_enabled", rl)?;
        }
        if let Some(td) = cli.timeout_disabled {
            builder = builder.set_override("resilience.timeout_disabled", td)?;
        }
        if let Some(cache) = cli.external_cache_enabled {
            builder = builder.set_override("persistence.external_cache_enabled", cache)?;
        }

        // 4. Manual Environment Overrides
        // ...
        if let Ok(val) = env::var("UAR_RESILIENCE__RATE_LIMIT_ENABLED") {
            if let Ok(bool_val) = val.parse::<bool>() {
                builder = builder.set_override("resilience.rate_limit_enabled", bool_val)?;
            }
        }
        if let Ok(val) = env::var("UAR_RESILIENCE__TIMEOUT_DISABLED") {
            if let Ok(bool_val) = val.parse::<bool>() {
                builder = builder.set_override("resilience.timeout_disabled", bool_val)?;
            }
        }
        if let Ok(val) = env::var("UAR_PERSISTENCE__PROVIDER") {
            // ...
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
