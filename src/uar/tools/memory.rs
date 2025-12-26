use crate::mcp::registry::NativeTool;
use crate::uar::domain::memory::Memory;
use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::matching::VectorMatcher;
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug)]
pub struct MemorySaveTool {
    persistence: Arc<dyn PersistenceLayer>,
    vector_matcher: Arc<VectorMatcher>,
}

impl MemorySaveTool {
    pub fn new(persistence: Arc<dyn PersistenceLayer>, vector_matcher: Arc<VectorMatcher>) -> Self {
        Self {
            persistence,
            vector_matcher,
        }
    }
}

#[async_trait]
impl NativeTool for MemorySaveTool {
    fn name(&self) -> &str {
        "memory_save"
    }

    fn description(&self) -> &str {
        "Save a piece of information to long-term memory. Use to remember facts, user preferences, or important context."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "The information content to memorize."
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Optional tags for categorization."
                },
                "agent_id": {
                    "type": "string",
                    "description": "Optional ID of the agent owning this memory. Omit for global memory."
                }
            },
            "required": ["content"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let content = args["content"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;
        let tags: Vec<String> = args["tags"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let agent_id = args["agent_id"].as_str().map(|s| s.to_string());

        let embeddings = self
            .vector_matcher
            .embed_batch(vec![content.to_string()])
            .await?;
        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))?;

        let memory = Memory {
            id: Uuid::new_v4().to_string(),
            agent_id,
            content: content.to_string(),
            tags,
            embedding,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        self.persistence.save_memory(&memory).await?;

        Ok(json!({
            "status": "success",
            "memory_id": memory.id
        }))
    }
}

#[derive(Debug)]
pub struct MemoryRecallTool {
    persistence: Arc<dyn PersistenceLayer>,
    vector_matcher: Arc<VectorMatcher>,
}

impl MemoryRecallTool {
    pub fn new(persistence: Arc<dyn PersistenceLayer>, vector_matcher: Arc<VectorMatcher>) -> Self {
        Self {
            persistence,
            vector_matcher,
        }
    }
}

#[async_trait]
impl NativeTool for MemoryRecallTool {
    fn name(&self) -> &str {
        "memory_recall"
    }

    fn description(&self) -> &str {
        "Search long-term memory for relevant information."
    }

    fn schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Semantic search query."
                },
                "agent_id": {
                    "type": "string",
                    "description": "Optional. If provided, searches Agent's memory + Global. If omitted, searches Global only."
                },
                "limit": {
                    "type": "integer",
                    "description": "Max results (default 5)."
                }
            },
            "required": ["query"]
        })
    }

    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value> {
        let query = args["query"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing query"))?;
        let agent_id = args["agent_id"].as_str(); // Option<&str>
        let limit = args["limit"].as_u64().unwrap_or(5) as usize;

        let embeddings = self
            .vector_matcher
            .embed_batch(vec![query.to_string()])
            .await?;
        let embedding = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))?;

        let matches = self
            .persistence
            .search_memory(agent_id, &embedding, limit, 0.0)
            .await?;

        let results: Vec<serde_json::Value> = matches
            .into_iter()
            .map(|m| {
                json!({
                    "content": m.memory.content,
                    "score": m.score,
                    "tags": m.memory.tags,
                    "type": if m.memory.agent_id.is_some() { "agent" } else { "global" }
                })
            })
            .collect();

        Ok(json!(results))
    }
}
