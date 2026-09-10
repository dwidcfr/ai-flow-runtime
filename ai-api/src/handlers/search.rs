use axum::Json;
use axum::extract::{Path, State};

use ai_flow_runtime::{SearchResult, COMPANY_MODULE_ID, SMALLTALK_MODULE_ID};

use crate::dto::debug::SearchCandidateDto;
use crate::dto::{SearchInspectRequest, SearchResponse, SessionSearchRequest};
use crate::errors::ApiError;
use crate::state::{with_runtime, AppState};

fn map_results(results: Vec<SearchResult>) -> Vec<SearchCandidateDto> {
    results
        .into_iter()
        .map(|r| SearchCandidateDto {
            module_id: r.module_id,
            section_id: r.section_id,
            score: r.score,
            title: r.title,
            snippet: r.snippet,
        })
        .collect()
}

pub async fn session_search(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(body): Json<SessionSearchRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    if body.query.trim().is_empty() {
        return Err(ApiError::invalid_request("query must not be empty"));
    }
    let query = body.query.clone();
    let module_ids_owned = body.module_ids.clone();
    let top_k = body.top_k;
    let results = with_runtime(&state, move |rt| {
        let module_ids = if module_ids_owned.is_empty() {
            None
        } else {
            Some(module_ids_owned.as_slice())
        };
        rt.search_session(
            &session_id,
            &query,
            module_ids,
            top_k,
        )
    })
    .await?;
    Ok(Json(SearchResponse {
        results: map_results(results),
    }))
}

pub async fn search_inspect(
    State(state): State<AppState>,
    Json(body): Json<SearchInspectRequest>,
) -> Result<Json<SearchResponse>, ApiError> {
    if body.query.trim().is_empty() {
        return Err(ApiError::invalid_request("query must not be empty"));
    }

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

    let knowledge_source = body.company_id.as_ref().and_then(|id| {
        state
            .companies_registry
            .get(id)
            .map(|c| c.knowledge_source.clone())
    });

    let company_module_path = knowledge_source
        .as_ref()
        .map(|rel| state.config.resolve_data_path(rel))
        .or_else(|| {
            state
                .config
                .default_modules
                .iter()
                .find(|m| m.module_id == "company")
                .map(|m| state.config.resolve_data_path(&m.path))
        });

    let smalltalk_path = state
        .config
        .default_modules
        .iter()
        .find(|m| m.module_id == "smalltalk")
        .map(|m| state.config.resolve_data_path(&m.path));

    let query = body.query.clone();
    let module_ids_owned = body.module_ids.clone();
    let top_k = body.top_k;

    let results = with_runtime(&state, move |rt| {
        let session_id = rt.create_session(&flow_id, &client_path)?;
        if let Some(path) = company_module_path.as_ref() {
            if let Some(p) = path.to_str() {
                let _ = rt.load_session_module(&session_id, COMPANY_MODULE_ID, p);
            }
        }
        if let Some(path) = smalltalk_path.as_ref() {
            if let Some(p) = path.to_str() {
                let _ = rt.load_session_module(&session_id, SMALLTALK_MODULE_ID, p);
            }
        }
        rt.build_session_index(&session_id)?;
        let module_ids = if module_ids_owned.is_empty() {
            None
        } else {
            Some(module_ids_owned.as_slice())
        };
        let results = rt.search_session(&session_id, &query, module_ids, top_k)?;
        rt.destroy_session(&session_id)?;
        Ok(results)
    })
    .await?;

    Ok(Json(SearchResponse {
        results: map_results(results),
    }))
}
