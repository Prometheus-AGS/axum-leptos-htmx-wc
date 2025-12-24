# Provider Configuration Guide

This document describes how to configure the application for different LLM providers.

## Environment Variables

### Required Variables

```bash
LLM_BASE_URL=https://api.openai.com
LLM_API_KEY=sk-....
LLM_MODEL=gpt-5.2
```

### Optional Variables

```bash
# Protocol selection (default: auto)
LLM_PROTOCOL=auto | responses | chat

# Enable/disable parallel tool calls (default: auto-detected by provider)
LLM_PARALLEL_TOOLS=true

# Azure OpenAI specific (required if using Azure)
AZURE_DEPLOYMENT_NAME=gpt-4
AZURE_API_VERSION=2024-08-01-preview
```

## Supported Providers

### OpenAI

```bash
LLM_BASE_URL=https://api.openai.com
LLM_API_KEY=sk-...
LLM_MODEL=gpt-5.2
```

**Supported Models**: `gpt-5.2`, `gpt-5.2-pro`, `gpt-5.2-codex`, `gpt-4o`, `gpt-4`, etc.

**Features**:
- Parallel tool calls: ✅ Supported
- Streaming: ✅ Supported
- Tool calling: ✅ Supported

### Azure OpenAI

```bash
LLM_BASE_URL=https://your-resource.openai.azure.com
LLM_API_KEY=your-azure-key
LLM_MODEL=gpt-4  # Not used in URL, but kept for reference
AZURE_DEPLOYMENT_NAME=gpt-4-deployment
AZURE_API_VERSION=2024-08-01-preview
```

**Features**:
- Parallel tool calls: ✅ Supported
- Streaming: ✅ Supported
- Tool calling: ✅ Supported

**Note**: Azure uses deployment names in the URL instead of model names. The URL pattern is:
```
{base_url}/openai/deployments/{deployment_name}/chat/completions?api-version={version}
```

### OpenRouter

```bash
LLM_BASE_URL=https://openrouter.ai/api
LLM_API_KEY=sk-or-...
LLM_MODEL=openai/gpt-4
```

**Features**:
- Parallel tool calls: ✅ Supported (model-dependent)
- Streaming: ✅ Supported
- Tool calling: ✅ Supported
- Multi-provider routing: ✅ Supported

**Note**: OpenRouter routes to multiple providers transparently. Tool calling support depends on the underlying model.

### Together.ai

```bash
LLM_BASE_URL=https://api.together.xyz
LLM_API_KEY=...
LLM_MODEL=meta-llama/Llama-3-70b-chat-hf
```

**Features**:
- Parallel tool calls: ⚠️ Model-dependent
- Streaming: ✅ Supported
- Tool calling: ✅ Supported

### Groq

```bash
LLM_BASE_URL=https://api.groq.com
LLM_API_KEY=gsk-...
LLM_MODEL=llama-3.1-70b-versatile
```

**Features**:
- Parallel tool calls: ✅ Supported
- Streaming: ✅ Supported
- Tool calling: ✅ Supported
- Fast inference: ✅ Optimized hardware

**Note**: Groq provides extremely fast inference with LPU (Language Processing Units).

## Provider Auto-Detection

The application automatically detects the provider based on the `LLM_BASE_URL`:

- `api.openai.com` → OpenAI
- `*.openai.azure.com` → Azure OpenAI
- `openrouter.ai` → OpenRouter
- `together.ai` or `together.xyz` → Together.ai
- `groq.com` → Groq
- Others → Generic OpenAI-compatible

## Tool Calling Configuration

### Parallel Tool Calls

By default, the application auto-detects whether the provider supports parallel tool calls. You can override this behavior:

```bash
# Force enable parallel tool calls
LLM_PARALLEL_TOOLS=true

# Force disable parallel tool calls
LLM_PARALLEL_TOOLS=false
```

**Providers with Parallel Tool Call Support**:
- ✅ OpenAI
- ✅ Azure OpenAI
- ✅ Groq
- ⚠️ OpenRouter (model-dependent)
- ⚠️ Together.ai (model-dependent)

## Example Configurations

### Example 1: OpenAI with GPT-5.2

```bash
LLM_PROTOCOL=auto
LLM_BASE_URL=https://api.openai.com
LLM_API_KEY=sk-proj-...
LLM_MODEL=gpt-5.2
LLM_PARALLEL_TOOLS=true
```

### Example 2: Azure OpenAI

```bash
LLM_PROTOCOL=auto
LLM_BASE_URL=https://my-resource.openai.azure.com
LLM_API_KEY=abc123...
LLM_MODEL=gpt-4
AZURE_DEPLOYMENT_NAME=gpt-4-deployment
AZURE_API_VERSION=2024-08-01-preview
```

### Example 3: Groq with Fast Inference

```bash
LLM_PROTOCOL=auto
LLM_BASE_URL=https://api.groq.com
LLM_API_KEY=gsk-...
LLM_MODEL=llama-3.1-70b-versatile
```

### Example 4: OpenRouter Multi-Provider

```bash
LLM_PROTOCOL=auto
LLM_BASE_URL=https://openrouter.ai/api
LLM_API_KEY=sk-or-...
LLM_MODEL=anthropic/claude-3-opus
```

## Troubleshooting

### "Missing required env var: LLM_BASE_URL"

Make sure you have a `.env` file in the project root with the required variables. Copy `.env.example` to `.env` and fill in your values.

### Azure: "Deployment not found"

Verify that:
1. `AZURE_DEPLOYMENT_NAME` matches your actual deployment name in Azure
2. `AZURE_API_VERSION` is a valid API version
3. Your API key has access to the deployment

### Tool calls not working

1. Check if your model supports tool calling
2. Try setting `LLM_PARALLEL_TOOLS=false` if the model doesn't support parallel calls
3. Verify your API key has the necessary permissions

## Testing

To test your configuration:

```bash
cargo run
```

The server will start on `http://127.0.0.1:3000`. Open the URL in your browser and try sending a message that would trigger a tool call (e.g., "What time is it?").
