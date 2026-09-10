use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::dto::debug::SearchCandidateDto;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SessionSearchRequest {
    pub query: String,
    #[serde(default)]
    pub module_ids: Vec<String>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct SearchInspectRequest {
    pub query: String,
    #[serde(default)]
    pub module_ids: Vec<String>,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SearchResponse {
    pub results: Vec<SearchCandidateDto>,
}
