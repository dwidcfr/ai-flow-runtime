use axum::extract::{Path, State};
use axum::Json;

use crate::dto::session::{prompt_set_from_session, ActiveModuleDto};
use crate::dto::{FlowInspectResponse, SessionInspectResponse};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

#[utoipa::path(
    get,
    path = "/sessions/{session_id}/inspect",
    params(("session_id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Full session inspect", body = SessionInspectResponse),
        (status = 404, description = "Not found", body = crate::dto::ErrorResponse),
    ),
    tag = "sessions"
)]
pub async fn inspect_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionInspectResponse>, ApiError> {
    let response = with_runtime_read(&state, move |runtime| {
        let session = runtime.get_session(&session_id)?;
        let default_set = runtime
            .prompt_registry()
            .read()
            .map_err(|e| ai_flow_runtime::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })?
            .default_set_id()
            .to_string();
        Ok(SessionInspectResponse {
            session_id: session.session_id.clone(),
            flow_id: session.flow_id.clone(),
            current_node: session.current_node.clone(),
            paused_node: session.paused_node.clone(),
            runtime_state: session.runtime_state.into(),
            active_module: session.active_module.as_ref().map(|m| ActiveModuleDto {
                module_id: m.module_id.clone(),
                section: m.section.clone(),
            }),
            prompt_set: prompt_set_from_session(session, &default_set),
            metadata: session.metadata.clone(),
            client_data: session.client_data.clone(),
        })
    })
    .await?;

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/sessions/{session_id}/flow",
    params(("session_id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Flow inspect", body = FlowInspectResponse),
        (status = 404, description = "Not found", body = crate::dto::ErrorResponse),
    ),
    tag = "sessions"
)]
pub async fn inspect_flow(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<FlowInspectResponse>, ApiError> {
    let response = with_runtime_read(&state, move |runtime| {
        let snapshot = runtime.flow_snapshot(&session_id, None)?;
        Ok(FlowInspectResponse {
            flow_id: snapshot.flow_id,
            current_node: snapshot.current_node,
            paused_node: snapshot.paused_node,
            available_transitions: snapshot.available_transitions,
            node_payload: snapshot.node_payload,
            is_end: snapshot.is_end,
            runtime_state: snapshot.runtime_state.into(),
        })
    })
    .await?;

    Ok(Json(response))
}
