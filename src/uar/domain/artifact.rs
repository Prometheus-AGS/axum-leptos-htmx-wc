use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentArtifact {
    pub version: String,
    pub kind: String, // must be "agent"
    pub id: String,
    pub metadata: AgentMetadata,
    pub runtime: AgentRuntimeConfig,
    pub policy: AgentPolicy,
    pub schemas: AgentSchemas,
    pub prompt: AgentPrompt,
    pub memory: AgentMemoryConfig,
    pub tools: AgentToolConfig,
    pub ui: AgentUiConfig,
    #[serde(default)]
    pub extensions: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRuntimeConfig {
    pub entry: String,
    #[serde(default)]
    pub protocols: HashMap<String, ProtocolConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPolicy {
    pub provider: ProviderPolicy,
    pub tools: ToolPolicy,
    pub skills: SkillPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPolicy {
    pub default: ProviderSelection,
    #[serde(default)]
    pub fallbacks: Vec<ProviderSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSelection {
    pub provider: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicy {
    #[serde(default)]
    pub allow: Vec<String>,
    #[serde(default)]
    pub deny: Vec<String>,
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,
}

fn default_max_concurrent() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPolicy {
    #[serde(default)]
    pub prefer: Vec<String>,
    #[serde(default = "default_max_active")]
    pub max_active: u32,
}

fn default_max_active() -> u32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSchemas {
    #[serde(default)]
    pub inputs: Option<serde_json::Value>,
    #[serde(default)]
    pub outputs: Option<serde_json::Value>,
    #[serde(default)]
    pub state: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPrompt {
    pub system: String,
    #[serde(default)]
    pub instructions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMemoryConfig {
    #[serde(default)]
    pub conversation: ConversationMemory,
    #[serde(default)]
    pub kb: KbMemory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMemory {
    #[serde(default)]
    pub enabled: bool,
}

impl Default for ConversationMemory {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KbMemory {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub collections: Vec<String>,
    #[serde(default)]
    pub citation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolConfig {
    #[serde(default)]
    pub bundles: Vec<ToolBundle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBundle {
    pub id: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUiConfig {
    #[serde(default)]
    pub forms: FeatureFlag,
    #[serde(default)]
    pub artifacts: ArtifactsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeatureFlag {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactsConfig {
    pub enabled: bool,
    #[serde(default)]
    pub preferred_types: Vec<String>,
}
