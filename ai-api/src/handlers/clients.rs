use std::fs;

use axum::extract::{Path, State};
use axum::Json;

use crate::dto::{ClientDto, ClientsResponse};
use crate::errors::ApiError;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/clients",
    responses((status = 200, description = "Available client files", body = ClientsResponse)),
    tag = "clients"
)]
pub async fn list_clients(State(state): State<AppState>) -> Result<Json<ClientsResponse>, ApiError> {
    let clients_dir = state.config.data_root.join("clients");
    if !clients_dir.exists() {
        return Ok(Json(ClientsResponse { clients: vec![] }));
    }

    let mut clients = Vec::new();
    for entry in fs::read_dir(&clients_dir).map_err(|e| ApiError::internal(e.to_string()))? {
        let entry = entry.map_err(|e| ApiError::internal(e.to_string()))?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            clients.push(ClientDto {
                path: format!("clients/{file_name}"),
                name: file_name.strip_suffix(".json").unwrap_or(&file_name).to_string(),
            });
        }
    }
    clients.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Json(ClientsResponse { clients }))
}

#[utoipa::path(
    get,
    path = "/clients/{name}",
    params(("name" = String, Path, description = "Client template name")),
    responses((status = 200, description = "Client JSON content")),
    tag = "clients"
)]
pub async fn get_client(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let path = state.config.data_root.join("clients").join(format!("{name}.json"));
    if !path.exists() {
        return Err(ApiError::invalid_request(format!("client not found: {name}")));
    }
    let content = fs::read_to_string(&path).map_err(|e| ApiError::internal(e.to_string()))?;
    let data: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(data))
}
