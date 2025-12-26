use crate::mcp::config::{McpServerEntry, expand_env_map, load_mcp_config};
use anyhow::{Context, anyhow};
use async_trait::async_trait;
use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::ServiceExt,
    transport::{StreamableHttpClientTransport, TokioChildProcess},
};
use std::{collections::HashMap, sync::Arc};
use tokio::process::Command;
use url::Url;

#[async_trait]
pub trait NativeTool: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn schema(&self) -> serde_json::Value;
    async fn call(&self, args: serde_json::Value) -> anyhow::Result<serde_json::Value>;
}

type DynClientService = rmcp::service::RunningService<
    rmcp::service::RoleClient,
    Box<dyn rmcp::service::DynService<rmcp::service::RoleClient>>,
>;

#[derive(Clone)]
pub struct McpRegistry {
    services: Arc<HashMap<String, Arc<DynClientService>>>,
    // namespaced_tool_name -> (server_name, tool_name)
    tool_index: Arc<HashMap<String, (String, String)>>,
    tools: Arc<Vec<(String, Tool)>>, // (namespaced_name, Tool)
    // namespaced_tool_name -> NativeTool
    native_tools: Arc<HashMap<String, Arc<dyn NativeTool>>>,
}

impl std::fmt::Debug for McpRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpRegistry")
            .field("tool_count", &self.tools.len())
            .field("service_count", &self.services.len())
            .field("native_tool_count", &self.native_tools.len())
            .finish()
    }
}

impl McpRegistry {
    pub async fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let cfg = load_mcp_config(path)?;
        Self::from_config(&cfg).await
    }

    pub async fn from_config(cfg: &crate::mcp::config::McpConfig) -> anyhow::Result<Self> {
        // 1) connect all servers
        let mut services: HashMap<String, Arc<DynClientService>> = HashMap::new();

        for (name, entry) in &cfg.mcp_servers {
            let svc = match entry {
                McpServerEntry::Stdio { command, args, env } => {
                    let env = expand_env_map(env);

                    let mut cmd = Command::new(command);
                    cmd.args(args);

                    for (k, v) in env {
                        cmd.env(k, v);
                    }

                    // rmcp docs show TokioChildProcess + configure pattern for adding args
                    let transport = TokioChildProcess::new(cmd)?;
                    // store as dyn to keep a homogeneous collection
                    ().into_dyn()
                        .serve(transport)
                        .await
                        .with_context(|| format!("failed to connect stdio MCP server '{name}'"))?
                }

                McpServerEntry::RemoteHttp { url, env } => {
                    let env = expand_env_map(env);

                    // Tavily expects ?tavilyApiKey=... (per your config contract).
                    // Keep the key OUT of logs.
                    let api_key = env
                        .get("TAVILY_API_KEY")
                        .cloned()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| anyhow!("remote MCP '{name}' missing TAVILY_API_KEY"))?;

                    let mut u = Url::parse(url)
                        .with_context(|| format!("invalid url for remote MCP '{name}': {url}"))?;

                    // If URL already has query, we just append.
                    u.query_pairs_mut().append_pair("tavilyApiKey", &api_key);

                    // rmcp streamable http transport from_uri
                    let transport = StreamableHttpClientTransport::from_uri(u.to_string());
                    ().into_dyn()
                        .serve(transport)
                        .await
                        .with_context(|| format!("failed to connect remote MCP server '{name}'"))?
                }
            };

            services.insert(name.clone(), Arc::new(svc));
        }

        // 2) list tools + build index
        let mut all_tools: Vec<(String, Tool)> = Vec::new();
        let mut tool_index: HashMap<String, (String, String)> = HashMap::new();

        for (server_name, svc) in &services {
            // list_tools exists on the rmcp running service in examples
            let result = svc
                .list_tools(Default::default())
                .await
                .with_context(|| format!("tools/list failed for MCP server '{server_name}'"))?;

            for t in result.tools {
                let tool_name = t.name.to_string();
                // Sanitize tool name for OpenAI compatibility
                // OpenAI requires: ^[a-zA-Z0-9_-]+$ (no colons, dots, or special chars)
                // Replace :: with __ for namespacing, and sanitize any other invalid chars
                let ns_name = Self::sanitize_tool_name(&format!("{server_name}__{tool_name}"));
                tool_index.insert(ns_name.clone(), (server_name.clone(), tool_name));
                all_tools.push((ns_name, t));
            }
        }

        Ok(Self {
            services: Arc::new(services),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(all_tools),
            native_tools: Arc::new(HashMap::new()),
        })
    }

    /// Creates an empty registry for testing.
    pub fn new_empty() -> Self {
        Self {
            services: Arc::new(HashMap::new()),
            tool_index: Arc::new(HashMap::new()),
            tools: Arc::new(Vec::new()),
            native_tools: Arc::new(HashMap::new()),
        }
    }

    /// Creates a registry with a single test tool.
    pub fn new_with_test_tool(name: &str, description: &str) -> Self {
        let ns_name = Self::sanitize_tool_name(&format!("test__{name}"));
        let tool = Tool {
            name: name.to_string().into(),
            description: Some(description.to_string().into()),
            input_schema: Arc::new(
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "mirror": { "type": "string" }
                    },
                    "required": ["mirror"]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            title: None,
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        };

        let mut tools = Vec::new();
        tools.push((ns_name.clone(), tool));
        let mut tool_index = HashMap::new();
        tool_index.insert(ns_name, ("test".to_string(), name.to_string()));

        Self {
            services: Arc::new(HashMap::new()),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(tools),
            native_tools: Arc::new(HashMap::new()),
        }
    }

    /// Sanitize tool names for `OpenAI` API compatibility.
    fn sanitize_tool_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    // Replace dots, colons, and any other invalid chars with underscore
                    '_'
                }
            })
            .collect()
    }

    /// Return namespaced tools as `(namespaced_name, Tool)`
    pub fn tools(&self) -> &[(String, Tool)] {
        &self.tools
    }

    /// Merge another registry into this one, returning a new registry.
    /// This is used to combine global tools with skill-specific tools.
    pub fn merge(&self, other: &McpRegistry) -> Self {
        let mut services = (*self.services).clone();
        services.extend((*other.services).clone());

        let mut tool_index = (*self.tool_index).clone();
        tool_index.extend((*other.tool_index).clone());

        let mut tools = (*self.tools).clone();
        tools.extend((*other.tools).clone());

        let mut native_tools = (*self.native_tools).clone();
        native_tools.extend((*other.native_tools).clone());

        Self {
            services: Arc::new(services),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(tools),
            native_tools: Arc::new(native_tools),
        }
    }

    pub fn with_native_tool(self, tool: Arc<dyn NativeTool>) -> Self {
        let ns_name = Self::sanitize_tool_name(&format!("native__{}", tool.name()));

        let mut tools = (*self.tools).clone();
        let mcp_tool = Tool {
            name: tool.name().to_string().into(),
            description: Some(tool.description().to_string().into()),
            input_schema: Arc::new(
                tool.schema()
                    .as_object()
                    .unwrap_or(&serde_json::Map::new())
                    .clone(),
            ),
            title: None,
            output_schema: None,
            annotations: None,
            icons: None,
            meta: None,
        };
        tools.push((ns_name.clone(), mcp_tool));

        let mut native_tools = (*self.native_tools).clone();
        native_tools.insert(ns_name, tool);

        Self {
            services: self.services,     // Keep ref
            tool_index: self.tool_index, // Keep ref
            tools: Arc::new(tools),
            native_tools: Arc::new(native_tools),
        }
    }

    pub fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|(ns_name, t)| {
                // rmcp Tool uses input_schema as an Arc<JsonObject>; convert to serde_json.
                let params = serde_json::to_value(&*t.input_schema)
                    .unwrap_or_else(|_| serde_json::json!({"type":"object","properties":{}}));

                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": ns_name,
                        "description": t.description.as_deref().unwrap_or(""),
                        "parameters": params
                    }
                })
            })
            .collect()
    }

    /// Execute a namespaced tool, e.g. "`time__now`" or "`tavily__search`".
    pub async fn call_namespaced_tool(
        &self,
        namespaced_tool: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        if namespaced_tool == "mirror" {
            return Ok(arguments);
        }

        if let Some(tool) = self.native_tools.get(namespaced_tool) {
            return tool.call(arguments).await;
        }

        // 1. Lookup server + raw_tool_name
        let (server_name, raw_tool_name) = self
            .tool_index
            .get(namespaced_tool)
            .ok_or_else(|| anyhow!("unknown tool: {namespaced_tool}"))?
            .clone();

        if server_name == "test" {
            return Ok(serde_json::json!({
                "result": format!("executed test tool {} with args {:?}", raw_tool_name, arguments)
            }));
        }

        // 2. Lookup service
        let service = self
            .services
            .get(&server_name)
            .ok_or_else(|| anyhow!("missing server handle: {server_name}"))?;

        // 3. Call tool
        let args_obj = arguments.as_object().cloned();
        let res = service
            .call_tool(CallToolRequestParam {
                name: raw_tool_name.clone().into(),
                arguments: args_obj,
            })
            .await
            .with_context(|| format!("tools/call failed for {server_name}::{raw_tool_name}"))?;

        // 4. Return content (simplified)
        Ok(serde_json::to_value(res)?)
    }
}
