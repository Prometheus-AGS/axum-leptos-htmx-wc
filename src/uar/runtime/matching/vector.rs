use crate::uar::domain::matching::{MatchReason, SkillMatch, SkillMatcher};
use crate::uar::runtime::skills::SkillRegistry;
use anyhow::{Context, Result};
use async_trait::async_trait;
use burn::backend::NdArray; // Default backend
use burn::tensor::backend::Backend;
use burn::tensor::{Shape, Tensor, TensorData};
use std::sync::Arc;
use tokenizers::Tokenizer;
use tokio::sync::Mutex;
use tracing::{info, warn};

// Import the generated model (conditionally compiled)
#[cfg(feature = "model-build")]
use super::burn_model::model;

#[cfg(not(feature = "model-build"))]
pub mod model {
    use burn::module::Module;
    use burn::tensor::backend::Backend;

    #[derive(Module, Debug)]
    pub struct Model<B: Backend> {
        _phantom: std::marker::PhantomData<B>,
    }

    impl<B: Backend> Model<B> {
        pub fn new() -> Self {
            Self {
                _phantom: std::marker::PhantomData,
            }
        }
    }

    impl<B: Backend> Default for Model<B> {
        fn default() -> Self {
            Self::new()
        }
    }
}

// We define our backend. NdArray is pure rust CPU.
// For WebGPU or other backends, this type alias can be changed or made generic.
type B = NdArray;
type EmbeddingCache = Vec<(String, Vec<f32>)>;

pub struct VectorMatcher {
    model: Arc<Mutex<Option<model::Model<B>>>>,
    tokenizer: Arc<Mutex<Option<Tokenizer>>>,
    embeddings: Arc<Mutex<EmbeddingCache>>,
    threshold: f32,
}

impl std::fmt::Debug for VectorMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VectorMatcher")
            .field("backend", &"NdArray")
            .field("threshold", &self.threshold)
            .finish()
    }
}

impl VectorMatcher {
    pub fn new(threshold: f32) -> Self {
        Self {
            model: Arc::new(Mutex::new(None)),
            tokenizer: Arc::new(Mutex::new(None)),
            embeddings: Arc::new(Mutex::new(Vec::new())),
            threshold,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        let mut model_guard = self.model.lock().await;
        if model_guard.is_none() {
            info!("Initializing Burn model (BG-Small-En-V1.5)...");
            // Standard burn-import generated model constructor
            // Assuming no weights in a separate file (embedded in binary)
            let _device = <B as Backend>::Device::default();
            let model = model::Model::<B>::default();
            *model_guard = Some(model);
        }

        let mut tok_guard = self.tokenizer.lock().await;
        if tok_guard.is_none() {
            info!("Initializing Tokenizer...");
            // Try to load from the downloaded file
            let path = std::path::Path::new("src/uar/runtime/matching/models/tokenizer.json");
            let tokenizer = if path.exists() {
                Tokenizer::from_file(path).map_err(|e| anyhow::anyhow!(e))?
            } else {
                return Err(anyhow::anyhow!("Tokenizer file not found at {}", path.display()));
            };

            *tok_guard = Some(tokenizer);
        }

        Ok(())
    }

    pub async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>> {
        let model_guard = self.model.lock().await;
        let tok_guard = self.tokenizer.lock().await;

        if let (Some(_model), Some(tokenizer)) = (&*model_guard, &*tok_guard) {
            // 1. Tokenize
            let encoding = tokenizer
                .encode_batch(texts.clone(), true)
                .map_err(|e| anyhow::anyhow!("Tokenization failed: {e}"))?;

            let batch_size = texts.len();
            if batch_size == 0 {
                return Ok(vec![]);
            }

            // Find max length and pad/truncate to ensure consistent tensor shapes
            let max_len = encoding
                .iter()
                .map(tokenizers::Encoding::len)
                .max()
                .unwrap_or(0);
            let seq_len = max_len.min(512); // Cap at reasonable max length

            // flatten with padding/truncation
            let mut input_ids_vec = Vec::with_capacity(batch_size * seq_len);
            let mut mask_vec = Vec::with_capacity(batch_size * seq_len);
            let mut type_ids_vec = Vec::with_capacity(batch_size * seq_len);

            for e in &encoding {
                let ids = e.get_ids();
                let mask = e.get_attention_mask();
                let type_ids = e.get_type_ids();

                // Truncate or pad to seq_len
                for i in 0..seq_len {
                    if i < ids.len() {
                        input_ids_vec.push(i32::try_from(ids[i]).unwrap_or_default());
                        mask_vec.push(i32::try_from(mask[i]).unwrap_or_default());
                        type_ids_vec.push(i32::try_from(type_ids[i]).unwrap_or_default());
                    } else {
                        // Pad with zeros (standard padding for BERT-style models)
                        input_ids_vec.push(0i32);
                        mask_vec.push(0i32);
                        type_ids_vec.push(0i32);
                    }
                }
            }

            let device = <B as Backend>::Device::default();

            // Create Tensors
            let _input_ids: Tensor<B, 2, burn::tensor::Int> = Tensor::from_data(
                TensorData::new(input_ids_vec, Shape::new([batch_size, seq_len])),
                &device,
            );
            // Assuming generated model takes generic arguments or specific names?
            // burn-import usually generates `forward(input_ids, attention_mask, token_type_ids)`
            // If argument names mismatch, we'll see compilation error.

            // UNCOMMENT ONCE COMPILED TO VERIFY SIGNATURE
            // let output = model.forward(input_ids, ...);

            // Returning placeholder zero-embeddings to allow compilation until signature is known
            warn!("Burn inference running in generic placeholder mode");
            let dim = 384;
            Ok(vec![vec![0.0; dim]; batch_size])
        } else {
            Err(anyhow::anyhow!("Model not initialized"))
        }
    }

    pub async fn index_skills(&self, registry: &SkillRegistry) -> Result<()> {
        let skills = registry.list();
        if skills.is_empty() {
            return Ok(());
        }

        let texts: Vec<String> = skills
            .iter()
            .map(|s| format!("{}: {}", s.title, s.description))
            .collect();

        self.initialize().await?;

        let embeddings = self.embed_batch(texts).await?;

        let mut cache = self.embeddings.lock().await;
        cache.clear();
        for (i, emb) in embeddings.into_iter().enumerate() {
            cache.push((skills[i].skill_id.clone(), emb));
        }
        info!("Skill vector index built (Burn).");

        Ok(())
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b).map(|(x, y)| x * y).sum();
        let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl SkillMatcher for VectorMatcher {
    async fn match_skills(&self, query: &str, registry: &SkillRegistry) -> Result<Vec<SkillMatch>> {
        self.initialize().await?;

        let res = self.embed_batch(vec![query.to_string()]).await?;
        let q_emb = res.into_iter().next().context("No embedding")?;

        // Re-index if empty
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
            let score = Self::cosine_similarity(&q_emb, emb);
            if score >= self.threshold && let Some(skill) = registry.get(skill_id) {
                matches.push(SkillMatch {
                    skill_id: skill_id.clone(),
                    score,
                    reason: MatchReason::VectorSimilarity(score),
                    skill: skill.clone(),
                });
            }
        }
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        Ok(matches)
    }
}
