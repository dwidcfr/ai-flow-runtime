use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::session::Session;
use crate::state::RuntimeState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub flow_id: Option<String>,
    pub prompt_set: String,
    pub current_node: Option<String>,
    pub runtime_state: RuntimeState,
    pub active_module: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleIndexStats {
    pub module_id: String,
    pub document_count: usize,
}

impl SessionSummary {
    pub fn from_session(session: &Session, default_prompt_set: &str) -> Self {
        let prompt_set = session
            .get_metadata("prompt_set_id")
            .and_then(|v| v.as_str())
            .unwrap_or(default_prompt_set)
            .to_string();

        let created_at = session
            .get_metadata("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let last_activity = session.history.last().map(|e| e.timestamp);

        Self {
            session_id: session.session_id.clone(),
            flow_id: session.flow_id.clone(),
            prompt_set,
            current_node: session.current_node.clone(),
            runtime_state: session.runtime_state,
            active_module: session
                .active_module
                .as_ref()
                .map(|m| m.module_id.clone()),
            created_at,
            last_activity,
        }
    }
}
