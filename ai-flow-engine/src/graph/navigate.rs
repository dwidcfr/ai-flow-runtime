use crate::error::{FlowError, Result};
use crate::model::{Flow, NodeType, TransitionResult};

pub fn available_transitions<'a>(flow: &'a Flow, node_id: &str) -> Result<Vec<&'a str>> {
    let node = flow.node(node_id).ok_or_else(|| FlowError::NodeNotFound {
        flow_id: flow.id.clone(),
        node_id: node_id.to_string(),
    })?;

    let mut actions: Vec<&str> = node.transitions.keys().map(String::as_str).collect();
    actions.sort_unstable();
    Ok(actions)
}

pub fn transition(flow: &Flow, node_id: &str, action: &str) -> Result<TransitionResult> {
    let node = flow.node(node_id).ok_or_else(|| FlowError::NodeNotFound {
        flow_id: flow.id.clone(),
        node_id: node_id.to_string(),
    })?;

    let target = node.transitions.get(action).ok_or_else(|| FlowError::TransitionNotFound {
        node_id: node_id.to_string(),
        action: action.to_string(),
    })?;

    let target_node = flow.node(target).ok_or_else(|| FlowError::NodeNotFound {
        flow_id: flow.id.clone(),
        node_id: target.clone(),
    })?;

    Ok(TransitionResult {
        action: action.to_string(),
        from: node_id.to_string(),
        to: target.clone(),
        target_is_end: target_node.node_type == NodeType::End,
    })
}

pub fn is_end(flow: &Flow, node_id: &str) -> bool {
    flow.node(node_id)
        .map(|n| n.node_type == NodeType::End)
        .unwrap_or(false)
}

impl Flow {
    pub fn available_transitions(&self, node_id: &str) -> Result<Vec<&str>> {
        available_transitions(self, node_id)
    }

    pub fn transition(&self, node_id: &str, action: &str) -> Result<TransitionResult> {
        transition(self, node_id, action)
    }

    pub fn is_end(&self, node_id: &str) -> bool {
        is_end(self, node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::FlowLoader;

    const FLOW_YAML: &str = r#"
id: payment_flow
initial: greeting
nodes:
  greeting:
    type: flow
    transitions:
      confirm: payment
      wrong_person: wrong_person
  payment:
    type: flow
    transitions:
      paid: end
      question: payment
  wrong_person:
    type: flow
    transitions:
      finish: end
  end:
    type: end
"#;

    fn sample_flow() -> Flow {
        FlowLoader::from_str(FLOW_YAML).unwrap()
    }

    #[test]
    fn available_transitions_returns_actions() {
        let flow = sample_flow();
        let actions = flow.available_transitions("greeting").unwrap();
        assert!(actions.contains(&"confirm"));
        assert!(actions.contains(&"wrong_person"));
    }

    #[test]
    fn transition_success() {
        let flow = sample_flow();
        let result = flow.transition("greeting", "confirm").unwrap();
        assert_eq!(result.to, "payment");
        assert!(!result.target_is_end);
    }

    #[test]
    fn transition_to_end() {
        let flow = sample_flow();
        let result = flow.transition("payment", "paid").unwrap();
        assert_eq!(result.to, "end");
        assert!(result.target_is_end);
    }

    #[test]
    fn transition_not_found() {
        let flow = sample_flow();
        let err = flow.transition("greeting", "missing").unwrap_err();
        assert!(matches!(err, FlowError::TransitionNotFound { .. }));
    }

    #[test]
    fn is_end_true_for_end_node() {
        let flow = sample_flow();
        assert!(flow.is_end("end"));
        assert!(!flow.is_end("greeting"));
    }

    #[test]
    fn node_not_found() {
        let flow = sample_flow();
        let err = flow.available_transitions("missing").unwrap_err();
        assert!(matches!(err, FlowError::NodeNotFound { .. }));
    }
}
