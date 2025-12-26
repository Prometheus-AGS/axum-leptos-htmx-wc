use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: String,
    pub agent_id: Option<String>, // None = Global
    pub content: String,
    pub tags: Vec<String>,
    #[serde(skip)]
    pub embedding: Vec<f32>,
    pub created_at: String, // RFC3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMatch {
    pub memory: Memory,
    pub score: f32,
}
