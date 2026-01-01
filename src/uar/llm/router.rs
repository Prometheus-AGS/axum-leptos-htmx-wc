use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteTarget {
    Fast,      // e.g. Haiku, Groq
    Smart,     // e.g. Opus, GPT-4
    Reasoning, // e.g. O1, R1
}

#[derive(Debug, Clone)]
pub struct RouterRequest {
    pub prompt: String,
    pub complexity_score: Option<f32>,
}

#[derive(Debug, Default)]
pub struct LLMRouter {
    // Model configs could be injected here
}

impl LLMRouter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Determines the best route based on prompt characteristics.
    /// This is a simplified port of the Single-Round routing logic.
    pub async fn route(&self, request: &RouterRequest) -> RouteTarget {
        // 1. Check for explicit reasoning cues
        if Self::needs_reasoning(&request.prompt) {
            return RouteTarget::Reasoning;
        }

        // 2. Complexity Analysis (Heuristic or Model-based)
        let score = request
            .complexity_score
            .unwrap_or_else(|| Self::calculate_complexity(&request.prompt));

        if score > 0.7 {
            RouteTarget::Smart
        } else {
            RouteTarget::Fast
        }
    }

    fn needs_reasoning(prompt: &str) -> bool {
        let keys = ["solve", "prove", "math", "logic", "analyze", "why"];
        let lower = prompt.to_lowercase();
        keys.iter().any(|k| lower.contains(k)) && prompt.len() > 50
    }

    fn calculate_complexity(prompt: &str) -> f32 {
        // Simple heuristic for now: length + code indicators
        let mut score: f32 = 0.0;

        // Length factor
        if prompt.len() > 1000 {
            score += 0.4;
        } else if prompt.len() > 200 {
            score += 0.2;
        }

        // Code factor
        if prompt.contains("```") || prompt.contains("fn ") || prompt.contains("def ") {
            score += 0.3;
        }

        // Context factor (placeholder)
        // If we had conversation history, we'd weight that.

        score.min(1.0)
    }
}
