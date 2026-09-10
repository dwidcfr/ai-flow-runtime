use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowStatsDto {
    pub node_count: usize,
    pub end_node_count: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowEdgeDto {
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowNodeDto {
    pub id: String,
    pub node_type: String,
    pub payload: serde_json::Value,
    pub transitions: Vec<FlowEdgeDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowDetailResponse {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<u32>,
    pub initial: String,
    pub stats: FlowStatsDto,
    pub nodes: Vec<FlowNodeDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowGraphEdgeDto {
    pub from: String,
    pub to: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowGraphResponse {
    pub flow_id: String,
    pub initial: String,
    pub nodes: Vec<FlowNodeDto>,
    pub edges: Vec<FlowGraphEdgeDto>,
}
