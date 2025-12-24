/**
 * Token Counter Utility
 * 
 * Client-side token estimation for various models
 */

/**
 * Estimate tokens for a given text using heuristic approach
 * 
 * Rough approximation:
 * - GPT models: ~4 characters per token
 * - Claude models: ~3.5 characters per token
 * - Other models: ~4 characters per token (default)
 */
export function estimateTokens(text: string, modelId?: string): number {
  if (!text) return 0;

  const charCount = text.length;
  
  // Adjust ratio based on model
  let charsPerToken = 4;
  
  if (modelId) {
    const lowerModel = modelId.toLowerCase();
    if (lowerModel.includes('claude')) {
      charsPerToken = 3.5;
    } else if (lowerModel.includes('gpt')) {
      charsPerToken = 4;
    }
  }

  return Math.ceil(charCount / charsPerToken);
}

/**
 * Estimate tokens for a conversation (array of messages)
 */
export interface Message {
  role: string;
  content: string;
}

export function estimateConversationTokens(messages: Message[], modelId?: string): number {
  let totalTokens = 0;

  // Add base tokens for message formatting (varies by model)
  const baseTokensPerMessage = 4; // Approximate overhead per message

  for (const message of messages) {
    totalTokens += baseTokensPerMessage;
    totalTokens += estimateTokens(message.content, modelId);
  }

  // Add base tokens for conversation
  totalTokens += 3;

  return totalTokens;
}

/**
 * Format token count with thousands separator
 */
export function formatTokenCount(count: number): string {
  return count.toLocaleString('en-US');
}

/**
 * Calculate percentage of context window used
 */
export function calculateContextPercentage(used: number, limit: number): number {
  if (limit === 0) return 0;
  return Math.min(100, (used / limit) * 100);
}

/**
 * Get color class based on context usage percentage
 */
export function getUsageColorClass(percentage: number): string {
  if (percentage >= 90) return 'text-danger';
  if (percentage >= 70) return 'text-warning';
  return 'text-success';
}

/**
 * Get background color class for progress bar
 */
export function getUsageBackgroundClass(percentage: number): string {
  if (percentage >= 90) return 'bg-danger';
  if (percentage >= 70) return 'bg-warning';
  return 'bg-success';
}

/**
 * Format cost in USD
 */
export function formatCost(cost: number): string {
  if (cost < 0.01) {
    return `$${cost.toFixed(4)}`;
  }
  return `$${cost.toFixed(2)}`;
}
