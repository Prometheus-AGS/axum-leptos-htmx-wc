use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
    #[serde(rename = "outputSchema", default)]
    pub output_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<McpTool>,
    #[serde(rename = "nextCursor", default)]
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<serde_json::Value>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
    #[serde(rename = "structuredContent", default)]
    pub structured_content: Option<serde_json::Value>,
}
