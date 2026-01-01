use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct SettingsType {
    pub id: Uuid,
    pub name: String,
    pub key: String,
    pub description: Option<String>,
    pub display_mode: String,  // 'form' or 'master-detail'
    pub schema: Option<Value>, // JSONB schema
    pub icon_url: Option<String>,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Settings {
    pub id: Uuid,
    pub settings_type_id: Uuid,
    pub name: String,
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub data: Value, // JSONB data
}

impl SettingsType {
    pub fn new(name: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            key: key.into(),
            description: None,
            display_mode: "form".to_string(),
            schema: None,
            icon_url: None,
        }
    }
}

impl Settings {
    pub fn new(
        type_id: Uuid,
        name: impl Into<String>,
        key: impl Into<String>,
        data: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            settings_type_id: type_id,
            name: name.into(),
            key: key.into(),
            created_at: Utc::now(),
            updated_at: None,
            data,
        }
    }
}
