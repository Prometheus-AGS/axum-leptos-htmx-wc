use axum_leptos_htmx_wc::{config, server};
use std::net::TcpListener;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Manager;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            if let Ok(resource_dir) = app.path().resource_dir() {
                std::env::set_var("MCP_CONFIG_DIR", &resource_dir);
                let mcp_config_path = resource_dir.join("mcp.json");
                if mcp_config_path.exists() {
                    std::env::set_var("MCP_CONFIG_PATH", &mcp_config_path);
                }
                let mcp_server_dir = resource_dir.join("mcp-servers");
                std::env::set_var("MCP_SERVER_DIR", &mcp_server_dir);
            }

            let mut app_config = config::AppConfig::load().expect("Failed to load configuration");
            let host = "127.0.0.1".to_string();
            let port = resolve_localhost_port(app_config.server.port);
            let server_url = format!("http://{host}:{port}");

            // FORCE DISABLE RESILIENCE FEATURES FOR DESKTOP/MOBILE APP
            // User requirement: "turned off when run in tauri"
            app_config.resilience.rate_limit_enabled = false;
            app_config.resilience.timeout_disabled = true;
            app_config.server.host = host.clone();
            app_config.server.port = port;

            let llm_settings = match config::load_llm_settings() {
                Ok(settings) => settings,
                Err(e) => {
                    log::error!("Failed to load LLM settings: {}", e);
                    return Ok(());
                }
            };

            let app_handle = app.handle();

            // Initialize the Axum server in a separate async task
            tauri::async_runtime::spawn({
                let server_config = Arc::new(app_config);
                let server_url = server_url.clone();
                async move {
                    log::info!("Starting embedded Axum server on {server_url}");
                    if let Err(e) = server::start_server(server_config, llm_settings).await {
                        log::error!("Axum server failed: {}", e);
                    }
                }
            });

            // Navigate the webview to the localhost server once it's ready
            tauri::async_runtime::spawn({
                let app_handle = app_handle.clone();
                let host = host.clone();
                async move {
                    if wait_for_ready(&host, port, Duration::from_secs(20)).await {
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.eval(&format!(
                                "window.location.replace('{}')",
                                server_url
                            ));
                        }
                    } else {
                        log::error!("Timed out waiting for local server to start");
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn resolve_localhost_port(fallback: u16) -> u16 {
    if let Ok(port) = std::env::var("TAURI_LOCALHOST_PORT") {
        if let Ok(parsed) = port.parse::<u16>() {
            return parsed;
        }
    }

    if let Ok(listener) = TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
            return addr.port();
        }
    }

    fallback
}

async fn wait_for_ready(host: &str, port: u16, timeout: Duration) -> bool {
    let deadline = Instant::now() + timeout;
    let address = format!("{host}:{port}");

    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(&address).await {
            let request = format!(
                "GET /readyz HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n"
            );
            if stream.write_all(request.as_bytes()).await.is_ok() {
                let mut buf = [0_u8; 128];
                if let Ok(read) = stream.read(&mut buf).await {
                    let response = std::str::from_utf8(&buf[..read]).unwrap_or("");
                    if response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
                    {
                        return true;
                    }
                }
            }
        }

        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    false
}
