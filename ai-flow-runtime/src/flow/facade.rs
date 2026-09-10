use ai_flow_engine::Flow;
use serde_json::Value;

use crate::error::{Result, RuntimeError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransitionOutcome {
    pub action: String,
    pub from: String,
    pub to: String,
    pub target_is_end: bool,
}

#[derive(Debug, Clone)]
pub struct FlowFacade {
    inner: Flow,
}

impl FlowFacade {
    pub fn from_engine(flow: Flow) -> Self {
        Self { inner: flow }
    }

    pub fn id(&self) -> &str {
        self.inner.id()
    }

    pub fn initial_node(&self) -> &str {
        self.inner.initial_node()
    }

    pub fn contains_node(&self, node_id: &str) -> bool {
        self.inner.contains_node(node_id)
    }

    pub fn available_transitions(&self, node_id: &str) -> Result<Vec<String>> {
        self.inner
            .available_transitions(node_id)
            .map(|actions| actions.into_iter().map(|a| a.to_string()).collect())
            .map_err(map_flow_error)
    }

    pub fn transition(&self, node_id: &str, action: &str) -> Result<TransitionOutcome> {
        let result = self
            .inner
            .transition(node_id, action)
            .map_err(map_flow_error)?;

        Ok(TransitionOutcome {
            action: result.action,
            from: result.from,
            to: result.to,
            target_is_end: result.target_is_end,
        })
    }

    pub fn is_end(&self, node_id: &str) -> bool {
        self.inner.is_end(node_id)
    }

    pub fn node_payload(&self, node_id: &str) -> Result<Value> {
        let node = self
            .inner
            .node(node_id)
            .ok_or_else(|| RuntimeError::NodeNotFound {
                flow_id: self.inner.id().to_string(),
                node_id: node_id.to_string(),
            })?;
        Ok(node.payload.clone())
    }

    pub(crate) fn inner_flow(&self) -> &Flow {
        &self.inner
    }
}

fn map_flow_error(err: ai_flow_engine::FlowError) -> RuntimeError {
    match err {
        ai_flow_engine::FlowError::NodeNotFound { flow_id, node_id } => {
            RuntimeError::NodeNotFound { flow_id, node_id }
        }
        ai_flow_engine::FlowError::TransitionNotFound { node_id, action } => {
            RuntimeError::TransitionNotFound {
                node: node_id,
                transition_id: action,
            }
        }
        other => RuntimeError::FlowEngineError {
            message: other.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use ai_flow_engine::FlowLoader;

    use super::*;

    const FLOW_YAML: &str = r#"
id: test_flow
initial: greeting
nodes:
  greeting:
    type: flow
    transitions:
      confirm: payment
  payment:
    type: flow
    transitions:
      paid: end
  end:
    type: end
"#;

    fn sample_facade() -> FlowFacade {
        let flow = FlowLoader::from_str(FLOW_YAML).unwrap();
        FlowFacade::from_engine(flow)
    }

    #[test]
    fn facade_delegates_initial_node() {
        let facade = sample_facade();
        assert_eq!(facade.initial_node(), "greeting");
    }

    #[test]
    fn facade_delegates_transition() {
        let facade = sample_facade();
        let outcome = facade.transition("greeting", "confirm").unwrap();
        assert_eq!(outcome.to, "payment");
    }

    #[test]
    fn facade_maps_transition_not_found() {
        let facade = sample_facade();
        let err = facade.transition("greeting", "missing").unwrap_err();
        assert!(matches!(err, RuntimeError::TransitionNotFound { .. }));
    }
}
