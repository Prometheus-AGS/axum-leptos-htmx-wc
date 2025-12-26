use axum_leptos_htmx_wc::{config, server};
use std::sync::Arc;
use tauri::Manager;

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

            // Initialize the Axum server in a separate async task
            tauri::async_runtime::spawn(async move {
                // Load configuration (environment variables should be handled by the library internally or passed here)
                // For now, we rely on the library's internal loading mechanism which checks env vars
                let config = config::AppConfig::load().expect("Failed to load configuration");

                let llm_settings = match config::load_llm_settings() {
                    Ok(settings) => settings,
                    Err(e) => {
                        log::error!("Failed to load LLM settings: {}", e);
                        return;
                    }
                };

                log::info!(
                    "Starting embedded Axum server on port {}",
                    config.server.port
                );

                if let Err(e) = server::start_server(Arc::new(config), llm_settings).await {
                    log::error!("Axum server failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
