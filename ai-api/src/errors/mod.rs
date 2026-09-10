use std::fmt;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use thiserror::Error;
use utoipa::ToSchema;

use ai_flow_runtime::RuntimeError;

use crate::dto::error::ErrorResponse;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiErrorCode {
    SessionNotFound,
    FlowNotFound,
    SessionFinished,
    InvalidState,
    InvalidRequest,
    InternalError,
}

#[derive(Debug, Clone)]
pub struct ApiError {
    pub code: ApiErrorCode,
    pub message: String,
    pub status: StatusCode,
}

impl ApiError {
    pub fn new(code: ApiErrorCode, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code,
            message: message.into(),
            status,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            ApiErrorCode::InternalError,
            message,
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }

    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::new(
            ApiErrorCode::InvalidRequest,
            message,
            StatusCode::BAD_REQUEST,
        )
    }

    pub fn flow_not_found(flow_id: &str) -> Self {
        Self::new(
            ApiErrorCode::FlowNotFound,
            format!("Flow not found: {flow_id}"),
            StatusCode::NOT_FOUND,
        )
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Error)]
pub enum ApiStartupError {
    #[error("config error: {0}")]
    Config(#[from] crate::config::ConfigError),
    #[error("startup error: {0}")]
    Startup(String),
}

impl From<RuntimeError> for ApiError {
    fn from(err: RuntimeError) -> Self {
        match err {
            RuntimeError::SessionNotFound(id) => Self::new(
                ApiErrorCode::SessionNotFound,
                format!("Session does not exist: {id}"),
                StatusCode::NOT_FOUND,
            ),
            RuntimeError::FlowNotFound(id) => Self::new(
                ApiErrorCode::FlowNotFound,
                format!("Flow not found: {id}"),
                StatusCode::NOT_FOUND,
            ),
            RuntimeError::SessionFinished => Self::new(
                ApiErrorCode::SessionFinished,
                "Session is finished".to_string(),
                StatusCode::CONFLICT,
            ),
            RuntimeError::InvalidStateTransition { .. }
            | RuntimeError::ModuleNotOpen
            | RuntimeError::ModuleAlreadyOpen
            | RuntimeError::PauseRequiredBeforeModule
            | RuntimeError::NoPausedNode => Self::new(
                ApiErrorCode::InvalidState,
                err.to_string(),
                StatusCode::CONFLICT,
            ),
            RuntimeError::TransitionNotFound { .. }
            | RuntimeError::NodeNotFound { .. }
            | RuntimeError::InvalidModuleRef(_)
            | RuntimeError::ModuleNotFound(_)
            | RuntimeError::ModuleSectionNotFound { .. }
            | RuntimeError::ModuleInstanceNotLoaded(_) => Self::new(
                ApiErrorCode::InvalidRequest,
                err.to_string(),
                StatusCode::BAD_REQUEST,
            ),
            RuntimeError::ModuleLoadError { .. }
            | RuntimeError::ModuleOperationNotSupported { .. }
            | RuntimeError::FlowProviderNotImplemented(_)
            | RuntimeError::FlowEngineError { .. }
            | RuntimeError::SearchError(_)
            | RuntimeError::RouterError(_)
            | RuntimeError::ResponseError(_)
            | RuntimeError::Configuration { .. } => Self::internal(err.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            code: self.code,
            message: self.message,
        };
        (self.status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_session_not_found() {
        let err = ApiError::from(RuntimeError::SessionNotFound("x".into()));
        assert_eq!(err.code, ApiErrorCode::SessionNotFound);
        assert_eq!(err.status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn maps_session_finished() {
        let err = ApiError::from(RuntimeError::SessionFinished);
        assert_eq!(err.code, ApiErrorCode::SessionFinished);
        assert_eq!(err.status, StatusCode::CONFLICT);
    }
}
