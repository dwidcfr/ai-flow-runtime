use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompanyDto {
    pub id: String,
    pub name: String,
    pub prompt_set: String,
    pub knowledge_source: String,
    pub flow_count: usize,
    pub active_session_count: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompaniesResponse {
    pub companies: Vec<CompanyDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct CompanyDetailDto {
    pub id: String,
    pub name: String,
    pub prompt_set: String,
    pub prompt_set_name: Option<String>,
    pub knowledge_source: String,
    pub knowledge_section_count: usize,
    pub flows: Vec<String>,
    pub flow_count: usize,
    pub active_session_count: usize,
}
