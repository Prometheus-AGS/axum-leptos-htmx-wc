use crate::uar::domain::artifact::*;
use std::collections::HashMap;

/// Returns the default agent artifact used when no specific agent is requested.
pub fn default_agent() -> AgentArtifact {
    AgentArtifact {
        version: "1.0.0".to_string(),
        kind: "agent".to_string(),
        id: "default-agent".to_string(),
        metadata: AgentMetadata {
            title: "Default Assistant".to_string(),
            description: "A helpful generic AI assistant.".to_string(),
            tags: vec!["default".to_string(), "general".to_string()],
            author: Some("System".to_string()),
            icon: None,
        },
        runtime: AgentRuntimeConfig {
            entry: "default".to_string(),
            protocols: HashMap::new(),
        },
        policy: AgentPolicy {
            provider: ProviderPolicy {
                default: ProviderSelection {
                    provider: "openai".to_string(),
                    model: "gpt-4o".to_string(),
                },
                fallbacks: vec![],
            },
            tools: ToolPolicy {
                allow: vec!["*".to_string()],
                deny: vec![],
                max_concurrent: 1,
            },
            skills: SkillPolicy {
                prefer: vec![],
                max_active: 3,
            },
        },
        schemas: AgentSchemas {
            inputs: None,
            outputs: None,
            state: None,
        },
        prompt: AgentPrompt {
            system: "You are a helpful, intelligent, and capable AI assistant. \
            You can answer questions, perform tasks, and use provided tools. \
            Always provide clear, concise, and accurate information."
                .to_string(),
            instructions: vec![],
        },
        memory: AgentMemoryConfig {
            conversation: ConversationMemory { enabled: true },
            kb: KbMemory::default(),
        },
        tools: AgentToolConfig { bundles: vec![] },
        ui: AgentUiConfig {
            forms: FeatureFlag::default(),
            artifacts: ArtifactsConfig::default(),
        },
        extensions: HashMap::new(),
    }
}

pub fn orchestrator_agent() -> AgentArtifact {
    let mut agent = default_agent();
    agent.id = "orchestrator-agent".to_string();
    agent.metadata.title = "Orchestrator".to_string();
    agent.metadata.description = "System orchestrator for complex tasks.".to_string();
    agent
}

/// Creates the default knowledge base if it doesn't exist.
/// This should be called on application startup.
pub async fn ensure_default_knowledge_base(
    persistence: &dyn crate::uar::persistence::PersistenceLayer,
    config: Option<&crate::config::KnowledgeBaseConfig>,
) -> anyhow::Result<crate::uar::domain::knowledge::KnowledgeBase> {
    use crate::uar::domain::knowledge::{KbConfig, KnowledgeBase};
    use crate::uar::rag::chunking::ChunkingStrategy;

    const DEFAULT_KB_NAME: &str = "default";

    // Check if default KB already exists
    if let Some(existing) = persistence
        .get_knowledge_base_by_name(DEFAULT_KB_NAME)
        .await?
    {
        tracing::debug!("Default knowledge base already exists: {}", existing.id);
        return Ok(existing);
    }

    // Build config from provided config or use hardcoded defaults
    let kb_config = if let Some(cfg) = config {
        // Convert ChunkingConfig to ChunkingStrategy
        let chunk_strategy = match cfg.chunking.strategy.as_str() {
            "fixed" => ChunkingStrategy::FixedSize {
                size: cfg.chunking.chunk_size,
            },
            "recursive" => ChunkingStrategy::Recursive {
                size: cfg.chunking.chunk_size,
            },
            "token" => ChunkingStrategy::Token {
                tokens: cfg.chunking.chunk_size,
            },
            "sentence" => ChunkingStrategy::Sentence,
            "document" => ChunkingStrategy::Document,
            "semantic" => ChunkingStrategy::Semantic {
                threshold: cfg.chunking.semantic_threshold.unwrap_or(0.7),
            },
            _ => ChunkingStrategy::Recursive { size: 512 },
        };

        KbConfig {
            embedding_provider: cfg.embedding_provider.clone(),
            embedding_model: cfg.embedding_model.clone(),
            vector_dimensions: cfg.vector_dimensions,
            file_processor: cfg.file_processor.clone(),
            chunk_strategy,
        }
    } else {
        KbConfig::default()
    };

    let now = chrono::Utc::now().to_rfc3339();
    let kb = KnowledgeBase {
        id: uuid::Uuid::new_v4().to_string(),
        name: DEFAULT_KB_NAME.to_string(),
        description: Some("Default knowledge base for general documents".to_string()),
        config: kb_config,
        created_at: now.clone(),
        updated_at: now,
    };

    persistence.save_knowledge_base(&kb).await?;
    tracing::info!("Created default knowledge base: {} ({})", kb.name, kb.id);

    Ok(kb)
}
