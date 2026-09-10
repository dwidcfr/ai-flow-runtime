use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use ai_flow_runtime::{RuntimeState, Session};

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeStateDto {
    Idle,
    Flow,
    Paused,
    Module,
    Finished,
}

impl From<RuntimeState> for RuntimeStateDto {
    fn from(state: RuntimeState) -> Self {
        match state {
            RuntimeState::Idle => Self::Idle,
            RuntimeState::Flow => Self::Flow,
            RuntimeState::Paused => Self::Paused,
            RuntimeState::Module => Self::Module,
            RuntimeState::Finished => Self::Finished,
        }
    }
}


#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct CreateSessionRequest {
    pub flow_id: String,
    pub client_source_path: String,
    #[serde(default)]
    pub prompt_set: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub current_node: Option<String>,
    pub prompt_set: String,
    pub metadata: HashMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opening_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ActiveModuleDto {
    pub module_id: String,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SessionResponse {
    pub session_id: String,
    pub flow_id: Option<String>,
    pub current_node: Option<String>,
    pub runtime_state: RuntimeStateDto,
    pub active_module: Option<ActiveModuleDto>,
    pub metadata: HashMap<String, Value>,
}

impl SessionResponse {
    pub fn from_session(session: &Session) -> Self {
        Self {
            session_id: session.session_id.clone(),
            flow_id: session.flow_id.clone(),
            current_node: session.current_node.clone(),
            runtime_state: session.runtime_state.into(),
            active_module: session.active_module.as_ref().map(|m| ActiveModuleDto {
                module_id: m.module_id.clone(),
                section: m.section.clone(),
            }),
            metadata: session.metadata.clone(),
        }
    }
}

pub fn prompt_set_from_session(session: &Session, default_set: &str) -> String {
    session
        .get_metadata("prompt_set_id")
        .and_then(|v| v.as_str())
        .unwrap_or(default_set)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_session_request_deserializes_without_prompt_set() {
        let req: CreateSessionRequest =
            serde_json::from_str(r#"{"flow_id":"payment_flow","client_source_path":"clients/sample.json"}"#)
                .unwrap();
        assert_eq!(req.flow_id, "payment_flow");
        assert!(req.prompt_set.is_none());
    }
}
