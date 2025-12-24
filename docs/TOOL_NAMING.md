# Tool Naming Conventions

## Overview

This document explains how MCP (Model Context Protocol) tool names are sanitized for compatibility with various LLM providers, particularly OpenAI and OpenAI-compatible APIs.

## The Problem

Different systems have different requirements for tool/function names:

### MCP Specification
- **Allowed characters**: `a-z`, `A-Z`, `0-9`, `_` (underscore), `-` (hyphen), `.` (dot)
- **Example valid names**: `getUser`, `DATA_EXPORT_v2`, `admin.tools.list`
- **Namespacing**: Often uses dots (`.`) or colons (`:`) for organization

### OpenAI API Requirements
- **Pattern**: `^[a-zA-Z0-9_-]+$`
- **Allowed characters**: `a-z`, `A-Z`, `0-9`, `_` (underscore), `-` (hyphen)
- **NOT allowed**: dots (`.`), colons (`:`), spaces, or any special characters
- **Applies to**: All OpenAI models including:
  - GPT-4 series (`gpt-4`, `gpt-4-turbo`, `gpt-4o`)
  - GPT-5 series (`gpt-5`, `gpt-5-mini`, `gpt-5-nano`, `gpt-5.1`, `gpt-5.2`)
  - Reasoning models (`o1`, `o1-mini`, `o3`, `o3-mini`, `o4-mini`)
  - Legacy models (`gpt-3.5-turbo`)

### Other Provider Compatibility

| Provider | Follows OpenAI Spec | Notes |
|----------|---------------------|-------|
| **Azure OpenAI** | ✅ Yes | Identical to OpenAI requirements |
| **OpenRouter** | ✅ Yes | Proxies to various models, uses OpenAI format |
| **Together.ai** | ✅ Yes | OpenAI-compatible API |
| **Groq** | ✅ Yes | OpenAI-compatible API |
| **Ollama** | ⚠️ Permissive | More lenient, but works with OpenAI format |

## Our Solution

### Sanitization Strategy

We transform MCP tool names to be OpenAI-compatible while maintaining the ability to map back to the original server and tool:

1. **Namespacing**: Use double underscore (`__`) instead of double colon (`::`
   - Original MCP: `tavily::search`
   - Sanitized: `tavily__search`

2. **Character Replacement**: Replace invalid characters with underscores
   - Dots (`.`) → `_`
   - Colons (`:`) → `_`
   - Any other special chars → `_`

3. **Preserve Valid Characters**: Keep alphanumerics, underscores, and hyphens as-is

### Implementation

```rust
fn sanitize_tool_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'  // Replace invalid chars with underscore
            }
        })
        .collect()
}
```

### Examples

| Original MCP Format | Sanitized for OpenAI | Notes |
|---------------------|----------------------|-------|
| `tavily::search` | `tavily__search` | Namespace separator |
| `time::now` | `time__now` | Namespace separator |
| `admin.tools.list` | `admin_tools_list` | Dots replaced |
| `get-user-data` | `get-user-data` | Hyphens preserved |
| `DATA_EXPORT_v2` | `DATA_EXPORT_v2` | Underscores preserved |

## Error Messages

If you see an error like:

```
Invalid 'tools[0].function.name': string does not match pattern. 
Expected a string that matches the pattern '^[a-zA-Z0-9_-]+$'. 
[parameter: tools[0].function.name] [code: invalid_value]
```

This means a tool name contains characters that violate OpenAI's naming requirements. The sanitization in `src/mcp/registry.rs` should prevent this, but if you see it:

1. Check that `sanitize_tool_name()` is being called on all tool names
2. Verify the MCP server isn't returning tool names with unusual characters
3. Check the logs to see which specific tool name is failing

## Model-Specific Considerations

### GPT-5.x and GPT-5+-mini Series
- Same naming requirements as GPT-4
- No special handling needed
- Supports parallel tool calls (set `parallel_tool_calls: true`)

### Reasoning Models (o1, o3, o4 series)
- Same naming requirements
- May have different `reasoning_format` options
- Tool calling works the same way

### Azure OpenAI
- Identical requirements to OpenAI
- Uses deployment names instead of model names
- Tool naming is the same

### OpenRouter
- Acts as a proxy to multiple providers
- Always use OpenAI-compatible format
- Some models may have additional restrictions

## Testing

To verify tool names are correctly sanitized:

1. Check the logs when tools are registered:
   ```
   INFO  - Registered tool: tavily__search (from server: tavily)
   ```

2. Check the request body sent to the LLM API:
   ```rust
   DEBUG - LLM request body: {
     "tools": [
       {
         "type": "function",
         "function": {
           "name": "tavily__search",  // Should match pattern
           ...
         }
       }
     ]
   }
   ```

3. If you get a 400 error, check the error details in logs:
   ```
   ERROR - LLM API returned error with details
         error_param=Some("tools[0].function.name")
         error_message="Invalid tool name format"
   ```

## References

- [OpenAI Function Calling Documentation](https://platform.openai.com/docs/guides/function-calling)
- [MCP Tool Specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)
- [Azure OpenAI Function Calling](https://learn.microsoft.com/en-us/azure/ai-services/openai/how-to/function-calling)
