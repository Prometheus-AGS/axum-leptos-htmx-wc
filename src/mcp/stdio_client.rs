use crate::mcp::types::{CallToolResult, ListToolsResult, McpTool};
use anyhow::{Context, anyhow};
use futures::FutureExt;
use serde_json::json;
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, Command},
    sync::{Mutex, mpsc, oneshot},
};

#[derive(Clone)]
pub struct StdioMcpClient {
    server_name: String,
    tx: mpsc::UnboundedSender<String>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
    next_id: Arc<AtomicU64>,
    _child: Arc<Mutex<Child>>, // keep process alive
}

impl StdioMcpClient {
    pub async fn spawn(
        server_name: String,
        command: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    ) -> anyhow::Result<Self> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit());

        for (k, v) in env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().context("failed to spawn MCP server process")?;
        let stdin = child.stdin.take().ok_or_else(|| anyhow!("missing stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("missing stdout"))?;

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Writer task: each JSON-RPC message is a single line, newline-terminated.
        tokio::spawn(async move {
            let mut w = stdin;
            while let Some(line) = rx.recv().await {
                // Ensure no embedded newlines
                let line = line.replace('\n', "");
                let _ = w.write_all(line.as_bytes()).await;
                let _ = w.write_all(b"\n").await;
                let _ = w.flush().await;
            }
        });

        // Reader task: read newline-delimited JSON-RPC messages.
        let pending_reader = pending.clone();
        tokio::spawn(async move {
            let mut r = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = r.next_line().await {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let parsed: serde_json::Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue, // ignore malformed stdout (server must not emit it, but be defensive)
                };

                // If it's a response with an id, route to waiter
                if let Some(id) = parsed.get("id").and_then(|x| x.as_u64()) {
                    if let Some(tx) = pending_reader.lock().await.remove(&id) {
                        let _ = tx.send(parsed);
                    }
                } else {
                    // notifications/requests from server could be handled here if desired
                }
            }
        });

        let client = Self {
            server_name,
            tx,
            pending,
            next_id: Arc::new(AtomicU64::new(1)),
            _child: Arc::new(Mutex::new(child)),
        };

        // Perform MCP lifecycle handshake
        client.initialize().await?;

        Ok(client)
    }

    async fn initialize(&self) -> anyhow::Result<()> {
        // initialize request as per lifecycle spec :contentReference[oaicite:9]{index=9}
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let init = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {
                    "roots": { "listChanged": true },
                    "sampling": {}
                },
                "clientInfo": {
                    "name": "PrometheusAxumServer",
                    "version": "0.1.0"
                }
            }
        });

        let resp = self.request_raw(id, init).await?;
        if resp.get("error").is_some() {
            return Err(anyhow!("MCP initialize error: {}", resp));
        }

        // After initialize response, send notifications/initialized :contentReference[oaicite:10]{index=10}
        let note = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.send_notification(note).await?;

        Ok(())
    }

    async fn send_notification(&self, msg: serde_json::Value) -> anyhow::Result<()> {
        let line = serde_json::to_string(&msg)?;
        self.tx
            .send(line)
            .map_err(|_| anyhow!("stdio writer task ended"))?;
        Ok(())
    }

    async fn request_raw(
        &self,
        id: u64,
        msg: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let line = serde_json::to_string(&msg)?;
        self.tx
            .send(line)
            .map_err(|_| anyhow!("stdio writer task ended"))?;

        // Basic timeout discipline recommended by spec :contentReference[oaicite:11]{index=11}
        let resp = tokio::time::timeout(std::time::Duration::from_secs(30), rx.map(|r| r.ok()))
            .await
            .map_err(|_| anyhow!("MCP request timeout"))?
            .ok_or_else(|| anyhow!("MCP request cancelled"))?;

        Ok(resp)
    }

    pub async fn list_tools_all(&self) -> anyhow::Result<Vec<McpTool>> {
        // tools/list request format :contentReference[oaicite:12]{index=12}
        let mut out = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let id = self.next_id.fetch_add(1, Ordering::Relaxed);
            let req = json!({
                "jsonrpc": "2.0",
                "id": id,
                "method": "tools/list",
                "params": {
                    "cursor": cursor
                }
            });

            let resp = self.request_raw(id, req).await?;
            let result = resp
                .get("result")
                .ok_or_else(|| anyhow!("missing result in tools/list"))?;
            let parsed: ListToolsResult = serde_json::from_value(result.clone())?;
            out.extend(parsed.tools);

            if let Some(nc) = parsed.next_cursor {
                cursor = Some(nc);
            } else {
                break;
            }
        }

        Ok(out)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> anyhow::Result<CallToolResult> {
        // tools/call request format :contentReference[oaicite:13]{index=13}
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": arguments
            }
        });

        let resp = self.request_raw(id, req).await?;
        if let Some(err) = resp.get("error") {
            return Err(anyhow!("tools/call error: {}", err));
        }
        let result = resp
            .get("result")
            .ok_or_else(|| anyhow!("missing result in tools/call"))?;
        Ok(serde_json::from_value(result.clone())?)
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }
}
