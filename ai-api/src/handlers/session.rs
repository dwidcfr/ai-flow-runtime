use axum::extract::{Path, State};
use axum::Json;

use crate::dto::{
    CreateSessionRequest, CreateSessionResponse, SessionResponse,
};
use crate::dto::session::prompt_set_from_session;
use crate::errors::ApiError;
use crate::state::{with_runtime, with_runtime_read, AppState};

#[utoipa::path(
    post,
    path = "/sessions",
    request_body = CreateSessionRequest,
    responses(
        (status = 200, description = "Session created", body = CreateSessionResponse),
        (status = 404, description = "Flow not found", body = crate::dto::ErrorResponse),
        (status = 400, description = "Invalid request", body = crate::dto::ErrorResponse),
    ),
    tag = "sessions"
)]
pub async fn create_session(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionRequest>,
) -> Result<Json<CreateSessionResponse>, ApiError> {
    if !state.flow_registry.contains(&body.flow_id) {
        return Err(ApiError::flow_not_found(&body.flow_id));
    }

    let client_path = state
        .config
        .resolve_data_path(&body.client_source_path);
    if !client_path.exists() {
        return Err(ApiError::invalid_request(format!(
            "client source not found: {}",
            client_path.display()
        )));
    }

    let client_source = client_path
        .to_str()
        .ok_or_else(|| ApiError::invalid_request("invalid client source path"))?
        .to_string();

    let flow_id = body.flow_id.clone();
    let prompt_set = body.prompt_set.clone();
    let default_modules = state.config.default_modules.clone();
    let data_root = state.config.data_root.clone();

    let session_id = with_runtime(&state, move |runtime| {
        let session_id = if let Some(set) = prompt_set.as_deref() {
            runtime.create_session_with_prompt_set(&flow_id, &client_source, set)?
        } else {
            runtime.create_session(&flow_id, &client_source)?
        };

        for module in &default_modules {
            let module_path = if std::path::Path::new(&module.path).is_absolute() {
                std::path::PathBuf::from(&module.path)
            } else {
                data_root.join(&module.path)
            };
            let module_source = module_path
                .to_str()
                .ok_or_else(|| {
                    ai_flow_runtime::RuntimeError::Configuration {
                        message: format!("invalid module path: {}", module_path.display()),
                    }
                })?
                .to_string();
            runtime.load_session_module(&session_id, &module.module_id, &module_source)?;
        }

        let opening = runtime.generate_opening_message(&session_id)?;

        Ok((session_id, opening.text))
    })
    .await?;

    let (session_id, opening_message) = session_id;

    let session_id_for_read = session_id.clone();
    let session = with_runtime_read(&state, move |runtime| {
        let session = runtime.get_session(&session_id_for_read)?;
        let default_set = runtime
            .prompt_registry()
            .read()
            .map_err(|e| ai_flow_runtime::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .default_set_id()
            .to_string();

        Ok((
            session.current_node.clone(),
            session.metadata.clone(),
            prompt_set_from_session(session, &default_set),
        ))
    })
    .await?;

    Ok(Json(CreateSessionResponse {
        session_id,
        current_node: session.0,
        prompt_set: session.2,
        metadata: session.1,
        opening_message: Some(opening_message),
    }))
}

#[utoipa::path(
    get,
    path = "/sessions/{session_id}",
    params(("session_id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Session info", body = SessionResponse),
        (status = 404, description = "Session not found", body = crate::dto::ErrorResponse),
    ),
    tag = "sessions"
)]
pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponse>, ApiError> {
    let response = with_runtime_read(&state, move |runtime| {
        let session = runtime.get_session(&session_id)?;
        Ok(SessionResponse::from_session(session))
    })
    .await?;

    Ok(Json(response))
}

#[utoipa::path(
    delete,
    path = "/sessions/{session_id}",
    params(("session_id" = String, Path, description = "Session ID")),
    responses(
        (status = 204, description = "Session deleted"),
        (status = 404, description = "Session not found", body = crate::dto::ErrorResponse),
    ),
    tag = "sessions"
)]
pub async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<axum::http::StatusCode, ApiError> {
    with_runtime(&state, move |runtime| runtime.destroy_session(&session_id)).await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
