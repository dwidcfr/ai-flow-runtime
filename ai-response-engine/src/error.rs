use thiserror::Error;

pub type Result<T> = std::result::Result<T, ResponseError>;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("parse error: {message}")]
    ParseError { message: String },

    #[error("llm error: {message}")]
    LlmError { message: String },
}
