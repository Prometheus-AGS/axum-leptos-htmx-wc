use axum_leptos_htmx_wc::config::AppConfig;
use axum_leptos_htmx_wc::uar::defaults::ensure_default_knowledge_base;
use axum_leptos_htmx_wc::uar::persistence::providers::{
    postgres::PostgresProvider, surreal::SurrealDbProvider,
};
use axum_leptos_htmx_wc::uar::persistence::PersistenceLayer;
use anyhow::{Context, Result};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::load().context("failed to load app configuration")?;

    if !config.file_processing.upload_dir.is_empty() {
        tokio::fs::create_dir_all(&config.file_processing.upload_dir)
            .await
            .context("failed to create upload directory")?;
    }

    let persistence: Arc<dyn PersistenceLayer> =
        if config.persistence.provider.as_str() == "surrealdb" {
            Arc::new(
                SurrealDbProvider::new(&config.persistence.database_url)
                    .await
                    .context("failed to initialize SurrealDB")?,
            )
        } else {
            Arc::new(
                PostgresProvider::new(&config.persistence.database_url)
                    .await
                    .context("failed to initialize Postgres")?,
            )
        };

    ensure_default_knowledge_base(&*persistence, config.knowledge_bases.default.as_ref())
        .await
        .context("failed to ensure default knowledge base")?;

    Ok(())
}
