use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::Path};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct McpConfig {
    #[serde(rename = "mcpServers")]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum McpServerEntry {
    Stdio {
        command: String,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        env: HashMap<String, String>,
    },
    RemoteHttp {
        url: String,
        #[serde(default)]
        env: HashMap<String, String>,
    },
}

pub fn load_mcp_config(path: impl AsRef<Path>) -> anyhow::Result<McpConfig> {
    let txt = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&txt)?)
}

/// Expand "${VAR}" placeholders from the process environment.
/// - If env var is missing, leaves the placeholder unchanged by default.
///   (You can choose to error instead—recommended for prod.)
pub fn expand_env_placeholders(input: &str) -> String {
    // Minimal, deterministic expansion:
    // Replace any exact pattern "${NAME}" occurrences.
    let mut out = input.to_string();
    // naive scan; good enough for config values
    for (k, v) in std::env::vars() {
        let needle = format!("${{{k}}}");
        if out.contains(&needle) {
            out = out.replace(&needle, &v);
        }
    }
    out
}

pub fn expand_env_map(map: &HashMap<String, String>) -> HashMap<String, String> {
    map.iter()
        .map(|(k, v)| (k.clone(), expand_env_placeholders(v)))
        .collect()
}
