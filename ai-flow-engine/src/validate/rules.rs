use crate::error::{FlowError, ValidationIssue, ValidationRule};
use crate::model::{Flow, NodeType};
use crate::parse::RawFlowDocument;

use super::reachability::{nodes_can_reach_end, nodes_reachable_from_initial};

pub fn validate_raw_document(doc: &RawFlowDocument) -> Result<(), FlowError> {
    let flow = crate::parse::build_flow(doc.clone());
    validate_flow(&flow)
}

pub fn validate_flow(flow: &Flow) -> Result<(), FlowError> {
    let mut issues = Vec::new();

    if flow.id.is_empty() {
        issues.push(ValidationIssue {
            rule: ValidationRule::EmptyFlowId,
            message: "flow id must not be empty".to_string(),
            node_id: None,
            action: None,
            target: None,
        });
    }

    if flow.initial.is_empty() {
        issues.push(ValidationIssue {
            rule: ValidationRule::EmptyInitial,
            message: "initial node must not be empty".to_string(),
            node_id: None,
            action: None,
            target: None,
        });
    }

    if flow.nodes.is_empty() {
        issues.push(ValidationIssue {
            rule: ValidationRule::EmptyNodes,
            message: "flow must contain at least one node".to_string(),
            node_id: None,
            action: None,
            target: None,
        });
    }

    if !flow.initial.is_empty() && !flow.contains_node(&flow.initial) {
        issues.push(ValidationIssue {
            rule: ValidationRule::InitialNodeNotFound,
            message: format!("initial node '{}' not found in nodes", flow.initial),
            node_id: Some(flow.initial.clone()),
            action: None,
            target: None,
        });
    }

    let has_end = flow
        .nodes
        .values()
        .any(|n| n.node_type == NodeType::End);
    if !has_end {
        issues.push(ValidationIssue {
            rule: ValidationRule::NoEndNode,
            message: "flow must contain at least one node with type 'end'".to_string(),
            node_id: None,
            action: None,
            target: None,
        });
    }

    for (node_id, node) in &flow.nodes {
        for (action, target) in &node.transitions {
            if !flow.contains_node(target) {
                issues.push(ValidationIssue {
                    rule: ValidationRule::UnknownTransitionTarget,
                    message: format!(
                        "transition '{action}' on node '{node_id}' points to unknown node '{target}'"
                    ),
                    node_id: Some(node_id.clone()),
                    action: Some(action.clone()),
                    target: Some(target.to_string()),
                });
            }
        }

        if node.node_type == NodeType::End && !node.transitions.is_empty() {
            issues.push(ValidationIssue {
                rule: ValidationRule::EndNodeHasTransitions,
                message: format!("end node '{node_id}' should not have transitions"),
                node_id: Some(node_id.clone()),
                action: None,
                target: None,
            });
        }
    }

    issues.extend(nodes_reachable_from_initial(flow));
    issues.extend(nodes_can_reach_end(flow));

    let blocking: Vec<_> = issues
        .iter()
        .filter(|i| i.rule != ValidationRule::EndNodeHasTransitions)
        .cloned()
        .collect();

    if !blocking.is_empty() {
        return Err(FlowError::validation(blocking));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::FlowLoader;

    fn load_yaml(yaml: &str) -> Result<Flow, FlowError> {
        FlowLoader::from_str(yaml)
    }

    #[test]
    fn valid_flow_passes() {
        let yaml = r#"
id: test
initial: a
nodes:
  a:
    type: flow
    transitions:
      go: end
  end:
    type: end
"#;
        assert!(load_yaml(yaml).is_ok());
    }

    #[test]
    fn missing_initial_node_fails() {
        let yaml = r#"
id: test
initial: missing
nodes:
  a:
    type: flow
    transitions:
      go: end
  end:
    type: end
"#;
        let err = load_yaml(yaml).unwrap_err();
        assert!(matches!(err, FlowError::Validation(_)));
    }

    #[test]
    fn no_end_node_fails() {
        let yaml = r#"
id: test
initial: a
nodes:
  a:
    type: flow
    transitions:
      loop: a
"#;
        let err = load_yaml(yaml).unwrap_err();
        assert!(matches!(err, FlowError::Validation(_)));
    }

    #[test]
    fn unknown_target_fails() {
        let yaml = r#"
id: test
initial: a
nodes:
  a:
    type: flow
    transitions:
      go: missing
  end:
    type: end
"#;
        let err = load_yaml(yaml).unwrap_err();
        assert!(matches!(err, FlowError::Validation(_)));
    }

    #[test]
    fn infinite_loop_without_exit_fails() {
        let yaml = r#"
id: test
initial: loop
nodes:
  loop:
    type: flow
    transitions:
      again: loop
  end:
    type: end
"#;
        let err = load_yaml(yaml).unwrap_err();
        assert!(matches!(err, FlowError::Validation(_)));
    }

    #[test]
    fn self_loop_with_exit_passes() {
        let yaml = r#"
id: test
initial: payment
nodes:
  payment:
    type: flow
    transitions:
      question: payment
      paid: end
  end:
    type: end
"#;
        assert!(load_yaml(yaml).is_ok());
    }
}
