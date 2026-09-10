use serde::Serialize;
use utoipa::ToSchema;

use crate::dto::session::{ActiveModuleDto, RuntimeStateDto};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SessionListItemDto {
    pub session_id: String,
    pub flow_id: Option<String>,
    pub prompt_set: String,
    pub current_node: Option<String>,
    pub runtime_state: RuntimeStateDto,
    pub active_module: Option<ActiveModuleDto>,
    pub created_at: Option<String>,
    pub last_activity: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SessionsListResponse {
    pub sessions: Vec<SessionListItemDto>,
}
