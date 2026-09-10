use axum::extract::State;
use axum::Json;

use crate::dto::{FlowDto, FlowsResponse};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/flows",
    responses((status = 200, description = "Available flows", body = FlowsResponse)),
    tag = "flows"
)]
pub async fn list_flows(State(state): State<AppState>) -> Json<FlowsResponse> {
    let flows = state
        .flow_registry
        .list()
        .iter()
        .map(|f| FlowDto {
            id: f.id.clone(),
            name: f.name.clone(),
            version: f.version.clone(),
        })
        .collect();
    Json(FlowsResponse { flows })
}
