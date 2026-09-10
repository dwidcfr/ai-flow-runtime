use serde::Serialize;
use utoipa::ToSchema;

use crate::errors::ApiErrorCode;

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub code: ApiErrorCode,
    pub message: String,
}
