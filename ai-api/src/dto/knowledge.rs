use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KnowledgeSourceDto {
    pub name: String,
    pub path: String,
    pub section_count: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KnowledgeSourcesResponse {
    pub sources: Vec<KnowledgeSourceDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KnowledgeSectionDto {
    pub id: String,
    pub description: Option<String>,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KnowledgeSourceDetailResponse {
    pub name: String,
    pub path: String,
    pub sections: Vec<KnowledgeSectionDto>,
    pub section_count: usize,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ModuleDto {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ModulesResponse {
    pub modules: Vec<ModuleDto>,
}
