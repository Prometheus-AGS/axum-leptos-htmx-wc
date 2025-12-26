use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct ModelList {
    pub object: String, // "list"
    pub data: Vec<ModelCard>,
}

#[derive(Serialize)]
pub struct ModelCard {
    pub id: String,
    pub object: String, // "model"
    pub created: u64,
    pub owned_by: String,
}

#[derive(Deserialize, Debug)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub stream: bool,
    // Add other fields as needed (temperature, etc.) but make them optional
}

#[derive(Deserialize, Debug, Clone)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String, // "chat.completion.chunk"
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatCompletionChunkChoice>,
}

#[derive(Serialize, Debug)]
pub struct ChatCompletionChunkChoice {
    pub index: usize,
    pub delta: ChatCompletionChunkDelta,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Debug, Default)]
pub struct ChatCompletionChunkDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}
