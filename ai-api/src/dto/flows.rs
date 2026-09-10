use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowDto {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowsResponse {
    pub flows: Vec<FlowDto>,
}
