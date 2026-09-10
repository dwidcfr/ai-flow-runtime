use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptSetDto {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptSetsResponse {
    pub default_set: String,
    pub sets: Vec<PromptSetDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptTemplateDto {
    pub system: String,
    pub developer: String,
    pub assembly: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptSetDetailDto {
    pub id: String,
    pub name: String,
    pub version: String,
    pub identity: PromptIdentityDto,
    pub router: PromptTemplateDto,
    pub response: PromptTemplateDto,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptIdentityDto {
    pub name: String,
    pub bank_name: String,
    pub role: String,
    pub language: String,
    pub communication_style: String,
    pub tone: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ReloadResponse {
    pub status: String,
    pub sets_count: usize,
}
