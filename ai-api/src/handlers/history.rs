use axum::extract::{Path, State};
use axum::Json;

use crate::dto::{HistoryEntryDto, HistoryResponse};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

#[utoipa::path(
    get,
    path = "/sessions/{session_id}/history",
    params(("session_id" = String, Path, description = "Session ID")),
    responses(
        (status = 200, description = "Conversation history", body = HistoryResponse),
        (status = 404, description = "Session not found", body = crate::dto::ErrorResponse),
    ),
    tag = "history"
)]
pub async fn get_history(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<HistoryResponse>, ApiError> {
    let entries = with_runtime_read(&state, move |runtime| {
        let session = runtime.get_session(&session_id)?;
        Ok(session
            .history
            .iter()
            .map(HistoryEntryDto::from)
            .collect::<Vec<_>>())
    })
    .await?;

    Ok(Json(HistoryResponse { entries }))
}
