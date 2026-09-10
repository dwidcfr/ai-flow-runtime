use axum::Json;
use axum::extract::State;

use crate::dto::mappers::{map_active_module, runtime_state_to_dto};
use crate::dto::{SessionListItemDto, SessionsListResponse};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

pub async fn list_sessions(
    State(state): State<AppState>,
) -> Result<Json<SessionsListResponse>, ApiError> {
    let summaries = with_runtime_read(&state, |rt| rt.list_sessions()).await?;
    let sessions = summaries
        .into_iter()
        .map(|s| {
            let active_module = s.active_module.map(|module_id| map_active_module(&module_id, None));
            SessionListItemDto {
                session_id: s.session_id,
                flow_id: s.flow_id,
                prompt_set: s.prompt_set,
                current_node: s.current_node,
                runtime_state: runtime_state_to_dto(s.runtime_state),
                active_module,
                created_at: s.created_at.map(|t| t.to_rfc3339()),
                last_activity: s.last_activity.map(|t| t.to_rfc3339()),
            }
        })
        .collect();
    Ok(Json(SessionsListResponse { sessions }))
}
