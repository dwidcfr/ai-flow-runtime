use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use ai_flow_runtime::{COMPANY_MODULE_ID, SMALLTALK_MODULE_ID};
use ai_prompts::{RenderVars, TemplateRenderer};
use ai_prompts::types::MissingPlaceholderPolicy;

use crate::companies::CompaniesRegistry;
use crate::dto::debug::SearchCandidateDto;
use crate::dto::search::SearchResponse;
use crate::dto::studio::*;
use crate::errors::ApiError;
use crate::state::{with_runtime, AppState};
use crate::studio::bootstrap::{bootstrap_new_project, bootstrap_project};
use crate::studio::convert::{knowledge_to_yaml, validate_draft_flow};
use crate::studio::mappers::{
    company_to_dto, flow_from_dto, flow_to_dto, knowledge_from_dto, knowledge_to_dto,
    prompt_to_dto, template_from_dto,
};
use crate::studio::publish::execute_publish;
use crate::studio::store::StudioStore;
use crate::studio::types::{
    DraftCompany, DraftFlow, DraftFlowNode, DraftFlowTransition, DraftManifest, DraftPromptSet,
    KnowledgeDocument, NodePosition,
};

async fn smoke_reindex_knowledge(state: &AppState, project_id: &str) -> Result<usize, ApiError> {
    let knowledge = state
        .studio
        .read_knowledge(project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let doc_count = knowledge.documents.len();
    if doc_count == 0 {
        return Ok(0);
    }
    let yaml = knowledge_to_yaml(&knowledge);
    let temp_path = state
        .studio
        .draft_dir(project_id)
        .join("_reindex_smoke.yaml");
    std::fs::write(&temp_path, yaml).map_err(|e| ApiError::internal(e.to_string()))?;

    let flow_id = state
        .flow_registry
        .list()
        .first()
        .map(|f| f.id.clone())
        .ok_or_else(|| ApiError::internal("no flows loaded"))?;
    let client_path = state
        .config
        .resolve_data_path("clients/sample.json")
        .to_string_lossy()
        .to_string();
    let temp = temp_path.to_string_lossy().to_string();

    let result = with_runtime(state, move |rt| {
        let session_id = rt.create_session(&flow_id, &client_path)?;
        rt.load_session_module(&session_id, COMPANY_MODULE_ID, &temp)?;
        rt.build_session_index(&session_id)?;
        rt.destroy_session(&session_id)?;
        Ok(())
    })
    .await;

    let _ = std::fs::remove_file(&temp_path);
    result?;
    Ok(doc_count)
}

async fn ensure_bootstrapped(state: &AppState, project_id: &str) -> Result<(), ApiError> {
    if state.studio.draft_exists(project_id) {
        return Ok(());
    }
    let entry = state
        .companies_registry
        .get(project_id)
        .ok_or_else(|| ApiError::invalid_request(format!("project not found: {project_id}")))?
        .clone();
    let store = state.studio.clone();
    let config = state.config.clone();
    let runtime = state.runtime.clone();
    tokio::task::spawn_blocking(move || {
        let rt = runtime.blocking_lock();
        bootstrap_project(&store, &config, &rt, &entry)
    })
    .await
    .map_err(|e| ApiError::internal(format!("bootstrap task failed: {e}")))?
    .map_err(|e| ApiError::internal(e))?;
    Ok(())
}

#[utoipa::path(
    get,
    path = "/studio/projects",
    responses((status = 200, description = "List studio projects", body = StudioProjectsResponse)),
    tag = "studio"
)]
pub async fn list_projects(
    State(state): State<AppState>,
) -> Result<Json<StudioProjectsResponse>, ApiError> {
    let registry =
        CompaniesRegistry::load(&state.config.data_root).map_err(|e| ApiError::internal(e.to_string()))?;
    let mut projects = Vec::new();
    for entry in registry.list() {
        let (dirty, published_version, edited_at) = if state.studio.draft_exists(&entry.id) {
            let m = state
                .studio
                .read_manifest(&entry.id)
                .unwrap_or_else(|_| DraftManifest::new(entry.id.clone()));
            (m.dirty, m.published_version, m.edited_at)
        } else {
            (false, state.studio.latest_version(&entry.id), None)
        };
        projects.push(StudioProjectDto {
            id: entry.id.clone(),
            name: entry.name.clone(),
            description: None,
            dirty,
            published_version,
            edited_at,
        });
    }
    Ok(Json(StudioProjectsResponse { projects }))
}

#[utoipa::path(
    post,
    path = "/studio/projects",
    request_body = CreateProjectRequest,
    responses((status = 201, description = "Project created", body = StudioProjectDetailResponse)),
    tag = "studio"
)]
pub async fn create_project(
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<StudioProjectDetailResponse>), ApiError> {
    if body.id.trim().is_empty() {
        return Err(ApiError::invalid_request("id must not be empty"));
    }
    let company = DraftCompany {
        id: body.id.clone(),
        name: body.name.clone(),
        description: body.description.clone(),
        prompt_set: body.prompt_set.clone().unwrap_or_else(|| body.id.clone()),
        knowledge_source: body
            .knowledge_source
            .clone()
            .unwrap_or_else(|| format!("company/{}.yaml", body.id)),
        flows: body.flows.clone().unwrap_or_default(),
    };

    let mut registry =
        CompaniesRegistry::load(&state.config.data_root).map_err(|e| ApiError::internal(e.to_string()))?;
    if registry.get(&company.id).is_some() {
        return Err(ApiError::invalid_request("project already exists"));
    }
    registry
        .entries_mut()
        .push(StudioStore::registry_entry_from_company(&company));
    registry
        .save(&state.config.data_root)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let manifest = bootstrap_new_project(&state.studio, company.clone())
        .map_err(|e| ApiError::internal(e))?;

    Ok((
        StatusCode::CREATED,
        Json(StudioProjectDetailResponse {
            id: company.id,
            name: company.name,
            description: company.description,
            prompt_set: company.prompt_set,
            knowledge_source: company.knowledge_source,
            flows: company.flows,
            dirty: manifest.dirty,
            published_version: manifest.published_version,
            edited_at: manifest.edited_at,
            bootstrapped_at: manifest.bootstrapped_at,
        }),
    ))
}

#[utoipa::path(
    patch,
    path = "/studio/projects/{project_id}",
    params(("project_id" = String, Path, description = "Project id")),
    request_body = UpdateProjectRequest,
    responses((status = 200, description = "Project updated", body = StudioProjectDetailResponse)),
    tag = "studio"
)]
pub async fn update_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<UpdateProjectRequest>,
) -> Result<Json<StudioProjectDetailResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let mut company = state
        .studio
        .read_company(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    if let Some(name) = body.name {
        if name.trim().is_empty() {
            return Err(ApiError::invalid_request("name must not be empty"));
        }
        company.name = name;
    }
    if body.description.is_some() {
        company.description = body.description;
    }
    state
        .studio
        .write_company(&project_id, &company)
        .map_err(|e| ApiError::internal(e))?;

    let mut registry =
        CompaniesRegistry::load(&state.config.data_root).map_err(|e| ApiError::internal(e.to_string()))?;
    if let Some(entry) = registry.entries_mut().iter_mut().find(|e| e.id == project_id) {
        entry.name = company.name.clone();
    }
    registry
        .save(&state.config.data_root)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let manifest = state
        .studio
        .read_manifest(&project_id)
        .unwrap_or_else(|_| DraftManifest::new(project_id.clone()));
    Ok(Json(StudioProjectDetailResponse {
        id: company.id,
        name: company.name,
        description: company.description,
        prompt_set: company.prompt_set,
        knowledge_source: company.knowledge_source,
        flows: company.flows,
        dirty: manifest.dirty,
        published_version: manifest.published_version,
        edited_at: manifest.edited_at,
        bootstrapped_at: manifest.bootstrapped_at,
    }))
}

#[utoipa::path(
    delete,
    path = "/studio/projects/{project_id}",
    params(("project_id" = String, Path, description = "Project id")),
    responses((status = 204, description = "Project deleted")),
    tag = "studio"
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut registry =
        CompaniesRegistry::load(&state.config.data_root).map_err(|e| ApiError::internal(e.to_string()))?;
    if registry.get(&project_id).is_none() {
        return Err(ApiError::invalid_request(format!("project not found: {project_id}")));
    }
    registry.entries_mut().retain(|e| e.id != project_id);
    registry
        .save(&state.config.data_root)
        .map_err(|e| ApiError::internal(e.to_string()))?;
    state
        .studio
        .delete_project(&project_id)
        .map_err(|e| ApiError::internal(e))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/studio/projects/{project_id}/status",
    params(("project_id" = String, Path, description = "Project id")),
    responses((status = 200, description = "Draft status", body = ProjectStatusResponse)),
    tag = "studio"
)]
pub async fn get_project_status(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectStatusResponse>, ApiError> {
    if state.companies_registry.get(&project_id).is_none() {
        return Err(ApiError::invalid_request(format!("project not found: {project_id}")));
    }
    let draft_exists = state.studio.draft_exists(&project_id);
    let manifest = if draft_exists {
        state
            .studio
            .read_manifest(&project_id)
            .unwrap_or_else(|_| DraftManifest::new(project_id.clone()))
    } else {
        DraftManifest::new(project_id.clone())
    };
    Ok(Json(ProjectStatusResponse {
        project_id,
        dirty: manifest.dirty,
        published_version: if draft_exists {
            manifest.published_version
        } else {
            state.studio.latest_version(&manifest.project_id)
        },
        edited_at: manifest.edited_at,
        bootstrapped_at: manifest.bootstrapped_at,
        draft_exists,
    }))
}

#[utoipa::path(
    get,
    path = "/studio/projects/{project_id}/assets",
    params(("project_id" = String, Path, description = "Project id")),
    responses((status = 200, description = "Project assets", body = ProjectAssetsResponse)),
    tag = "studio"
)]
pub async fn get_project_assets(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<ProjectAssetsResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let company = state
        .studio
        .read_company(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let flows = state
        .studio
        .list_flow_ids(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let prompt_sets = state
        .studio
        .list_prompt_ids(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let clients = state
        .studio
        .list_client_names(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let knowledge = state
        .studio
        .read_knowledge(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let modules: Vec<String> = state
        .config
        .default_modules
        .iter()
        .map(|m| m.module_id.clone())
        .collect();
    Ok(Json(ProjectAssetsResponse {
        project_id: project_id.clone(),
        company: company_to_dto(&company),
        flows,
        prompt_sets,
        clients,
        knowledge_documents: knowledge.documents.len(),
        knowledge_categories: knowledge.categories.len(),
        modules,
    }))
}

#[utoipa::path(
    get,
    path = "/studio/projects/{project_id}",
    params(("project_id" = String, Path, description = "Project id")),
    responses((status = 200, description = "Project detail", body = StudioProjectDetailResponse)),
    tag = "studio"
)]
pub async fn get_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<StudioProjectDetailResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let company = state
        .studio
        .read_company(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let manifest = state
        .studio
        .read_manifest(&project_id)
        .unwrap_or_else(|_| DraftManifest::new(project_id.clone()));
    Ok(Json(StudioProjectDetailResponse {
        id: company.id,
        name: company.name,
        description: company.description,
        prompt_set: company.prompt_set,
        knowledge_source: company.knowledge_source,
        flows: company.flows,
        dirty: manifest.dirty,
        published_version: manifest.published_version,
        edited_at: manifest.edited_at,
        bootstrapped_at: manifest.bootstrapped_at,
    }))
}

pub async fn get_company(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<DraftCompanyResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let company = state
        .studio
        .read_company(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(company_to_dto(&company)))
}

pub async fn update_company(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<UpdateCompanyRequest>,
) -> Result<Json<DraftCompanyResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let company = DraftCompany {
        id: body.id,
        name: body.name,
        description: body.description,
        prompt_set: body.prompt_set,
        knowledge_source: body.knowledge_source,
        flows: body.flows,
    };
    state
        .studio
        .write_company(&project_id, &company)
        .map_err(|e| ApiError::internal(e))?;
    Ok(Json(company_to_dto(&company)))
}

pub async fn get_knowledge(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<DraftKnowledgeResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let knowledge = state
        .studio
        .read_knowledge(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(knowledge_to_dto(&knowledge)))
}

pub async fn put_knowledge(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<DraftKnowledgeResponse>,
) -> Result<Json<DraftKnowledgeResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let knowledge = knowledge_from_dto(&body);
    state
        .studio
        .write_knowledge(&project_id, &knowledge)
        .map_err(|e| ApiError::internal(e))?;
    let _ = smoke_reindex_knowledge(&state, &project_id).await;
    Ok(Json(knowledge_to_dto(&knowledge)))
}

pub async fn upsert_document(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<UpsertDocumentRequest>,
) -> Result<Json<DraftKnowledgeResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let mut knowledge = state
        .studio
        .read_knowledge(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let doc_id = body.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    let doc = KnowledgeDocument {
        id: doc_id,
        category_id: body.category_id,
        title: body.title,
        section: body.section,
        content: body.content,
        metadata: body.metadata.unwrap_or(json!({})),
    };
    if let Some(idx) = knowledge.documents.iter().position(|d| d.id == doc.id) {
        knowledge.documents[idx] = doc;
    } else {
        knowledge.documents.push(doc);
    }
    state
        .studio
        .write_knowledge(&project_id, &knowledge)
        .map_err(|e| ApiError::internal(e))?;
    let _ = smoke_reindex_knowledge(&state, &project_id).await;
    Ok(Json(knowledge_to_dto(&knowledge)))
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path((project_id, doc_id)): Path<(String, String)>,
) -> Result<Json<DraftKnowledgeResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let mut knowledge = state
        .studio
        .read_knowledge(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    knowledge.documents.retain(|d| d.id != doc_id);
    state
        .studio
        .write_knowledge(&project_id, &knowledge)
        .map_err(|e| ApiError::internal(e))?;
    let _ = smoke_reindex_knowledge(&state, &project_id).await;
    Ok(Json(knowledge_to_dto(&knowledge)))
}

pub async fn knowledge_reindex(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<KnowledgeReindexResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let count = smoke_reindex_knowledge(&state, &project_id).await?;
    Ok(Json(KnowledgeReindexResponse {
        status: "indexed".into(),
        document_count: count,
    }))
}

pub async fn knowledge_search(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<KnowledgeSearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    if body.query.trim().is_empty() {
        return Err(ApiError::invalid_request("query must not be empty"));
    }
    let knowledge = state
        .studio
        .read_knowledge(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let yaml = knowledge_to_yaml(&knowledge);
    let temp_path = state
        .studio
        .draft_dir(&project_id)
        .join("_search_preview.yaml");
    std::fs::write(&temp_path, yaml).map_err(|e| ApiError::internal(e.to_string()))?;

    let flow_id = state
        .flow_registry
        .list()
        .first()
        .map(|f| f.id.clone())
        .ok_or_else(|| ApiError::internal("no flows loaded"))?;
    let client_path = state
        .config
        .resolve_data_path("clients/sample.json")
        .to_string_lossy()
        .to_string();
    let smalltalk_path = state
        .config
        .default_modules
        .iter()
        .find(|m| m.module_id == "smalltalk")
        .map(|m| state.config.resolve_data_path(&m.path).to_string_lossy().to_string());
    let query = body.query.clone();
    let top_k = body.top_k;
    let temp = temp_path.to_string_lossy().to_string();

    let results = with_runtime(&state, move |rt| {
        let session_id = rt.create_session(&flow_id, &client_path)?;
        let _ = rt.load_session_module(&session_id, COMPANY_MODULE_ID, &temp);
        if let Some(ref path) = smalltalk_path {
            let _ = rt.load_session_module(&session_id, SMALLTALK_MODULE_ID, path);
        }
        rt.build_session_index(&session_id)?;
        let results = rt.search_session(&session_id, &query, None, top_k)?;
        rt.destroy_session(&session_id)?;
        Ok(results)
    })
    .await?;

    let _ = std::fs::remove_file(&temp_path);
    Ok(Json(SearchResponse {
        results: results
            .into_iter()
            .map(|r| SearchCandidateDto {
                module_id: r.module_id,
                section_id: r.section_id,
                score: r.score,
                title: r.title,
                snippet: r.snippet,
            })
            .collect(),
    }))
}

pub async fn list_flows(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let ids = state
        .studio
        .list_flow_ids(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(json!({ "flows": ids })))
}

pub async fn get_flow(
    State(state): State<AppState>,
    Path((project_id, flow_id)): Path<(String, String)>,
) -> Result<Json<DraftFlowDto>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let flow = state
        .studio
        .read_flow(&project_id, &flow_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(flow_to_dto(&flow)))
}

pub async fn put_flow(
    State(state): State<AppState>,
    Path((project_id, flow_id)): Path<(String, String)>,
    Json(body): Json<DraftFlowDto>,
) -> Result<Json<DraftFlowDto>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    if body.id != flow_id {
        return Err(ApiError::invalid_request("flow id mismatch"));
    }
    let flow = flow_from_dto(&body);
    let (valid, issues) = validate_draft_flow(&flow);
    if !valid {
        return Err(ApiError::invalid_request(format!(
            "flow validation failed: {:?}",
            issues
        )));
    }
    state
        .studio
        .write_flow(&project_id, &flow)
        .map_err(|e| ApiError::internal(e))?;
    Ok(Json(flow_to_dto(&flow)))
}

pub async fn delete_flow(
    State(state): State<AppState>,
    Path((project_id, flow_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    state
        .studio
        .delete_flow(&project_id, &flow_id)
        .map_err(|e| ApiError::internal(e))?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn create_flow(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<CreateFlowRequest>,
) -> Result<(StatusCode, Json<DraftFlowDto>), ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let initial = body.initial.clone().unwrap_or_else(|| "start".into());
    let flow = DraftFlow {
        id: body.id.clone(),
        name: body.name.clone(),
        version: 1,
        initial: initial.clone(),
        nodes: vec![
            DraftFlowNode {
                id: initial.clone(),
                node_type: "flow".into(),
                payload: json!({ "task": "New Node" }),
                transitions: vec![DraftFlowTransition {
                    action: "finish".into(),
                    target: "end".into(),
                }],
                position: Some(NodePosition { x: 120.0, y: 80.0 }),
            },
            DraftFlowNode {
                id: "end".into(),
                node_type: "end".into(),
                payload: json!({}),
                transitions: vec![],
                position: Some(NodePosition { x: 120.0, y: 220.0 }),
            },
        ],
    };
    state
        .studio
        .write_flow(&project_id, &flow)
        .map_err(|e| ApiError::internal(e))?;
    let mut company = state
        .studio
        .read_company(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    if !company.flows.contains(&flow.id) {
        company.flows.push(flow.id.clone());
        state
            .studio
            .write_company(&project_id, &company)
            .map_err(|e| ApiError::internal(e))?;
    }
    Ok((StatusCode::CREATED, Json(flow_to_dto(&flow))))
}

#[utoipa::path(
    post,
    path = "/studio/projects/{project_id}/flows/validate",
    request_body = DraftFlowDto,
    responses((status = 200, description = "Validation result", body = FlowValidationResponse)),
    tag = "studio"
)]
pub async fn validate_flow_handler(
    Json(body): Json<DraftFlowDto>,
) -> Result<Json<FlowValidationResponse>, ApiError> {
    let flow = flow_from_dto(&body);
    let (valid, issues) = validate_draft_flow(&flow);
    Ok(Json(FlowValidationResponse { valid, issues }))
}

pub async fn get_prompt(
    State(state): State<AppState>,
    Path((project_id, set_id)): Path<(String, String)>,
) -> Result<Json<DraftPromptResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let prompt = state
        .studio
        .read_prompt(&project_id, &set_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(prompt_to_dto(&prompt)))
}

pub async fn put_prompt(
    State(state): State<AppState>,
    Path((project_id, set_id)): Path<(String, String)>,
    Json(body): Json<UpdatePromptRequest>,
) -> Result<Json<DraftPromptResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    if body.id != set_id {
        return Err(ApiError::invalid_request("prompt set id mismatch"));
    }
    let prompt = DraftPromptSet {
        id: body.id,
        name: body.name,
        version: body.version,
        identity: body.identity,
        router: template_from_dto(&body.router),
        response: template_from_dto(&body.response),
    };
    state
        .studio
        .write_prompt(&project_id, &prompt)
        .map_err(|e| ApiError::internal(e))?;
    Ok(Json(prompt_to_dto(&prompt)))
}

pub async fn preview_prompt(
    State(state): State<AppState>,
    Path((project_id, set_id)): Path<(String, String)>,
    Json(body): Json<PromptPreviewRequest>,
) -> Result<Json<PromptPreviewResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let prompt = state
        .studio
        .read_prompt(&project_id, &set_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let template = match body.template.as_str() {
        "response" => &prompt.response.assembly,
        _ => &prompt.router.assembly,
    };
    let mut vars = RenderVars::default();
    if let Some(ctx) = &body.sample_context {
        if let Some(obj) = ctx.as_object() {
            for (k, v) in obj {
                vars.insert(k.clone(), v.as_str().unwrap_or(&v.to_string()).to_string());
            }
        }
    } else {
        vars.insert("identity", "Operator: Alex\nOrganization: Acme Insurance");
        vars.insert("history", "user: да, это я");
        vars.insert("search", "payment methods");
        vars.insert("flow", "payment");
        vars.insert("user_message", "как оплатить?");
        vars.insert("context_json", "{\"flow\":{\"node\":\"payment\"}}");
    }
    let renderer = TemplateRenderer::new(MissingPlaceholderPolicy::Empty);
    let rendered = renderer.render(template, &vars);
    Ok(Json(PromptPreviewResponse { rendered }))
}

pub async fn list_clients(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let names = state
        .studio
        .list_client_names(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(json!({ "clients": names })))
}

pub async fn get_client(
    State(state): State<AppState>,
    Path((project_id, name)): Path<(String, String)>,
) -> Result<Json<ClientTemplateResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    let data = state
        .studio
        .read_client(&project_id, &name)
        .map_err(|e| ApiError::invalid_request(e))?;
    Ok(Json(ClientTemplateResponse { name, data }))
}

pub async fn upsert_client(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(body): Json<UpsertClientRequest>,
) -> Result<Json<ClientTemplateResponse>, ApiError> {
    upsert_client_impl(&state, &project_id, body).await
}

pub async fn upsert_client_named(
    State(state): State<AppState>,
    Path((project_id, name)): Path<(String, String)>,
    Json(body): Json<UpsertClientRequest>,
) -> Result<Json<ClientTemplateResponse>, ApiError> {
    let mut body = body;
    body.name = name;
    upsert_client_impl(&state, &project_id, body).await
}

async fn upsert_client_impl(
    state: &AppState,
    project_id: &str,
    body: UpsertClientRequest,
) -> Result<Json<ClientTemplateResponse>, ApiError> {
    ensure_bootstrapped(state, project_id).await?;
    state
        .studio
        .write_client(project_id, &body.name, &body.data)
        .map_err(|e| ApiError::internal(e))?;
    Ok(Json(ClientTemplateResponse {
        name: body.name,
        data: body.data,
    }))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path((project_id, name)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    state
        .studio
        .delete_client(&project_id, &name)
        .map_err(|e| ApiError::internal(e))?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/studio/projects/{project_id}/publish",
    params(("project_id" = String, Path, description = "Project id")),
    responses((status = 200, description = "Publish result", body = PublishResponse)),
    tag = "studio"
)]
pub async fn publish_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<PublishResponse>, ApiError> {
    ensure_bootstrapped(&state, &project_id).await?;
    execute_publish(&state, &project_id).await.map(Json)
}

pub async fn list_versions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<VersionsResponse>, ApiError> {
    let versions = state
        .studio
        .list_versions(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?
        .into_iter()
        .map(|v| VersionDto {
            version: v.version,
            published_at: v.published_at,
            files: v.files,
        })
        .collect();
    Ok(Json(VersionsResponse { versions }))
}

pub async fn get_version(
    State(state): State<AppState>,
    Path((project_id, version)): Path<(String, u32)>,
) -> Result<Json<VersionDetailResponse>, ApiError> {
    let versions = state
        .studio
        .list_versions(&project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let manifest = versions
        .into_iter()
        .find(|v| v.version == version)
        .ok_or_else(|| ApiError::invalid_request(format!("version not found: {version}")))?;
    Ok(Json(VersionDetailResponse {
        version: manifest.version,
        published_at: manifest.published_at.clone(),
        files: manifest.files.clone(),
        manifest: serde_json::to_value(&manifest).unwrap_or(json!({})),
    }))
}

#[utoipa::path(
    post,
    path = "/flows/reload",
    responses((status = 200, description = "Flows reloaded")),
    tag = "flows"
)]
pub async fn reload_flows(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let flows_dir = state.config.flows_dir.clone();
    let ids = with_runtime(&state, move |rt| rt.reload_flows_from_dir(&flows_dir)).await?;
    Ok(Json(json!({ "status": "reloaded", "flows": ids })))
}
