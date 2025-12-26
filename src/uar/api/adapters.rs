use crate::uar::domain::events::NormalizedEvent;
use serde_json::{Value, json};

pub fn to_ag_ui(event: &NormalizedEvent) -> Value {
    match event {
        NormalizedEvent::ChatDelta { run_id, text_delta } => json!({
            "type": "token.delta",
            "id": run_id,
            "payload": { "delta": text_delta }
        }),
        NormalizedEvent::ToolStart {
            run_id,
            tool_call_id,
            tool,
            input,
        } => json!({
            "type": "tool.call",
            "id": run_id,
            "payload": {
                "call_id": tool_call_id,
                "tool": tool,
                "args": input
            }
        }),
        NormalizedEvent::Artifact { run_id, artifact } => json!({
            "type": "ui.render",
            "id": run_id,
            "payload": {
                "schema": "a2ui.v1",
                "component": "artifact",
                "props": artifact
            }
        }),
        // Default fallthrough to raw event
        other => serde_json::to_value(other).unwrap_or(json!({"error": "serialization_failed"})),
    }
}
