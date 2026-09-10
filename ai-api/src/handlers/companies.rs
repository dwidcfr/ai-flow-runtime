use axum::Json;
use axum::extract::{Path, State};

use crate::dto::{CompaniesResponse, CompanyDetailDto, CompanyDto};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

fn knowledge_section_count(data_root: &std::path::Path, source: &str) -> usize {
    let path = data_root.join(source);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return 0;
    };
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return 0;
    };
    value
        .get("sections")
        .and_then(|v| v.as_mapping())
        .map(|m| m.len())
        .unwrap_or(0)
}

async fn active_sessions_for_prompt_set(
    state: &AppState,
    prompt_set: &str,
) -> Result<usize, ApiError> {
    let prompt_set = prompt_set.to_string();
    with_runtime_read(state, move |rt| {
        let sessions = rt.list_sessions()?;
        Ok(sessions
            .iter()
            .filter(|s| s.prompt_set == prompt_set)
            .count())
    })
    .await
}

pub async fn list_companies(
    State(state): State<AppState>,
) -> Result<Json<CompaniesResponse>, ApiError> {
    let mut companies = Vec::new();
    for entry in state.companies_registry.list() {
        let active_session_count =
            active_sessions_for_prompt_set(&state, &entry.prompt_set).await?;
        companies.push(CompanyDto {
            id: entry.id.clone(),
            name: entry.name.clone(),
            prompt_set: entry.prompt_set.clone(),
            knowledge_source: entry.knowledge_source.clone(),
            flow_count: entry.flows.len(),
            active_session_count,
        });
    }
    Ok(Json(CompaniesResponse { companies }))
}

pub async fn get_company(
    State(state): State<AppState>,
    Path(company_id): Path<String>,
) -> Result<Json<CompanyDetailDto>, ApiError> {
    let entry = state
        .companies_registry
        .get(&company_id)
        .ok_or_else(|| ApiError::invalid_request(format!("company not found: {company_id}")))?;

    let active_session_count =
        active_sessions_for_prompt_set(&state, &entry.prompt_set).await?;

    let prompt_set_id = entry.prompt_set.clone();
    let prompt_set_name = with_runtime_read(&state, move |rt| {
        let registry_ref = rt.prompt_registry();
        let registry = registry_ref.read().map_err(|e| {
            ai_flow_runtime::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            }
        })?;
        let meta = registry
            .list_sets()
            .into_iter()
            .find(|m| m.id == prompt_set_id);
        Ok(meta.map(|m| m.name))
    })
    .await?;

    let knowledge_section_count =
        knowledge_section_count(&state.config.data_root, &entry.knowledge_source);

    Ok(Json(CompanyDetailDto {
        id: entry.id.clone(),
        name: entry.name.clone(),
        prompt_set: entry.prompt_set.clone(),
        prompt_set_name,
        knowledge_source: entry.knowledge_source.clone(),
        knowledge_section_count,
        flows: entry.flows.clone(),
        flow_count: entry.flows.len(),
        active_session_count,
    }))
}
