use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClientDto {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ClientsResponse {
    pub clients: Vec<ClientDto>,
}
