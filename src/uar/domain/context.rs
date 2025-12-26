use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextStrategy {
    /// Keep only the most recent messages that fit within the token budget.
    SlidingWindow,
    /// Progressively summarize older messages to preserve key information.
    ProgressiveSummarization,
    /// Three-tier memory (short/mid/long-term) - placeholder for now.
    HierarchicalMemory,
    /// Keep first (System+First User) and last N messages, truncate middle.
    KeepFirstLast,
    /// No strategy applied.
    None,
}

impl Default for ContextStrategy {
    fn default() -> Self {
        Self::SlidingWindow
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    /// The strategy to use.
    pub strategy: ContextStrategy,
    /// Maximum tokens allowed for the conversation history (context window).
    /// If None, defaults to model's limit minus a safety buffer.
    pub max_tokens: Option<usize>,
    /// Threshold at which to trigger context management (0.0 - 1.0).
    /// e.g. 0.9 means trigger when 90% of max_tokens is reached.
    pub trigger_threshold: f32,
    /// For SlidingWindow: max messages to keep (optional).
    pub max_messages: Option<usize>,
    /// For Summarization: max tokens reserved for the summary.
    pub summary_budget: Option<usize>,
    /// For Summarization: Model to use for summarization (if different from main).
    pub summarization_model: Option<String>,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            strategy: ContextStrategy::SlidingWindow,
            max_tokens: None,
            trigger_threshold: 0.85,
            max_messages: None,
            summary_budget: Some(1000),
            summarization_model: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextAction {
    pub strategy: ContextStrategy,
    pub messages_removed: usize,
    pub tokens_saved: usize,
    pub was_applied: bool,
    pub summary_generated: bool,
}
