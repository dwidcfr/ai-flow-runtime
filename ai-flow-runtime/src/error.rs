use thiserror::Error;

use crate::state::RuntimeAction;
use crate::state::RuntimeState;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("flow not found: {0}")]
    FlowNotFound(String),

    #[error("invalid state transition: from {from:?} via {action:?}")]
    InvalidStateTransition {
        from: RuntimeState,
        action: RuntimeAction,
    },

    #[error("transition '{transition_id}' not found on node '{node}'")]
    TransitionNotFound {
        node: String,
        transition_id: String,
    },

    #[error("node '{node_id}' not found in flow '{flow_id}'")]
    NodeNotFound { flow_id: String, node_id: String },

    #[error("module is not open")]
    ModuleNotOpen,

    #[error("module is already open")]
    ModuleAlreadyOpen,

    #[error("pause_flow must be called before open_module")]
    PauseRequiredBeforeModule,

    #[error("no paused node to resume")]
    NoPausedNode,

    #[error("invalid module ref: {0}")]
    InvalidModuleRef(String),

    #[error("module not found: {0}")]
    ModuleNotFound(String),

    #[error("module section not found: {module_id}.{section}")]
    ModuleSectionNotFound { module_id: String, section: String },

    #[error("module load error in {module_id}: {message}")]
    ModuleLoadError { module_id: String, message: String },

    #[error("module operation not supported: {module_id}.{operation}")]
    ModuleOperationNotSupported { module_id: String, operation: String },

    #[error("module instance not loaded for session: {0}")]
    ModuleInstanceNotLoaded(String),

    #[error("flow provider '{0}' is not implemented")]
    FlowProviderNotImplemented(&'static str),

    #[error("flow engine error: {message}")]
    FlowEngineError { message: String },

    #[error("session is finished")]
    SessionFinished,

    #[error("search error: {0}")]
    SearchError(String),

    #[error("router error: {0}")]
    RouterError(String),

    #[error("response error: {0}")]
    ResponseError(String),

    #[error("configuration error: {message}")]
    Configuration { message: String },
}

pub type Result<T> = std::result::Result<T, RuntimeError>;
