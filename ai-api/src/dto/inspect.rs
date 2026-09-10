use std::collections::HashMap;

use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::dto::session::{ActiveModuleDto, RuntimeStateDto};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SessionInspectResponse {
    pub session_id: String,
    pub flow_id: Option<String>,
    pub current_node: Option<String>,
    pub paused_node: Option<String>,
    pub runtime_state: RuntimeStateDto,
    pub active_module: Option<ActiveModuleDto>,
    pub prompt_set: String,
    pub metadata: HashMap<String, Value>,
    pub client_data: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowInspectResponse {
    pub flow_id: Option<String>,
    pub current_node: Option<String>,
    pub paused_node: Option<String>,
    pub available_transitions: Vec<String>,
    pub node_payload: Value,
    pub is_end: bool,
    pub runtime_state: RuntimeStateDto,
}
