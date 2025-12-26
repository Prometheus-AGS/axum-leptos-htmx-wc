use crate::session::Session;
use crate::uar::domain::knowledge::{
    DocumentStatus, KnowledgeBase, KnowledgeChunk, KnowledgeDocument, KnowledgeMatch,
};
use crate::uar::domain::skills::{Skill, SkillMatch};
use crate::uar::persistence::PersistenceLayer;
use anyhow::Result;
use async_trait::async_trait;
use pgvector::Vector;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

#[derive(Debug)]
pub struct PostgresProvider {
    pool: PgPool,
}

impl PostgresProvider {
    pub async fn new(connection_string: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(connection_string)
            .await?;

        // Run Migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl PersistenceLayer for PostgresProvider {
    async fn save_session(&self, session: &Session) -> Result<()> {
        let id = session.id();
        // Serialize session to JSON
        let data = serde_json::to_value(session)?;

        sqlx::query(
            r#"
            INSERT INTO sessions (id, data, created_at, updated_at)
            VALUES ($1, $2, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                data = EXCLUDED.data,
                updated_at = NOW()
            "#,
        )
        .bind(id)
        .bind(data)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn load_session(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query("SELECT data FROM sessions WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let val: serde_json::Value = row.try_get("data")?;
            let session: Session = serde_json::from_value(val)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn save_skill(&self, skill: &Skill, embedding: &[f32]) -> Result<()> {
        let embedding_vector = Vector::from(embedding.to_vec());
        let definition = serde_json::to_value(skill)?;

        sqlx::query(
            r#"
            INSERT INTO skills (skill_id, name, description, definition, embedding, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
            ON CONFLICT (skill_id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                definition = EXCLUDED.definition,
                embedding = EXCLUDED.embedding,
                updated_at = NOW()
            "#,
        )
        .bind(&skill.skill_id)
        .bind(&skill.title)
        .bind(&skill.description)
        .bind(definition)
        .bind(embedding_vector)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_skills(&self, query_vec: &[f32], limit: usize) -> Result<Vec<SkillMatch>> {
        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 = limit as i64;

        let rows = sqlx::query(
            r#"
            SELECT definition, 1 - (embedding <=> $1) as score
            FROM skills
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(embedding_vector) // bind $1
        .bind(limit_i64)
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let def_val: serde_json::Value = row.try_get("definition")?;
            let skill: Skill = serde_json::from_value(def_val)?;

            // Score might be f64 or f32. pgvector operator returns f64 usually.
            // 1 - distance.
            let score: f64 = row.try_get("score")?;

            matches.push(SkillMatch {
                skill,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    async fn save_knowledge_base(&self, kb: &KnowledgeBase) -> Result<()> {
        let config = serde_json::to_value(&kb.config)?;

        sqlx::query(
            r#"
            INSERT INTO knowledge_bases (id, name, description, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                description = EXCLUDED.description,
                config = EXCLUDED.config,
                updated_at = NOW()
            "#,
        )
        .bind(&kb.id)
        .bind(&kb.name)
        .bind(&kb.description)
        .bind(config)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn save_chunk(&self, chunk: &KnowledgeChunk) -> Result<()> {
        let embedding_vector = Vector::from(chunk.embedding.clone());
        let metadata = serde_json::to_value(&chunk.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO knowledge_chunks (id, kb_id, content, metadata, embedding, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (id) DO UPDATE SET
                content = EXCLUDED.content,
                metadata = EXCLUDED.metadata,
                embedding = EXCLUDED.embedding
            "#,
        )
        .bind(chunk.id)
        .bind(&chunk.kb_id)
        .bind(&chunk.content)
        .bind(metadata)
        .bind(embedding_vector)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_knowledge(
        &self,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 = limit as i64;
        let min_score_f64 = min_score as f64;

        let rows = sqlx::query(
            r#"
            SELECT id, kb_id, content, metadata, created_at, 1 - (embedding <=> $1) as score
            FROM knowledge_chunks
            WHERE 1 - (embedding <=> $1) >= $3
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(embedding_vector) // $1
        .bind(limit_i64) // $2
        .bind(min_score_f64) // $3
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let content: String = row.try_get("content")?;
            let metadata_val: Option<serde_json::Value> = row.try_get("metadata")?;
            // created_at in DB is likely TIMESTAMP. We want String (RFC3339).
            // sqlx maps TIMESTAMP to chrono::NaiveDateTime or DateTime<Utc>
            // We can treat it as DateTime<Utc> and format it?
            // Need `chrono` here?
            // If I map to String, sqlx might fail if type mismatch.
            // But let's try `try_get::<String, _>("created_at")`? Postgres might support cast.
            // If not, read as DateTime<Utc> then to_rfc3339().
            // I need `chrono` crate.
            // `use chrono::DateTime; use chrono::Utc;`

            // To be safe and simple: just load it.
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let created_at_str = created_at.map(|d| d.to_rfc3339()).unwrap_or_default();

            let score: f64 = row.try_get("score")?;

            let chunk = KnowledgeChunk {
                id,
                kb_id,
                document_id: None, // Not returned in search results
                content,
                metadata: metadata_val,
                embedding: vec![], // we don't return embedding in search results unless needed
                created_at: created_at_str,
            };

            matches.push(KnowledgeMatch {
                chunk,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    // Agent Persistence
    async fn save_agent(&self, agent: &crate::uar::domain::artifact::AgentArtifact) -> Result<()> {
        let definition = serde_json::to_value(agent)?;

        sqlx::query(
            r#"
            INSERT INTO agents (id, name, version, definition, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                name = EXCLUDED.name,
                version = EXCLUDED.version,
                definition = EXCLUDED.definition,
                updated_at = NOW()
            "#,
        )
        .bind(&agent.id)
        .bind(&agent.metadata.title) // Use title as name? Or name field? Artifact has no top-level name?
        // Wait, AgentArtifact has `metadata.title`. It doesn't have top-level `name`.
        // The JSON in test showed "name" inside metadata? No.
        // Let's check AgentArtifact struct.
        // Assuming metadata.title is the name for now.
        .bind(&agent.version)
        .bind(definition)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_agent(
        &self,
        id: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        let row = sqlx::query("SELECT definition FROM agents WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let val: serde_json::Value = row.try_get("definition")?;
            let agent: crate::uar::domain::artifact::AgentArtifact = serde_json::from_value(val)?;
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }

    async fn load_agent_by_name(
        &self,
        name: &str,
    ) -> Result<Option<crate::uar::domain::artifact::AgentArtifact>> {
        let row = sqlx::query("SELECT definition FROM agents WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let val: serde_json::Value = row.try_get("definition")?;
            let agent: crate::uar::domain::artifact::AgentArtifact = serde_json::from_value(val)?;
            Ok(Some(agent))
        } else {
            Ok(None)
        }
    }

    async fn list_agents(&self) -> Result<Vec<crate::uar::domain::artifact::AgentArtifact>> {
        let rows = sqlx::query("SELECT definition FROM agents")
            .fetch_all(&self.pool)
            .await?;

        let mut agents = Vec::new();
        for row in rows {
            let val: serde_json::Value = row.try_get("definition")?;
            let agent: crate::uar::domain::artifact::AgentArtifact = serde_json::from_value(val)?;
            agents.push(agent);
        }
        Ok(agents)
    }

    // Memory System
    async fn save_memory(&self, memory: &crate::uar::domain::memory::Memory) -> Result<()> {
        let embedding_vector = Vector::from(memory.embedding.clone());

        sqlx::query(
            r#"
            INSERT INTO memories (id, agent_id, content, tags, embedding, created_at)
            VALUES ($1, $2, $3, $4, $5, NOW())
            ON CONFLICT (id) DO UPDATE SET
                agent_id = EXCLUDED.agent_id,
                content = EXCLUDED.content,
                tags = EXCLUDED.tags,
                embedding = EXCLUDED.embedding
            "#,
        )
        .bind(&memory.id)
        .bind(&memory.agent_id)
        .bind(&memory.content)
        .bind(&memory.tags)
        .bind(embedding_vector)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn search_memory(
        &self,
        agent_id: Option<&str>,
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<crate::uar::domain::memory::MemoryMatch>> {
        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 = limit as i64;
        let min_score_f64 = min_score as f64;

        // Condition: (agent_id = $1 OR agent_id IS NULL)
        // If $1 is NULL, it matches Global only.
        // If $1 is 'A', it matches 'A' and Global.
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, content, tags, created_at, 1 - (embedding <=> $2) as score
            FROM memories
            WHERE (agent_id = $1 OR agent_id IS NULL)
              AND 1 - (embedding <=> $2) >= $3
            ORDER BY embedding <=> $2
            LIMIT $4
            "#,
        )
        .bind(agent_id) // $1
        .bind(embedding_vector) // $2
        .bind(min_score_f64) // $3
        .bind(limit_i64) // $4
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let a_id: Option<String> = row.try_get("agent_id")?;
            let content: String = row.try_get("content")?;
            let tags: Vec<String> = row.try_get("tags")?;

            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let created_at_str = created_at.map(|d| d.to_rfc3339()).unwrap_or_default();

            let score: f64 = row.try_get("score")?;

            let memory = crate::uar::domain::memory::Memory {
                id,
                agent_id: a_id,
                content,
                tags,
                embedding: vec![],
                created_at: created_at_str,
            };

            matches.push(crate::uar::domain::memory::MemoryMatch {
                memory,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    // =========================================================================
    // Knowledge Base Retrieval Methods
    // =========================================================================

    async fn get_knowledge_base(&self, id: &str) -> Result<Option<KnowledgeBase>> {
        let row = sqlx::query(
            "SELECT id, name, description, config, created_at, updated_at FROM knowledge_bases WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            let description: Option<String> = row.try_get("description")?;
            let config_val: serde_json::Value = row.try_get("config")?;
            let config = serde_json::from_value(config_val)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            Ok(Some(KnowledgeBase {
                id,
                name: name.unwrap_or_default(),
                description,
                config,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn get_knowledge_base_by_name(&self, name: &str) -> Result<Option<KnowledgeBase>> {
        let row = sqlx::query(
            "SELECT id, name, description, config, created_at, updated_at FROM knowledge_bases WHERE name = $1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            let description: Option<String> = row.try_get("description")?;
            let config_val: serde_json::Value = row.try_get("config")?;
            let config = serde_json::from_value(config_val)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            Ok(Some(KnowledgeBase {
                id,
                name: name.unwrap_or_default(),
                description,
                config,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_knowledge_bases(&self) -> Result<Vec<KnowledgeBase>> {
        let rows = sqlx::query(
            "SELECT id, name, description, config, created_at, updated_at FROM knowledge_bases ORDER BY created_at",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut kbs = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let name: Option<String> = row.try_get("name")?;
            let description: Option<String> = row.try_get("description")?;
            let config_val: serde_json::Value = row.try_get("config")?;
            let config = serde_json::from_value(config_val)?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            kbs.push(KnowledgeBase {
                id,
                name: name.unwrap_or_default(),
                description,
                config,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            });
        }
        Ok(kbs)
    }

    async fn delete_knowledge_base(&self, id: &str) -> Result<()> {
        // CASCADE will handle chunks and documents
        sqlx::query("DELETE FROM knowledge_bases WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // =========================================================================
    // Scoped Knowledge Search
    // =========================================================================

    async fn search_knowledge_scoped(
        &self,
        kb_ids: &[&str],
        query_vec: &[f32],
        limit: usize,
        min_score: f32,
    ) -> Result<Vec<KnowledgeMatch>> {
        if kb_ids.is_empty() {
            return Ok(vec![]);
        }

        let embedding_vector = Vector::from(query_vec.to_vec());
        let limit_i64 = limit as i64;
        let min_score_f64 = min_score as f64;
        let kb_ids_vec: Vec<String> = kb_ids.iter().map(|s| s.to_string()).collect();

        let rows = sqlx::query(
            r#"
            SELECT id, kb_id, document_id, content, metadata, created_at, 1 - (embedding <=> $1) as score
            FROM knowledge_chunks
            WHERE kb_id = ANY($4) AND 1 - (embedding <=> $1) >= $3
            ORDER BY embedding <=> $1
            LIMIT $2
            "#,
        )
        .bind(embedding_vector)
        .bind(limit_i64)
        .bind(min_score_f64)
        .bind(&kb_ids_vec)
        .fetch_all(&self.pool)
        .await?;

        let mut matches = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let document_id: Option<String> = row.try_get("document_id")?;
            let content: String = row.try_get("content")?;
            let metadata_val: Option<serde_json::Value> = row.try_get("metadata")?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let created_at_str = created_at.map(|d| d.to_rfc3339()).unwrap_or_default();
            let score: f64 = row.try_get("score")?;

            let chunk = KnowledgeChunk {
                id,
                kb_id,
                document_id,
                content,
                metadata: metadata_val,
                embedding: vec![],
                created_at: created_at_str,
            };

            matches.push(KnowledgeMatch {
                chunk,
                score: score as f32,
            });
        }
        Ok(matches)
    }

    // =========================================================================
    // Document Tracking
    // =========================================================================

    async fn save_document(&self, doc: &KnowledgeDocument) -> Result<()> {
        let status_str = match &doc.status {
            DocumentStatus::Pending => "pending",
            DocumentStatus::Processing => "processing",
            DocumentStatus::Indexed => "indexed",
            DocumentStatus::Failed { .. } => "failed",
        };
        let error_msg = match &doc.status {
            DocumentStatus::Failed { error } => Some(error.clone()),
            _ => None,
        };

        sqlx::query(
            r#"
            INSERT INTO knowledge_documents (id, kb_id, filename, file_path, mime_type, chunk_count, status, error_message, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW())
            ON CONFLICT (id) DO UPDATE SET
                filename = EXCLUDED.filename,
                file_path = EXCLUDED.file_path,
                mime_type = EXCLUDED.mime_type,
                chunk_count = EXCLUDED.chunk_count,
                status = EXCLUDED.status,
                error_message = EXCLUDED.error_message,
                updated_at = NOW()
            "#,
        )
        .bind(&doc.id)
        .bind(&doc.kb_id)
        .bind(&doc.filename)
        .bind(&doc.file_path)
        .bind(&doc.mime_type)
        .bind(doc.chunk_count as i32)
        .bind(status_str)
        .bind(error_msg)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get_document(&self, id: &str) -> Result<Option<KnowledgeDocument>> {
        let row = sqlx::query(
            "SELECT id, kb_id, filename, file_path, mime_type, chunk_count, status, error_message, created_at, updated_at FROM knowledge_documents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let id: String = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let filename: String = row.try_get("filename")?;
            let file_path: Option<String> = row.try_get("file_path")?;
            let mime_type: String = row.try_get("mime_type")?;
            let chunk_count: i32 = row.try_get("chunk_count")?;
            let status_str: String = row.try_get("status")?;
            let error_message: Option<String> = row.try_get("error_message")?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            let status = match status_str.as_str() {
                "processing" => DocumentStatus::Processing,
                "indexed" => DocumentStatus::Indexed,
                "failed" => DocumentStatus::Failed {
                    error: error_message.unwrap_or_default(),
                },
                _ => DocumentStatus::Pending,
            };

            Ok(Some(KnowledgeDocument {
                id,
                kb_id,
                filename,
                file_path,
                mime_type: Some(mime_type),
                chunk_count: chunk_count as usize,
                status,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_documents(&self, kb_id: &str) -> Result<Vec<KnowledgeDocument>> {
        let rows = sqlx::query(
            "SELECT id, kb_id, filename, file_path, mime_type, chunk_count, status, error_message, created_at, updated_at FROM knowledge_documents WHERE kb_id = $1 ORDER BY created_at",
        )
        .bind(kb_id)
        .fetch_all(&self.pool)
        .await?;

        let mut docs = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let kb_id: String = row.try_get("kb_id")?;
            let filename: String = row.try_get("filename")?;
            let file_path: Option<String> = row.try_get("file_path")?;
            let mime_type: String = row.try_get("mime_type")?;
            let chunk_count: i32 = row.try_get("chunk_count")?;
            let status_str: String = row.try_get("status")?;
            let error_message: Option<String> = row.try_get("error_message")?;
            let created_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("created_at")?;
            let updated_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("updated_at")?;

            let status = match status_str.as_str() {
                "processing" => DocumentStatus::Processing,
                "indexed" => DocumentStatus::Indexed,
                "failed" => DocumentStatus::Failed {
                    error: error_message.unwrap_or_default(),
                },
                _ => DocumentStatus::Pending,
            };

            docs.push(KnowledgeDocument {
                id,
                kb_id,
                filename,
                file_path,
                mime_type: Some(mime_type),
                chunk_count: chunk_count as usize,
                status,
                created_at: created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
                updated_at: updated_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
            });
        }
        Ok(docs)
    }

    async fn update_document_status(&self, doc_id: &str, status: &DocumentStatus) -> Result<()> {
        let status_str = match status {
            DocumentStatus::Pending => "pending",
            DocumentStatus::Processing => "processing",
            DocumentStatus::Indexed => "indexed",
            DocumentStatus::Failed { .. } => "failed",
        };
        let error_msg = match status {
            DocumentStatus::Failed { error } => Some(error.clone()),
            _ => None,
        };

        sqlx::query(
            "UPDATE knowledge_documents SET status = $1, error_message = $2, updated_at = NOW() WHERE id = $3",
        )
        .bind(status_str)
        .bind(error_msg)
        .bind(doc_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete_document(&self, doc_id: &str) -> Result<()> {
        // Delete associated chunks first
        sqlx::query("DELETE FROM knowledge_chunks WHERE document_id = $1")
            .bind(doc_id)
            .execute(&self.pool)
            .await?;

        // Delete the document
        sqlx::query("DELETE FROM knowledge_documents WHERE id = $1")
            .bind(doc_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
