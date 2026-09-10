use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlowError {
    #[error("io error at {path}: {message}")]
    Io { path: PathBuf, message: String },

    #[error("yaml parse error: {message}")]
    YamlParse { message: String },

    #[error("flow validation failed")]
    Validation(Vec<ValidationIssue>),

    #[error("node '{node_id}' not found in flow '{flow_id}'")]
    NodeNotFound { flow_id: String, node_id: String },

    #[error("transition '{action}' not found on node '{node_id}'")]
    TransitionNotFound { node_id: String, action: String },
}

pub type Result<T> = std::result::Result<T, FlowError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub rule: ValidationRule,
    pub message: String,
    pub node_id: Option<String>,
    pub action: Option<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValidationRule {
    EmptyFlowId,
    EmptyInitial,
    InitialNodeNotFound,
    DuplicateNodeId,
    NoEndNode,
    UnknownTransitionTarget,
    UnreachableFromInitial,
    CannotReachEnd,
    EndNodeHasTransitions,
    EmptyNodes,
}

impl FlowError {
    pub fn validation(issues: Vec<ValidationIssue>) -> Self {
        Self::Validation(issues)
    }

    pub fn validation_issue_count(&self) -> Option<usize> {
        match self {
            Self::Validation(issues) => Some(issues.len()),
            _ => None,
        }
    }
}
