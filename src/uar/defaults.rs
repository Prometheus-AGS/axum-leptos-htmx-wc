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
