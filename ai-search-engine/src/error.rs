use thiserror::Error;

pub type Result<T> = std::result::Result<T, SearchError>;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("provider not found: {module_id}")]
    ProviderNotFound { module_id: String },

    #[error("document not found: {document_id}")]
    DocumentNotFound { document_id: String },

    #[error("embedding error: {message}")]
    EmbeddingError { message: String },

    #[error("index error: {message}")]
    IndexError { message: String },

    #[error("provider error: {module_id}: {message}")]
    ProviderError { module_id: String, message: String },
}
