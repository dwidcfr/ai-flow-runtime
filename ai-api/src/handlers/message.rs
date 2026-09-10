use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;

use ai_flow_runtime::HandleMessageOptions;

use crate::dto::message_debug_from_trace;
use crate::dto::session::ActiveModuleDto;
use crate::dto::{MessageQuery, MessageRequest, MessageResponse};
use crate::errors::ApiError;
use crate::middleware::request_id::REQUEST_ID_HEADER;
use crate::state::{with_runtime, AppState};

fn debug_enabled(query: &MessageQuery, headers: &HeaderMap) -> bool {
    query.debug
        || headers
            .get("x-debug-mode")
            .and_then(|v| v.to_str().ok())
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false)
}

#[utoipa::path(
    post,
    path = "/sessions/{session_id}/messages",
    params(
        ("session_id" = String, Path, description = "Session ID"),
        ("debug" = Option<bool>, Query, description = "Include debug trace"),
    ),
    request_body = MessageRequest,
    responses(
        (status = 200, description = "Assistant response", body = MessageResponse),
        (status = 404, description = "Session not found", body = crate::dto::ErrorResponse),
        (status = 409, description = "Session finished", body = crate::dto::ErrorResponse),
    ),
    tag = "messages"
)]
pub async fn send_message(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(query): Query<MessageQuery>,
    headers: HeaderMap,
    Json(body): Json<MessageRequest>,
) -> Result<Json<MessageResponse>, ApiError> {
    if body.message.trim().is_empty() {
        return Err(ApiError::invalid_request("message must not be empty"));
    }

    let collect_debug = debug_enabled(&query, &headers);
    let include_decision = collect_debug || state.config.include_router_decision;

    if !state.config.log_user_messages {
        tracing::debug!(session_id = %session_id, "received user message");
    } else {
        tracing::debug!(session_id = %session_id, message = %body.message, "received user message");
    }

    let _request_id = headers.get(REQUEST_ID_HEADER);
    let message = body.message;

    let result = with_runtime(&state, move |runtime| {
        let outcome = runtime.handle_user_message_with_options(
            &session_id,
            &message,
            HandleMessageOptions {
                collect_trace: collect_debug,
            },
        )?;
        let session = runtime.get_session(&session_id)?;
        Ok((
            outcome.response.text,
            outcome.response.finish_conversation,
            session.current_node.clone(),
            session.active_module.as_ref().map(|m| ActiveModuleDto {
                module_id: m.module_id.clone(),
                section: m.section.clone(),
            }),
            if include_decision {
                serde_json::to_value(&outcome.decision).ok()
            } else {
                None
            },
            if collect_debug {
                outcome.trace.map(|t| message_debug_from_trace(&t))
            } else {
                None
            },
        ))
    })
    .await?;

    Ok(Json(MessageResponse {
        assistant_message: result.0,
        finish_conversation: result.1,
        current_node: result.2,
        active_module: result.3,
        router_decision: result.4,
        debug: result.5,
    }))
}
