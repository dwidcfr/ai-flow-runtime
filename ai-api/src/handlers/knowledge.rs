use std::fs;

use axum::Json;
use axum::extract::{Path, State};

use crate::dto::{
    KnowledgeSectionDto, KnowledgeSourceDetailResponse, KnowledgeSourceDto, KnowledgeSourcesResponse,
    ModuleDto, ModulesResponse,
};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

pub async fn list_modules(
    State(state): State<AppState>,
) -> Result<Json<ModulesResponse>, ApiError> {
    let modules = with_runtime_read(&state, |rt| Ok(rt.list_modules())).await?;
    Ok(Json(ModulesResponse {
        modules: modules
            .into_iter()
            .map(|m| ModuleDto {
                id: m.id,
                name: m.name,
                description: m.description,
            })
            .collect(),
    }))
}

pub async fn list_knowledge_sources(
    State(state): State<AppState>,
) -> Result<Json<KnowledgeSourcesResponse>, ApiError> {
    let company_dir = state.config.data_root.join("company");
    let mut sources = Vec::new();
    if company_dir.exists() {
        for entry in fs::read_dir(&company_dir).map_err(|e| ApiError::internal(e.to_string()))? {
            let entry = entry.map_err(|e| ApiError::internal(e.to_string()))?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            let rel = format!("company/{name}.yaml");
            sources.push(KnowledgeSourceDto {
                name: name.clone(),
                path: rel,
                section_count: section_count_for_file(&path),
            });
        }
    }
    sources.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(Json(KnowledgeSourcesResponse { sources }))
}

pub async fn get_knowledge_source(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<KnowledgeSourceDetailResponse>, ApiError> {
    let path = state.config.data_root.join("company").join(format!("{name}.yaml"));
    if !path.exists() {
        return Err(ApiError::invalid_request(format!(
            "knowledge source not found: {name}"
        )));
    }
    let content = fs::read_to_string(&path).map_err(|e| ApiError::internal(e.to_string()))?;
    let value: serde_yaml::Value =
        serde_yaml::from_str(&content).map_err(|e| ApiError::internal(e.to_string()))?;
    let sections = parse_sections(&value);
    let section_count = sections.len();
    Ok(Json(KnowledgeSourceDetailResponse {
        name,
        path: format!("company/{}.yaml", path.file_stem().and_then(|s| s.to_str()).unwrap_or("")),
        sections,
        section_count,
    }))
}

fn section_count_for_file(path: &std::path::Path) -> usize {
    let Ok(content) = fs::read_to_string(path) else {
        return 0;
    };
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return 0;
    };
    parse_sections(&value).len()
}

fn parse_sections(value: &serde_yaml::Value) -> Vec<KnowledgeSectionDto> {
    value
        .get("sections")
        .and_then(|v| v.as_mapping())
        .map(|map| {
            let mut sections: Vec<_> = map
                .iter()
                .map(|(k, v)| {
                    let id = k.as_str().unwrap_or("").to_string();
                    let description = v
                        .get("description")
                        .and_then(|d| d.as_str())
                        .map(|s| s.to_string());
                    let content = v
                        .get("content")
                        .and_then(|c| c.as_str())
                        .unwrap_or_default()
                        .to_string();
                    KnowledgeSectionDto {
                        id,
                        description,
                        content,
                    }
                })
                .collect();
            sections.sort_by(|a, b| a.id.cmp(&b.id));
            sections
        })
        .unwrap_or_default()
}
