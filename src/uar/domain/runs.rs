use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Run {
    pub run_id: String,
    pub agent_id: String,
    pub conversation_id: Option<String>,
    pub user_id: Option<String>,
    pub status: RunStatus,
    pub context: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    Paused,
    Done,
    Error,
    Cancelled,
}
