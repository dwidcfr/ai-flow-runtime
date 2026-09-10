use axum::extract::{Path, State};
use axum::Json;

use crate::dto::{PromptSetDto, PromptSetsResponse, ReloadResponse};
use crate::errors::ApiError;
use crate::state::{with_runtime, AppState};

#[utoipa::path(
    get,
    path = "/prompt-sets",
    responses(
        (status = 200, description = "Available prompt sets", body = PromptSetsResponse),
    ),
    tag = "prompts"
)]
pub async fn list_prompt_sets(
    State(state): State<AppState>,
) -> Result<Json<PromptSetsResponse>, ApiError> {
    let runtime = state.runtime.clone();
    let response = tokio::task::spawn_blocking(move || {
        let rt = runtime.blocking_lock();
        let registry_arc = rt.prompt_registry();
        let registry = registry_arc.read().map_err(|e| {
            ApiError::internal(format!("prompt registry lock poisoned: {e}"))
        })?;
        let default_set = registry.default_set_id().to_string();
        let sets = registry
            .list_sets()
            .into_iter()
            .map(|meta| PromptSetDto {
                id: meta.id,
                name: meta.name,
                version: meta.version,
            })
            .collect();
        Ok::<_, ApiError>(PromptSetsResponse { default_set, sets })
    })
    .await
    .map_err(|e| ApiError::internal(format!("prompt list task failed: {e}")))??;

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/prompt-sets/{set_id}",
    params(("set_id" = String, Path, description = "Prompt set ID")),
    responses(
        (status = 200, description = "Prompt set detail", body = crate::dto::PromptSetDetailDto),
        (status = 404, description = "Not found", body = crate::dto::ErrorResponse),
    ),
    tag = "prompts"
)]
pub async fn get_prompt_set(
    State(state): State<AppState>,
    Path(set_id): Path<String>,
) -> Result<Json<crate::dto::PromptSetDetailDto>, ApiError> {
    let runtime = state.runtime.clone();
    let response = tokio::task::spawn_blocking(move || {
        let rt = runtime.blocking_lock();
        let registry_arc = rt.prompt_registry();
        let registry = registry_arc.read().map_err(|e| {
            ApiError::internal(format!("prompt registry lock poisoned: {e}"))
        })?;
        if !registry.has_set(&set_id) {
            return Err(ApiError::invalid_request(format!("prompt set not found: {set_id}")));
        }
        let identity = registry.identity(&set_id).map_err(|e| ApiError::internal(e.to_string()))?;
        let router = registry.router_template(&set_id).map_err(|e| ApiError::internal(e.to_string()))?;
        let response_tpl = registry
            .response_template(&set_id)
            .map_err(|e| ApiError::internal(e.to_string()))?;
        let meta = registry
            .list_sets()
            .into_iter()
            .find(|m| m.id == set_id)
            .ok_or_else(|| ApiError::invalid_request(format!("prompt set not found: {set_id}")))?;
        Ok::<_, ApiError>(crate::dto::PromptSetDetailDto {
            id: meta.id,
            name: meta.name,
            version: meta.version,
            identity: crate::dto::PromptIdentityDto {
                name: identity.name,
                bank_name: identity.bank_name,
                role: identity.role,
                language: identity.language,
                communication_style: identity.communication_style,
                tone: identity.tone,
            },
            router: crate::dto::PromptTemplateDto {
                system: router.system.clone(),
                developer: router.developer.clone(),
                assembly: router.assembly.clone(),
            },
            response: crate::dto::PromptTemplateDto {
                system: response_tpl.system.clone(),
                developer: response_tpl.developer.clone(),
                assembly: response_tpl.assembly.clone(),
            },
        })
    })
    .await
    .map_err(|e| ApiError::internal(format!("prompt detail task failed: {e}")))??;

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/prompt-sets/reload",
    responses(
        (status = 200, description = "Prompt registry reloaded", body = ReloadResponse),
    ),
    tag = "prompts"
)]
pub async fn reload_prompts(
    State(state): State<AppState>,
) -> Result<Json<ReloadResponse>, ApiError> {
    with_runtime(&state, |runtime| runtime.reload_prompts()).await?;

    let runtime = state.runtime.clone();
    let sets_count = tokio::task::spawn_blocking(move || {
        let rt = runtime.blocking_lock();
        rt.prompt_registry()
            .read()
            .map(|r| r.list_sets().len())
            .map_err(|e| ApiError::internal(format!("prompt registry lock poisoned: {e}")))
    })
    .await
    .map_err(|e| ApiError::internal(format!("prompt count task failed: {e}")))??;

    Ok(Json(ReloadResponse {
        status: "reloaded".into(),
        sets_count,
    }))
}
