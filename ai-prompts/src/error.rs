use thiserror::Error;

pub type Result<T> = std::result::Result<T, PromptError>;

#[derive(Debug, Error)]
pub enum PromptError {
    #[error("missing file: {0}")]
    MissingFile(String),

    #[error("invalid yaml in {path}: {message}")]
    InvalidYaml { path: String, message: String },

    #[error("missing field: {field} in {path}")]
    MissingField { field: String, path: String },

    #[error("prompt set not found: {0}")]
    SetNotFound(String),

    #[error("version mismatch for set {set_id}: expected {expected}, found {found}")]
    VersionMismatch {
        set_id: String,
        expected: String,
        found: String,
    },

    #[error("meta id mismatch: directory {dir} != meta.id {meta_id}")]
    MetaIdMismatch { dir: String, meta_id: String },

    #[error("default set '{0}' not found in registry")]
    DefaultSetNotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
