use super::token_service::TokenService;
use crate::llm::{Message, MessageRole};
use crate::uar::domain::context::{ContextAction, ContextConfig, ContextStrategy};
use tracing::{info, warn};

#[derive(Debug)]
pub struct ContextManager {
    config: ContextConfig,
}

impl ContextManager {
    pub fn new(config: ContextConfig) -> Self {
        Self { config }
    }

    /// Check if context management is needed and apply the configured strategy.
    /// Returns the (potentially modified) messages and an action report if changes were made.
    pub async fn apply(
        &self,
        messages: Vec<Message>,
        model_token_limit: usize,
    ) -> (Vec<Message>, Option<ContextAction>) {
        let current_tokens = TokenService::estimate_messages(&messages);
        // Use configured max or model limit - buffer (e.g. 1000 tokens for output)
        let effective_max = self
            .config
            .max_tokens
            .unwrap_or(model_token_limit.saturating_sub(1000));
        let threshold = (effective_max as f32 * self.config.trigger_threshold) as usize;

        if current_tokens <= threshold {
            return (messages, None);
        }

        info!(
            "Context usage {} exceeds threshold {}. Applying {:?} strategy.",
            current_tokens, threshold, self.config.strategy
        );

        match self.config.strategy {
            ContextStrategy::SlidingWindow => {
                self.apply_sliding_window(messages, effective_max, current_tokens)
                    .await
            }
            ContextStrategy::KeepFirstLast => {
                self.apply_keep_first_last(messages, effective_max, current_tokens)
                    .await
            }
            ContextStrategy::ProgressiveSummarization => {
                // For now, fall back to KeepFirstLast until LLM summarizer is wired
                warn!(
                    "ProgressiveSummarization not yet fully wired, falling back to KeepFirstLast"
                );
                self.apply_keep_first_last(messages, effective_max, current_tokens)
                    .await
            }
            _ => (messages, None),
        }
    }

    async fn apply_sliding_window(
        &self,
        messages: Vec<Message>,
        token_budget: usize,
        original_tokens: usize,
    ) -> (Vec<Message>, Option<ContextAction>) {
        let mut budget = token_budget;
        let mut final_list = Vec::new();

        // 1. Preserve System Prompt
        let system_msg = messages
            .iter()
            .find(|m| m.role == MessageRole::System)
            .cloned();

        if let Some(sys) = &system_msg {
            let t = TokenService::estimate_string(sys.content.as_text().unwrap_or("")) + 3;
            budget = budget.saturating_sub(t);
            final_list.push(sys.clone());
        }

        // 2. Keep recent messages within remaining budget
        let mut tail = Vec::new();
        // Skip system message in reverse iteration if we already handled it
        for msg in messages.iter().rev() {
            if msg.role == MessageRole::System {
                continue;
            }

            let t = TokenService::estimate_string(msg.content.as_text().unwrap_or("")) + 3;
            if t <= budget {
                tail.push(msg.clone());
                budget -= t;
            } else {
                break; // Stop once we hit limit
            }
        }
        tail.reverse();
        final_list.extend(tail);

        let new_len = final_list.len();
        let removed_count = messages.len() - new_len;
        let tokens_saved =
            original_tokens.saturating_sub(TokenService::estimate_messages(&final_list));

        (
            final_list,
            Some(ContextAction {
                strategy: ContextStrategy::SlidingWindow,
                messages_removed: removed_count,
                tokens_saved,
                was_applied: true,
                summary_generated: false,
            }),
        )
    }

    async fn apply_keep_first_last(
        &self,
        messages: Vec<Message>,
        token_budget: usize,
        original_tokens: usize,
    ) -> (Vec<Message>, Option<ContextAction>) {
        // Simple implementation: Keep System + First User + Last N fit

        // 1. Identify critical head messages
        let mut head = Vec::new();
        let mut budget = token_budget;

        // Keep all system messages
        let mut msg_iter = messages.iter().enumerate();

        while let Some((_idx, msg)) = msg_iter.next() {
            if msg.role == MessageRole::System {
                let t = TokenService::estimate_string(msg.content.as_text().unwrap_or("")) + 3;
                if t < budget {
                    head.push(msg.clone());
                    budget -= t;
                }
            } else {
                // First non-system (User)
                // Keep it if budget allows and strategy implies keeping "First"
                // Assumption: "KeepFirstLast" usually implies keeping the very first prompt.
                let t = TokenService::estimate_string(msg.content.as_text().unwrap_or("")) + 3;
                if t < budget {
                    head.push(msg.clone());
                    budget -= t;
                }
                break;
            }
        }

        // Keep last messages
        let mut tail = Vec::new();
        for msg in messages.iter().rev() {
            // How to avoid re-adding head messages?
            // Simple check: if message content/id matches head?
            // Or just ensure we don't overlap indices.
            // But we don't have indices in the loop easily.
            // Simplified: Head is usually small (1-2 msgs). Tail is greedy.
            // We just ensure we don't duplicate logic.
            // If message is System, likely in head.
            if msg.role == MessageRole::System {
                continue;
            }

            // Check if this message is already in head (by content - imperfect but works for now)
            if head
                .iter()
                .any(|h| h.content == msg.content && h.role == msg.role)
            {
                continue;
            }

            let t = TokenService::estimate_string(msg.content.as_text().unwrap_or("")) + 3;
            if t <= budget {
                tail.push(msg.clone());
                budget -= t;
            } else {
                break;
            }
        }
        tail.reverse();

        let mut final_list = head;
        final_list.extend(tail);

        let new_len = final_list.len();
        let removed_count = messages.len() - new_len;
        let tokens_saved =
            original_tokens.saturating_sub(TokenService::estimate_messages(&final_list));

        (
            final_list,
            Some(ContextAction {
                strategy: ContextStrategy::KeepFirstLast,
                messages_removed: removed_count,
                tokens_saved,
                was_applied: true,
                summary_generated: false,
            }),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::MessageContent;

    fn make_msg(content: &str, role: MessageRole) -> Message {
        Message {
            role,
            content: MessageContent::text(content),
            tool_call_id: None,
            tool_calls: None,
        }
    }

    #[tokio::test]
    async fn test_sliding_window_truncation() {
        let config = ContextConfig {
            strategy: ContextStrategy::SlidingWindow,
            max_tokens: Some(100), // Very small limit
            trigger_threshold: 0.5,
            ..Default::default()
        };
        let manager = ContextManager::new(config);

        let mut messages = Vec::new();
        messages.push(make_msg("System Prompt", MessageRole::System)); // ~4 tokens
        for i in 0..10 {
            messages.push(make_msg(&format!("Message {}", i), MessageRole::User)); // ~4 tokens each
        }
        // Total ~ 4 + 40 = 44 tokens roughly.
        // Wait, "Message X" is small.
        // TokenService uses cl100k_base.
        // Let's assume estimate is working.

        // Force limit to be VERY small to trigger.
        // If max_tokens=100, threshold=50.
        // If we duplicate messages to exceed 50.
        for i in 10..50 {
            messages.push(make_msg(&format!("Message {}", i), MessageRole::User));
        }
        // Now we have 50 user messages + 1 system.

        let (optimized, action) = manager.apply(messages.clone(), 1000).await;

        assert!(action.is_some());
        let act = action.unwrap();
        assert_eq!(act.strategy, ContextStrategy::SlidingWindow);
        assert!(act.messages_removed > 0);

        // Ensure System Prompt is preserved
        assert_eq!(optimized[0].role, MessageRole::System);
        assert_eq!(optimized[0].content.as_text().unwrap(), "System Prompt");

        // Ensure we are under budget (100 tokens + buffer logic?)
        // apply() uses effective_max = config.max_tokens (100).
        let final_tokens = TokenService::estimate_messages(&optimized);
        assert!(final_tokens <= 100, "Tokens {} > 100", final_tokens);
    }

    #[tokio::test]
    async fn test_keep_first_last() {
        let config = ContextConfig {
            strategy: ContextStrategy::KeepFirstLast,
            max_tokens: Some(60),
            trigger_threshold: 0.1,
            ..Default::default()
        };
        let manager = ContextManager::new(config);

        let mut messages = Vec::new();
        messages.push(make_msg("System", MessageRole::System));
        messages.push(make_msg("First User", MessageRole::User)); // Keep this
        for _i in 0..20 {
            messages.push(make_msg("Filler", MessageRole::Assistant));
        }
        messages.push(make_msg("Last User", MessageRole::User)); // Keep this

        let (optimized, action) = manager.apply(messages, 1000).await;

        assert!(action.is_some());
        let outcome = optimized;

        // Expect: System, First User, <Tail>
        assert_eq!(outcome[0].content.as_text().unwrap(), "System");
        assert_eq!(outcome[1].content.as_text().unwrap(), "First User");
        assert_eq!(
            outcome.last().unwrap().content.as_text().unwrap(),
            "Last User"
        );

        // Ensure some middle messages are gone
        assert!(outcome.len() < 23);
    }
}
