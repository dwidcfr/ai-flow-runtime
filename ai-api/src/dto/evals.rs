use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalScenarioDto {
    pub name: String,
    pub domain: Option<String>,
    pub llm: Option<String>,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalScenariosResponse {
    pub scenarios: Vec<EvalScenarioDto>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct EvalRunRequest {
    pub scenario: Option<String>,
    pub domain: Option<String>,
    #[serde(default)]
    pub llm: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct AssertionFailureDto {
    pub field: String,
    pub expected: String,
    pub actual: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalScenarioResultDto {
    pub name: String,
    pub domain: Option<String>,
    pub status: String,
    pub skip_reason: Option<String>,
    pub failures: Vec<AssertionFailureDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalMetricsDto {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalReportDto {
    pub metrics: EvalMetricsDto,
    pub results: Vec<EvalScenarioResultDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalRunResponse {
    pub run_id: Option<String>,
    pub status: String,
    pub report: Option<EvalReportDto>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalAsyncRunResponse {
    pub run_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct EvalRunStatusResponse {
    pub run_id: String,
    pub status: String,
    pub report: Option<EvalReportDto>,
    pub error: Option<String>,
}
