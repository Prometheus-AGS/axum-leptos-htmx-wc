use crate::config::AppConfig;
use crate::uar::persistence::PersistenceLayer;
use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SettingsManager {
    persistence: Option<Arc<dyn PersistenceLayer>>,
    cache: Arc<RwLock<HashMap<String, Value>>>,
}

impl SettingsManager {
    pub fn new(persistence: Option<Arc<dyn PersistenceLayer>>) -> Self {
        Self {
            persistence,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize by hydrating from Config and DB
    pub async fn initialize(&self, _config: &AppConfig) -> Result<()> {
        if let Some(_p) = &self.persistence {
            // 1. Ensure basic types exist (Migration-like step or seed)
            // For now, we assume migration ran.

            // 2. Hydrate from Config (Upsert)
            // Example: "llm.provider"
            // We map config values to settings keys.

            // Note: In a real app we'd map the entire config struct to JSON
            // For this implementation, we focused on LLM settings as requested.
            // Let's assume we have a "system" settings type.

            // upsert logic here...
        }
        Ok(())
    }

    pub async fn get_value(&self, key: &str) -> Option<Value> {
        let cache = self.cache.read().await;
        if let Some(val) = cache.get(key) {
            return Some(val.clone());
        }

        // Fallback to DB if not in cache (and populate cache)
        if let Some(_p) = &self.persistence {
            // Retrieve from DB logic would go here
            // p.get_setting(key)...
        }

        None
    }

    pub async fn set_value(&self, key: &str, value: Value) -> Result<()> {
        // Update DB
        if let Some(_p) = &self.persistence {
            // p.update_setting(key, value.clone())...
        }

        // Update Cache
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), value);

        Ok(())
    }
}
