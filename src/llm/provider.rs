//! Provider-specific configuration and detection.
//!
//! This module handles differences between LLM API providers, including
//! URL patterns, authentication, and feature support.

/// Supported LLM providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Provider {
    /// `OpenAI` (api.openai.com)
    OpenAI,
    /// Azure `OpenAI` Service
    AzureOpenAI {
        /// Deployment name (required for Azure)
        deployment_name: String,
        /// API version (e.g., "2024-08-01-preview")
        api_version: String,
    },
    /// `OpenRouter` (openrouter.ai)
    OpenRouter,
    /// Together AI (together.ai, together.xyz)
    TogetherAI,
    /// Groq (groq.com)
    Groq,
    /// Generic OpenAI-compatible provider
    Generic,
}

impl Provider {
    /// Detect provider from base URL.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let provider = Provider::detect_from_url("https://api.openai.com");
    /// assert_eq!(provider, Provider::OpenAI);
    /// ```
    #[must_use]
    pub fn detect_from_url(base_url: &str) -> Self {
        let lower = base_url.to_lowercase();

        if lower.contains("azure.com") || lower.contains("openai.azure.com") {
            Self::AzureOpenAI {
                deployment_name: String::new(),
                api_version: "2024-08-01-preview".to_string(),
            }
        } else if lower.contains("openrouter.ai") {
            Self::OpenRouter
        } else if lower.contains("together.ai") || lower.contains("together.xyz") {
            Self::TogetherAI
        } else if lower.contains("groq.com") {
            Self::Groq
        } else if lower.contains("openai.com") {
            Self::OpenAI
        } else {
            Self::Generic
        }
    }

    /// Check if this provider supports parallel tool calls.
    #[must_use]
    pub fn supports_parallel_tools(&self) -> bool {
        match self {
            Self::OpenAI | Self::AzureOpenAI { .. } | Self::Groq => true,
            Self::OpenRouter | Self::TogetherAI | Self::Generic => true, // Most do, but model-dependent
        }
    }

    /// Build the chat completions URL for this provider.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL (without trailing slash)
    /// * `model` - The model name (unused for Azure, which uses deployment name)
    #[must_use]
    pub fn build_chat_url(&self, base_url: &str, _model: &str) -> String {
        let base = base_url.trim_end_matches('/');

        match self {
            Self::AzureOpenAI {
                deployment_name,
                api_version,
            } => {
                format!(
                    "{base}/openai/deployments/{deployment_name}/chat/completions?api-version={api_version}"
                )
            }
            _ => format!("{base}/v1/chat/completions"),
        }
    }

    /// Check if a model supports vision/image inputs.
    ///
    /// This is a heuristic check based on known model naming patterns.
    /// For accurate detection, consider using the models.dev API.
    ///
    /// # Arguments
    ///
    /// * `model` - The model identifier to check
    ///
    /// # Example
    ///
    /// ```rust
    /// use axum_leptos_htmx_wc::llm::Provider;
    ///
    /// assert!(Provider::supports_vision("gpt-4o"));
    /// assert!(Provider::supports_vision("claude-3-5-sonnet"));
    /// assert!(!Provider::supports_vision("gpt-3.5-turbo"));
    /// ```
    #[must_use]
    pub fn supports_vision(model: &str) -> bool {
        let lower = model.to_lowercase();

        // OpenAI vision models
        if lower.contains("gpt-4o") || lower.contains("gpt-4-vision") {
            return true;
        }

        // Anthropic Claude 3 models (all support vision)
        if lower.contains("claude-3") {
            return true;
        }

        // Google Gemini models (vision capable)
        if lower.contains("gemini") {
            return true;
        }

        // Mistral Pixtral (vision model)
        if lower.contains("pixtral") {
            return true;
        }

        // Qwen VL models
        if lower.contains("qwen") && lower.contains("vl") {
            return true;
        }

        // LLaVA models
        if lower.contains("llava") {
            return true;
        }

        // CogVLM models
        if lower.contains("cogvlm") {
            return true;
        }

        // InternVL models
        if lower.contains("internvl") {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_openai() {
        let provider = Provider::detect_from_url("https://api.openai.com");
        assert_eq!(provider, Provider::OpenAI);
    }

    #[test]
    fn test_detect_azure() {
        let provider = Provider::detect_from_url("https://my-resource.openai.azure.com");
        assert!(matches!(provider, Provider::AzureOpenAI { .. }));
    }

    #[test]
    fn test_detect_openrouter() {
        let provider = Provider::detect_from_url("https://openrouter.ai");
        assert_eq!(provider, Provider::OpenRouter);
    }

    #[test]
    fn test_detect_groq() {
        let provider = Provider::detect_from_url("https://api.groq.com");
        assert_eq!(provider, Provider::Groq);
    }

    #[test]
    fn test_build_url_openai() {
        let provider = Provider::OpenAI;
        let url = provider.build_chat_url("https://api.openai.com", "gpt-4");
        assert_eq!(url, "https://api.openai.com/v1/chat/completions");
    }

    #[test]
    fn test_build_url_azure() {
        let provider = Provider::AzureOpenAI {
            deployment_name: "gpt-4".to_string(),
            api_version: "2024-08-01-preview".to_string(),
        };
        let url = provider.build_chat_url("https://my-resource.openai.azure.com", "gpt-4");
        assert_eq!(
            url,
            "https://my-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2024-08-01-preview"
        );
    }
}
