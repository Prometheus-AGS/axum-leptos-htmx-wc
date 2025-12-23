use crate::mcp::config::{McpServerEntry, expand_env_map, load_mcp_config};
use anyhow::{Context, anyhow};
use rmcp::{
    model::{CallToolRequestParam, Tool},
    service::ServiceExt,
    transport::{StreamableHttpClientTransport, TokioChildProcess},
};
use std::{collections::HashMap, sync::Arc};
use tokio::process::Command;
use url::Url;

type DynClientService = rmcp::service::RunningService<
    rmcp::service::RoleClient,
    Box<dyn rmcp::service::DynService<rmcp::service::RoleClient>>,
>;

#[derive(Clone)]
pub struct McpRegistry {
    services: Arc<HashMap<String, DynClientService>>,
    // namespaced_tool_name -> (server_name, tool_name)
    tool_index: Arc<HashMap<String, (String, String)>>,
    tools: Arc<Vec<(String, Tool)>>, // (namespaced_name, Tool)
}

impl McpRegistry {
    pub async fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let cfg = load_mcp_config(path)?;

        // 1) connect all servers
        let mut services: HashMap<String, DynClientService> = HashMap::new();

        for (name, entry) in cfg.mcp_servers {
            let svc = match entry {
                McpServerEntry::Stdio { command, args, env } => {
                    let env = expand_env_map(&env);

                    let mut cmd = Command::new(command);
                    cmd.args(args);

                    for (k, v) in env {
                        cmd.env(k, v);
                    }

                    // rmcp docs show TokioChildProcess + configure pattern for adding args :contentReference[oaicite:3]{index=3}
                    let transport = TokioChildProcess::new(cmd)?;
                    // store as dyn to keep a homogeneous collection :contentReference[oaicite:4]{index=4}
                    ().into_dyn()
                        .serve(transport)
                        .await
                        .with_context(|| format!("failed to connect stdio MCP server '{name}'"))?
                }

                McpServerEntry::RemoteHttp { url, env } => {
                    let env = expand_env_map(&env);

                    // Tavily expects ?tavilyApiKey=... (per your config contract).
                    // Keep the key OUT of logs.
                    let api_key = env
                        .get("TAVILY_API_KEY")
                        .cloned()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| anyhow!("remote MCP '{name}' missing TAVILY_API_KEY"))?;

                    let mut u = Url::parse(&url)
                        .with_context(|| format!("invalid url for remote MCP '{name}': {url}"))?;

                    // If URL already has query, we just append.
                    u.query_pairs_mut().append_pair("tavilyApiKey", &api_key);

                    // rmcp streamable http transport from_uri :contentReference[oaicite:5]{index=5}
                    let transport = StreamableHttpClientTransport::from_uri(u.to_string());
                    ().into_dyn()
                        .serve(transport)
                        .await
                        .with_context(|| format!("failed to connect remote MCP server '{name}'"))?
                }
            };

            services.insert(name, svc);
        }

        // 2) list tools + build index
        let mut all_tools: Vec<(String, Tool)> = Vec::new();
        let mut tool_index: HashMap<String, (String, String)> = HashMap::new();

        for (server_name, svc) in services.iter() {
            // list_tools exists on the rmcp running service in examples
            let result = svc
                .list_tools(Default::default())
                .await
                .with_context(|| format!("tools/list failed for MCP server '{server_name}'"))?;

            for t in result.tools {
                let tool_name = t.name.to_string();
                let ns_name = format!("{server_name}::{tool_name}");
                tool_index.insert(ns_name.clone(), (server_name.clone(), tool_name));
                all_tools.push((ns_name, t));
            }
        }

        Ok(Self {
            services: Arc::new(services),
            tool_index: Arc::new(tool_index),
            tools: Arc::new(all_tools),
        })
    }

    /// Return namespaced tools as `(namespaced_name, Tool)`
    pub fn tools(&self) -> &[(String, Tool)] {
        &self.tools
    }

    /// Convert MCP tools to OpenAI "tools" schema (function tools).
    /// Works for both /v1/chat/completions and /v1/responses tool definitions.
    pub fn openai_tools_json(&self) -> Vec<serde_json::Value> {
        self.tools
            .iter()
            .map(|(ns_name, t)| {
                // rmcp Tool uses input_schema as an Arc<JsonObject>; convert to serde_json.
                // We round-trip through serde_json::Value via serde support in rmcp model types.
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

    /// Execute a namespaced tool, e.g. "time::now" or "tavily::search".
    /// `arguments` must be a JSON object for MCP tools/call.
    pub async fn call_namespaced_tool(
        &self,
        namespaced_tool: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (server, tool) = self
            .tool_index
            .get(namespaced_tool)
            .ok_or_else(|| anyhow!("unknown tool: {namespaced_tool}"))?
            .clone();

        let svc = self
            .services
            .get(&server)
            .ok_or_else(|| anyhow!("missing server handle: {server}"))?;

        let args_obj = arguments.as_object().cloned();

        let res = svc
            .call_tool(CallToolRequestParam {
                name: tool.clone().into(),
                arguments: args_obj,
            })
            .await
            .with_context(|| format!("tools/call failed for {server}::{tool}"))?;

        Ok(serde_json::to_value(res)?)
    }
}
