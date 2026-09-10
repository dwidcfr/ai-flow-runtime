use thiserror::Error;

pub type Result<T> = std::result::Result<T, RouterError>;

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("parse error: {message}")]
    ParseError { message: String },

    #[error("llm error: {message}")]
    LlmError { message: String },
}
