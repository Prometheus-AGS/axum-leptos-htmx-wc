use crate::llm::Message;
use tiktoken_rs::cl100k_base;

#[derive(Debug)]
pub struct TokenService;

impl TokenService {
    /// Estimate tokens for a string using cl100k_base (GPT-4/3.5 standard).
    pub fn estimate_string(content: &str) -> usize {
        // Fallback to cl100k_base if model generic
        let bpe = cl100k_base().unwrap();
        bpe.encode_with_special_tokens(content).len()
    }

    /// Estimate tokens for a list of messages.
    /// This follows OpenAI's chat format rules roughly (overhead per message).
    pub fn estimate_messages(messages: &[Message]) -> usize {
        let bpe = cl100k_base().unwrap();
        let mut num_tokens = 0;

        // Every message follows <|start|>{role/name}\n{content}<|end|>\n
        // Approximate overhead is 3 tokens per message logic + content
        // We'll use a safe approximation.

        for message in messages {
            num_tokens += 3; // overhead
            let content_str = message.content.as_text().unwrap_or("");
            num_tokens += bpe.encode_with_special_tokens(content_str).len();

            // If we had name field, +1 token.
            // If tool calls, we need to count them too.
            if let Some(calls) = &message.tool_calls {
                for call in calls {
                    num_tokens += bpe.encode_with_special_tokens(&call.function.name).len();
                    num_tokens += bpe
                        .encode_with_special_tokens(&call.function.arguments)
                        .len();
                }
            }
        }

        num_tokens += 3; // Every reply is primed with <|start|>assistant<|message|>
        num_tokens
    }
}
