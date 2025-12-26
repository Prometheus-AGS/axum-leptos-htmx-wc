use crate::uar::domain::skills::{Skill, SkillManifest};
use crate::uar::persistence::PersistenceLayer;
use crate::uar::runtime::matching::vector::VectorMatcher;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
    persistence: Option<Arc<dyn PersistenceLayer>>,
    vector_matcher: Option<Arc<VectorMatcher>>,
}

// Manual Debug implementation to skip generic/Arc fields if needed, or just derive if they implement Debug
impl std::fmt::Debug for SkillRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillRegistry")
            .field("skills_count", &self.skills.len())
            .field("persistence", &self.persistence.is_some())
            .field("vector_matcher", &self.vector_matcher.is_some())
            .finish()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl SkillRegistry {
    pub fn new(
        persistence: Option<Arc<dyn PersistenceLayer>>,
        vector_matcher: Option<Arc<VectorMatcher>>,
    ) -> Self {
        Self {
            skills: HashMap::new(),
            persistence,
            vector_matcher,
        }
    }

    /// Loads skills from a directory recursively.
    /// Looks for `SKILL.md` files.
    pub async fn load_from_dir(&mut self, path: &str) -> anyhow::Result<()> {
        let path = Path::new(path);
        if !path.exists() {
            warn!("Skills directory not found: {:?}", path);
            return Ok(());
        }

        let mut entries = fs::read_dir(path).await?;
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_dir() {
                // Check for SKILL.md inside
                let skill_file = path.join("SKILL.md");
                if skill_file.exists() {
                    if let Err(e) = self.load_skill_package(&skill_file).await {
                        error!("Failed to load skill from {:?}: {:?}", skill_file, e);
                    }
                } else {
                    // Recurse
                    let _ = Box::pin(self.load_from_dir(path.to_str().unwrap())).await;
                }
            }
        }
        Ok(())
    }

    async fn load_skill_package(&mut self, path: &Path) -> anyhow::Result<()> {
        let content = fs::read_to_string(path).await?;
        let (manifest, overlay) = Self::parse_skill_file(&content)?;

        let skill_id = manifest.name.to_lowercase().replace(' ', "-");

        // Check for mcp.json in the same directory
        let mut mcp_config = None;
        if let Some(parent) = path.parent() {
            let mcp_path = parent.join("mcp.json");
            if mcp_path.exists() {
                match crate::mcp::config::load_mcp_config(&mcp_path) {
                    Ok(mut cfg) => {
                        info!("Loaded mcp.json for skill: {}", skill_id);
                        // Namespace the servers to avoid collisions
                        let new_servers: HashMap<_, _> = cfg
                            .mcp_servers
                            .drain()
                            .map(|(k, v)| (format!("{}__{}", skill_id, k), v))
                            .collect();
                        cfg.mcp_servers = new_servers;
                        mcp_config = Some(cfg);
                    }
                    Err(e) => {
                        error!("Failed to load mcp.json for skill {}: {:?}", skill_id, e);
                    }
                }
            }
        }

        let skill = Skill {
            skill_id: skill_id.clone(),
            version: manifest.version,
            title: manifest.name,
            description: manifest.description,
            triggers: manifest.triggers,
            prompt_overlay: overlay,
            preferred_tools: manifest.tools,
            mcp_config,
            constraints: Default::default(),
        };

        info!("Loaded skill: {}", skill.title);
        self.register(skill).await;
        Ok(())
    }

    fn parse_skill_file(content: &str) -> anyhow::Result<(SkillManifest, String)> {
        // Simple frontmatter parser
        if !content.starts_with("---") {
            return Err(anyhow::anyhow!("Missing frontmatter"));
        }

        let parts: Vec<&str> = content.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(anyhow::anyhow!("Invalid frontmatter format"));
        }

        let yaml = parts[1];
        let body = parts[2].trim().to_string();

        let manifest: SkillManifest = serde_yaml::from_str(yaml)?;
        Ok((manifest, body))
    }

    pub async fn register(&mut self, skill: Skill) {
        // Save to Persistence if available
        if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
            // Generate embedding for "Title: Description"
            let text = format!("{}: {}", skill.title, skill.description);
            match vm.embed_batch(vec![text]).await {
                Ok(embeddings) => {
                    if let Some(emb) = embeddings.first() {
                        if let Err(e) = db.save_skill(&skill, emb).await {
                            error!("Failed to persist skill {}: {:?}", skill.skill_id, e);
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to generate embedding for skill {}: {:?}",
                        skill.skill_id, e
                    );
                }
            }
        }

        self.skills.insert(skill.skill_id.clone(), skill);
    }

    pub fn get(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }

    pub fn list(&self) -> Vec<Skill> {
        self.skills.values().cloned().collect()
    }

    pub async fn find_matches(&self, query: &str) -> Vec<Skill> {
        // If persistence available, use vector search
        if let (Some(db), Some(vm)) = (&self.persistence, &self.vector_matcher) {
            match vm.embed_batch(vec![query.to_string()]).await {
                Ok(embeddings) => {
                    if let Some(q_vec) = embeddings.first() {
                        match db.search_skills(q_vec, 5).await {
                            // Limit 5
                            Ok(matches) => {
                                return matches.into_iter().map(|m| m.skill).collect();
                            }
                            Err(e) => {
                                error!("Skill search failed: {:?}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Query embedding failed: {:?}", e);
                }
            }
        }

        // Fallback to in-memory keyword match
        self.skills
            .values()
            .filter(|s| {
                s.title.to_lowercase().contains(&query.to_lowercase())
                    || s.description.to_lowercase().contains(&query.to_lowercase())
                    || s.triggers
                        .keywords
                        .iter()
                        .any(|k| k.to_lowercase().contains(&query.to_lowercase()))
            })
            .cloned()
            .collect()
    }
}
