use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum NormalizedEvent {
    RunStart {
        run_id: String,
        agent_id: String,
    },
    ChatDelta {
        run_id: String,
        text_delta: String,
    },
    ReasoningDelta {
        run_id: String,
        text_delta: String,
    },

    Citation {
        run_id: String,
        sources: Vec<CitationSource>,
    },
    MemoryRecall {
        run_id: String,
        items: Vec<MemoryItem>,
    },

    ToolStart {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        tool: String,
        input: serde_json::Value,
    },
    ToolDelta {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        delta: serde_json::Value,
    },
    ToolEnd {
        run_id: String,
        call_index: usize,
        tool_call_id: String,
        tool: String,
        output: serde_json::Value,
        ok: bool,
    },

    Artifact {
        run_id: String,
        artifact: ArtifactPayload,
    },

    Error {
        run_id: String,
        code: String,
        message: String,
    },
    RunDone {
        run_id: String,
    },
    StatePatch {
        run_id: String,
        patch: Vec<StatePatchOp>,
    },
    ContextAction(super::context::ContextAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CitationSource {
    pub title: String,
    pub url: String,
    pub snippet: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryItem {
    pub key: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactPayload {
    pub artifact_id: String,
    pub artifact_type: String,
    pub title: String,
    pub content: String,
    pub language: Option<String>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatePatchOp {
    pub op: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}
