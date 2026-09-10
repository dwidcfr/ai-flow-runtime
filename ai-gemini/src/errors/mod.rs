use thiserror::Error;

pub type Result<T> = std::result::Result<T, GeminiError>;

#[derive(Debug, Error)]
pub enum GeminiError {
    #[error("network error: {0}")]
    Network(String),

    #[error("request timed out")]
    Timeout,

    #[error("unauthorized: invalid API key")]
    Unauthorized,

    #[error("rate limited")]
    RateLimited,

    #[error("api error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("invalid JSON response: {0}")]
    InvalidJson(String),

    #[error("empty response from model")]
    EmptyResponse,

    #[error("configuration error: {0}")]
    Configuration(String),
}

impl GeminiError {
    pub fn from_status(status: u16, message: String) -> Self {
        match status {
            401 | 403 => GeminiError::Unauthorized,
            429 => GeminiError::RateLimited,
            s if s >= 500 => GeminiError::Api { status: s, message },
            s => GeminiError::Api { status: s, message },
        }
    }

    pub fn is_retryable(&self) -> bool {
        match self {
            GeminiError::Network(_) | GeminiError::Timeout | GeminiError::RateLimited => true,
            GeminiError::Api { status, .. } => *status >= 500,
            _ => false,
        }
    }
}
