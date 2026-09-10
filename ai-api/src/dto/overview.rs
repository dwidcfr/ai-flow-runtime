use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct GeminiStatusDto {
    pub enabled: bool,
    pub configured: bool,
    pub model_router: String,
    pub model_response: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OverviewCountsDto {
    pub active_sessions: usize,
    pub flows: usize,
    pub prompt_sets: usize,
    pub modules: usize,
    pub search_index_documents: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct OverviewResponse {
    pub api_version: String,
    pub runtime_version: String,
    pub uptime_secs: u64,
    pub gemini: GeminiStatusDto,
    pub counts: OverviewCountsDto,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlatformComponentStatus {
    pub id: String,
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlatformStatusResponse {
    pub components: Vec<PlatformComponentStatus>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PlatformConfigResponse {
    pub api_url_hint: String,
    pub data_root: String,
    pub flows_dir: String,
    pub request_timeout_secs: u64,
    pub max_body_size: usize,
    pub default_prompt_set: String,
    pub loaded_flows: Vec<String>,
    pub loaded_prompt_sets: Vec<String>,
    pub default_modules: Vec<String>,
    pub gemini_model_router: String,
    pub gemini_model_response: String,
    pub include_router_decision: bool,
}
