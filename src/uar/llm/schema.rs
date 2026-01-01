use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSpec {
    pub id: String,
    pub name: String,
    pub provider_id: String,
    pub context_window: u32,
    pub pricing: ModelPricing,
    pub capabilities: ModelCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_cost_per_m: f32,
    pub output_cost_per_m: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub vision: bool,
    pub function_calling: bool,
    pub json_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSpec {
    pub id: String,
    pub name: String,
    pub api_base: Option<String>,
    pub required_env_vars: Vec<String>,
}
