use thiserror::Error;

pub type Result<T> = std::result::Result<T, EvalError>;

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("load error in {path}: {message}")]
    Load { path: String, message: String },

    #[error("fixture not found: {kind}.{name}")]
    FixtureNotFound { kind: String, name: String },

    #[error("scenario not found: {0}")]
    ScenarioNotFound(String),

    #[error("duplicate scenario name: {0}")]
    DuplicateScenario(String),

    #[error("runtime error: {0}")]
    Runtime(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
