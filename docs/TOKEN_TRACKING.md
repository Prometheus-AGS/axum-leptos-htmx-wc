# Token Tracking Implementation

## Overview

This document describes the hybrid token tracking system implemented in the application, combining client-side estimation with server-side usage reporting from LLM APIs.

## Architecture

### Client-Side Components

#### 1. Model Information Cache (`web/utils/model-info.ts`)

Fetches and caches model specifications from the `models.dev` API:

```typescript
// Fetch model data on init
await modelInfoCache.init();

// Get model information
const info = getModelInfo("gpt-4o");
const contextLimit = getContextLimit("gpt-4o");
const cost = estimateCost("gpt-4o", inputTokens, outputTokens);
```

**Features:**
- Caches model data for 24 hours
- Provides context limits, token limits, and pricing
- Supports multiple providers (OpenAI, Anthropic, etc.)
- Falls back to GPT-4 defaults if model not found

#### 2. Token Counter Utility (`web/utils/token-counter.ts`)

Client-side token estimation using heuristics:

```typescript
// Estimate tokens for text
const tokens = estimateTokens(text, "gpt-4o");

// Estimate for conversation
const total = estimateConversationTokens(messages, "gpt-4o");

// Format and display
const formatted = formatTokenCount(tokens);
const percentage = calculateContextPercentage(used, limit);
const color = getUsageColorClass(percentage);
```

**Heuristics:**
- GPT models: ~4 characters per token
- Claude models: ~3.5 characters per token
- Includes overhead for message formatting

#### 3. Token Counter Component (`web/components/token-counter/token-counter.ts`)

Web Component for displaying token usage:

```html
<token-counter
  input-tokens="1234"
  output-tokens="567"
  context-limit="128000"
  model-id="gpt-4o"
  cost="0.0012"
  is-estimate="false">
</token-counter>
```

**Features:**
- Visual progress bar with color coding:
  - Green: 0-70% usage
  - Yellow: 70-90% usage
  - Red: 90-100% usage
- Tooltip with detailed breakdown
- Cost estimation display
- Responsive design

### Server-Side Components

#### 1. Usage Event (`src/normalized.rs`)

New event type for token usage information:

```rust
#[serde(rename = "usage")]
Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
},
```

#### 2. Chat Completions Driver (`src/llm/chat_completions.rs`)

Updated to request and parse usage information:

```rust
// Request usage in stream
let body = json!({
    "model": self.settings.model,
    "stream": true,
    "stream_options": {
        "include_usage": true
    },
    // ...
});

// Parse usage from final chunk
if let Some(usage) = v.get("usage") {
    yield NormalizedEvent::Usage {
        prompt_tokens: prompt as u32,
        completion_tokens: completion as u32,
        total_tokens: total as u32,
    };
}
```

### State Management

Token usage is tracked in the Alpine.js global store:

```typescript
interface ChatStore {
  tokenUsage: {
    input: number;
    output: number;
    total: number;
    limit: number;
    isEstimate: boolean;
    cost: number;
  };
  updateTokenUsage(input, output, limit, isEstimate?, cost?): void;
}
```

## Usage Flow

### 1. Initialization

```typescript
// On app start
await modelInfoCache.init(); // Fetch model data
```

### 2. Client-Side Estimation

Before sending a message:

```typescript
const inputTokens = estimateConversationTokens(messages, modelId);
const limit = getContextLimit(modelId);
const cost = estimateCost(modelId, inputTokens, 0);

Alpine.store('chat').updateTokenUsage(inputTokens, 0, limit, true, cost);
```

### 3. Server-Side Reporting

After receiving the stream:

```typescript
// Listen for usage event
eventSource.addEventListener('usage', (event) => {
  const data = JSON.parse(event.data);
  const cost = estimateCost(modelId, data.prompt_tokens, data.completion_tokens);
  
  Alpine.store('chat').updateTokenUsage(
    data.prompt_tokens,
    data.completion_tokens,
    limit,
    false, // Not an estimate
    cost
  );
});
```

### 4. Display

```html
<!-- In header or footer -->
<token-counter
  x-bind:input-tokens="$store.chat.tokenUsage.input"
  x-bind:output-tokens="$store.chat.tokenUsage.output"
  x-bind:context-limit="$store.chat.tokenUsage.limit"
  x-bind:cost="$store.chat.tokenUsage.cost"
  x-bind:is-estimate="$store.chat.tokenUsage.isEstimate"
  model-id="gpt-4o">
</token-counter>
```

## Persistence

Token usage can be persisted to PGlite by adding a `token_usage` JSONB column to the `conversations` table:

```sql
ALTER TABLE conversations
ADD COLUMN token_usage JSONB DEFAULT '{"input": 0, "output": 0, "total": 0}';
```

## API Reference

### models.dev API

**Endpoint:** `https://models.dev/api.json`

**Response Structure:**
```json
{
  "openai": {
    "models": {
      "gpt-4o": {
        "name": "gpt-4o",
        "limit": {
          "context": 128000,
          "input": 128000,
          "output": 4096
        },
        "cost": {
          "input": 2.5,
          "output": 10.0
        },
        "modalities": {
          "input": ["text", "image"],
          "output": ["text"]
        },
        "tool_call": true,
        "reasoning": false
      }
    }
  }
}
```

### OpenAI Stream Options

To receive usage information in streaming mode:

```json
{
  "stream": true,
  "stream_options": {
    "include_usage": true
  }
}
```

The usage information is sent in the final chunk before `[DONE]`.

## Benefits

1. **Immediate Feedback**: Client-side estimation provides instant token counts
2. **Accurate Reporting**: Server-side usage from API ensures accuracy
3. **Cost Awareness**: Real-time cost estimation helps users manage expenses
4. **Context Management**: Visual indicators prevent context overflow
5. **Model Flexibility**: Supports any model in the models.dev database

## Future Enhancements

1. **Per-Conversation Tracking**: Store token usage per conversation
2. **Historical Analytics**: Track token usage over time
3. **Budget Alerts**: Warn when approaching token or cost limits
4. **Token Optimization**: Suggest ways to reduce token usage
5. **Batch Estimation**: Pre-calculate tokens for multiple messages
