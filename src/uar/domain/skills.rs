use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub skill_id: String,
    pub version: String,
    pub title: String,
    pub description: String,
    pub triggers: SkillTriggers,
    pub prompt_overlay: String,
    #[serde(default)]
    pub preferred_tools: Vec<String>,
    #[serde(default, skip)]
    pub mcp_config: Option<crate::mcp::config::McpConfig>,
    #[serde(default)]
    pub constraints: SkillConstraints,
}

/// Represents the YAML frontmatter of a SKILL.md file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub triggers: SkillTriggers,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillTriggers {
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub semantic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillConstraints {
    #[serde(default)]
    pub deny_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMatch {
    pub skill: Skill,
    pub score: f32,
}
