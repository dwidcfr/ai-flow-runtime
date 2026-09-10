use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MessageRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MessageResponse {
    pub assistant_message: String,
    pub current_node: Option<String>,
    pub active_module: Option<super::session::ActiveModuleDto>,
    pub finish_conversation: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub router_decision: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<super::debug::MessageDebugDto>,
}
