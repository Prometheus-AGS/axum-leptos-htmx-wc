use crate::uar::domain::matching::{MatchReason, SkillMatch, SkillMatcher};
use crate::uar::runtime::skills::SkillRegistry;
use anyhow::{Context, Result};
use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

pub struct VectorMatcher {
    model: Arc<Mutex<Option<TextEmbedding>>>,
    // Cache: skill_id -> embedding
    embeddings: Arc<Mutex<Vec<(String, Vec<f32>)>>>,
    threshold: f32,
}

impl std::fmt::Debug for VectorMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorMatcher")
            .field("model_loaded", &"Dynamic")
            .field("embeddings_count", &"Dynamic")
            .field("threshold", &self.threshold)
            .finish()
    }
}

impl VectorMatcher {
    pub fn new(threshold: f32) -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            embeddings: Arc::new(Mutex::new(Vec::new())),
            threshold,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut model_guard = self.model.lock().await;
        if model_guard.is_none() {
            info!("Initializing fastembed model (BG-Small-En-V1.5)...");
            let mut options = InitOptions::new(EmbeddingModel::BGESmallENV15);
            options.show_download_progress = true;

            let model = TextEmbedding::try_new(options)?;
            *model_guard = Some(model);
        }
        Ok(())
    }

    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let mut model_guard = self.model.lock().await;
        if let Some(_) = &mut *model_guard {
            let mut owned_model = model_guard
                .take()
                .context("Model unexpectedly None during embed_batch")?;

            let (embeddings_res, returned_model) = tokio::task::spawn_blocking(move || {
                let res = owned_model.embed(texts, None);
                (res, owned_model)
            })
            .await?;

            *model_guard = Some(returned_model);
            embeddings_res.map_err(|e| anyhow::anyhow!(e))
        } else {
            // Try to init? Or fail?
            // Since init is async and we hold lock? NO, we just checked lock.
            // We can drop lock and init?
            // For safety, error out. Usage should ensure init.
            Err(anyhow::anyhow!("VectorMatcher not initialized"))
        }
    }

    pub async fn index_skills(&self, registry: &SkillRegistry) -> Result<()> {
        let skills = registry.list();
        let mut texts = Vec::new();
        let mut ids = Vec::new();

        for skill in &skills {
            // Embed title + description
            let text = format!("{}: {}", skill.title, skill.description);
            texts.push(text);
            ids.push(skill.skill_id.clone());
        }

        if texts.is_empty() {
            return Ok(());
        }

        let mut model_guard = self.model.lock().await; // Lock mutably!
        if let Some(_) = &mut *model_guard {
            info!("Generating embeddings for {} skills...", texts.len());
            // Move model to closure? No, `model` is behind Mutex guard, not Send/static easily?
            // Actually `TextEmbedding` is Send/Sync.
            // But we have `&mut` to it from the guard.
            // We can't move the guard into spawn_blocking easily if we want to return it.
            // But we can clone the model if it was Arc... it's not.
            // FastEmbed `TextEmbedding` might be heavy to clone?
            // "The Model passed to `init` is cheap to clone" - checking fastembed docs...
            // Actually, we don't have a clone.
            // If we block here, we block the `await`.
            // Let's rely on the fact that this is a test.
            // Maybe just allow blocking? `tokio::test` usually handles some blocking but if it starves...

            // Let's try to just run it. The hang might be `InitOptions` downloading files prompting...
            // "show_download_progress: true" might be messing with stdout capturing in test?
            let mut owned_model = model_guard
                .take()
                .context("Model unexpectedly None during indexing")?;
            let (embeddings_res, returned_model) = tokio::task::spawn_blocking(move || {
                let res = owned_model.embed(texts, None);
                (res, owned_model)
            })
            .await?; // Await the spawn_blocking

            *model_guard = Some(returned_model); // Put the model back

            let embeddings = embeddings_res?; // Unwrap result

            let mut cache = self.embeddings.lock().await;
            cache.clear();
            for (i, emb) in embeddings.into_iter().enumerate() {
                cache.push((ids[i].clone(), emb));
            }
            info!("Skill vector index built.");
        } else {
            warn!("Model not initialized, skipping indexing.");
        }

        Ok(())
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl SkillMatcher for VectorMatcher {
    async fn match_skills(&self, query: &str, registry: &SkillRegistry) -> Result<Vec<SkillMatch>> {
        // Ensure initialized (lazy logic or expect init called?)
        // Ideally should be initialized at startup.

        let mut model_guard = self.model.lock().await; // Lock mutably
        let query_embedding = if let Some(_) = &mut *model_guard {
            let mut owned_model = model_guard.take().context("Model unexpected None")?;
            let query_owned = query.to_string();

            info!("Embedding query: {}", query);
            let (embeddings_res, returned_model) = tokio::task::spawn_blocking(move || {
                let res = owned_model.embed(vec![query_owned], None);
                (res, owned_model)
            })
            .await?;
            info!("Query embedding generated");

            *model_guard = Some(returned_model);
            let embeddings = embeddings_res?;

            embeddings
                .into_iter()
                .next()
                .context("No embedding generated")?
        } else {
            warn!("VectorMatcher not initialized");
            return Ok(vec![]);
        };
        drop(model_guard); // Release lock

        // Check if indexing is needed
        {
            let cache = self.embeddings.lock().await;
            if cache.is_empty() {
                drop(cache);
                self.index_skills(registry).await?;
            }
        }

        let cache = self.embeddings.lock().await;

        let mut matches = Vec::new();

        for (skill_id, emb) in cache.iter() {
            let score = Self::cosine_similarity(&query_embedding, emb);
            if score >= self.threshold {
                if let Some(skill) = registry.get(skill_id) {
                    matches.push(SkillMatch {
                        skill_id: skill_id.clone(),
                        score,
                        reason: MatchReason::VectorSimilarity(score),
                        skill: skill.clone(),
                    });
                }
            }
        }

        // Sort by score desc
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(matches)
    }
}
