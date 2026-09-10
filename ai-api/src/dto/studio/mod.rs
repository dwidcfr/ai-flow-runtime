use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StudioProjectDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub dirty: bool,
    pub published_version: u32,
    pub edited_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StudioProjectsResponse {
    pub projects: Vec<StudioProjectDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct StudioProjectDetailResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_set: String,
    pub knowledge_source: String,
    pub flows: Vec<String>,
    pub dirty: bool,
    pub published_version: u32,
    pub edited_at: Option<String>,
    pub bootstrapped_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_set: Option<String>,
    pub knowledge_source: Option<String>,
    pub flows: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateCompanyRequest {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_set: String,
    pub knowledge_source: String,
    pub flows: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DraftCompanyResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_set: String,
    pub knowledge_source: String,
    pub flows: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NodePositionDto {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FlowTransitionDto {
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FlowNodeDto {
    pub id: String,
    pub node_type: String,
    pub payload: serde_json::Value,
    pub transitions: Vec<FlowTransitionDto>,
    pub position: Option<NodePositionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DraftFlowDto {
    pub id: String,
    pub name: Option<String>,
    pub version: u32,
    pub initial: String,
    pub nodes: Vec<FlowNodeDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateFlowRequest {
    pub id: String,
    pub name: Option<String>,
    pub initial: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ValidationIssueDto {
    pub rule: String,
    pub message: String,
    pub node_id: Option<String>,
    pub action: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct FlowValidationResponse {
    pub valid: bool,
    pub issues: Vec<ValidationIssueDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeCategoryDto {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeDocumentDto {
    pub id: String,
    pub category_id: String,
    pub title: String,
    pub section: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DraftKnowledgeResponse {
    pub categories: Vec<KnowledgeCategoryDto>,
    pub documents: Vec<KnowledgeDocumentDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpsertDocumentRequest {
    pub id: Option<String>,
    pub category_id: String,
    pub title: String,
    pub section: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeSearchRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PromptTemplateDto {
    pub system: String,
    pub developer: String,
    pub assembly: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DraftPromptResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub identity: serde_json::Value,
    pub router: PromptTemplateDto,
    pub response: PromptTemplateDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdatePromptRequest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub identity: serde_json::Value,
    pub router: PromptTemplateDto,
    pub response: PromptTemplateDto,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PromptPreviewRequest {
    pub template: String,
    pub sample_context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PromptPreviewResponse {
    pub rendered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ClientTemplateResponse {
    pub name: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpsertClientRequest {
    pub name: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct PublishResponse {
    pub status: String,
    pub version: Option<u32>,
    pub stage: Option<String>,
    pub error: Option<String>,
    pub report: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VersionDto {
    pub version: u32,
    pub published_at: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VersionsResponse {
    pub versions: Vec<VersionDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct VersionDetailResponse {
    pub version: u32,
    pub published_at: String,
    pub files: Vec<String>,
    pub manifest: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectStatusResponse {
    pub project_id: String,
    pub dirty: bool,
    pub published_version: u32,
    pub edited_at: Option<String>,
    pub bootstrapped_at: Option<String>,
    pub draft_exists: bool,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ProjectAssetsResponse {
    pub project_id: String,
    pub company: DraftCompanyResponse,
    pub flows: Vec<String>,
    pub prompt_sets: Vec<String>,
    pub clients: Vec<String>,
    pub knowledge_documents: usize,
    pub knowledge_categories: usize,
    pub modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct KnowledgeReindexResponse {
    pub status: String,
    pub document_count: usize,
}
