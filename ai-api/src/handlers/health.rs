use axum::extract::State;
use axum::Json;

use crate::dto::HealthResponse;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service health", body = HealthResponse)
    ),
    tag = "health"
)]
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: state.config.version.clone(),
        uptime_secs: state.uptime_secs(),
    })
}
