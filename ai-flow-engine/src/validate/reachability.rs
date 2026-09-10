use std::collections::{HashMap, HashSet, VecDeque};

use crate::error::ValidationIssue;
use crate::error::ValidationRule;
use crate::model::{Flow, NodeType};

pub fn nodes_can_reach_end(flow: &Flow) -> Vec<ValidationIssue> {
    let end_nodes: HashSet<&str> = flow
        .nodes
        .iter()
        .filter(|(_, n)| n.node_type == NodeType::End)
        .map(|(id, _)| id.as_str())
        .collect();

    if end_nodes.is_empty() {
        return Vec::new();
    }

    let mut reverse: HashMap<&str, Vec<&str>> = HashMap::new();
    for (node_id, node) in &flow.nodes {
        for target in node.transitions.values() {
            reverse
                .entry(target.as_str())
                .or_default()
                .push(node_id.as_str());
        }
    }

    let mut can_reach_end: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = end_nodes.iter().copied().collect();

    while let Some(current) = queue.pop_front() {
        if !can_reach_end.insert(current) {
            continue;
        }
        if let Some(predecessors) = reverse.get(current) {
            for &pred in predecessors {
                if !can_reach_end.contains(pred) {
                    queue.push_back(pred);
                }
            }
        }
    }

    let mut issues = Vec::new();
    for (node_id, node) in &flow.nodes {
        if node.node_type != NodeType::End && !can_reach_end.contains(node_id.as_str()) {
            issues.push(ValidationIssue {
                rule: ValidationRule::CannotReachEnd,
                message: format!(
                    "node '{node_id}' cannot reach any end node (potential infinite loop)"
                ),
                node_id: Some(node_id.clone()),
                action: None,
                target: None,
            });
        }
    }

    issues
}

pub fn nodes_reachable_from_initial(flow: &Flow) -> Vec<ValidationIssue> {
    let mut visited: HashSet<&str> = HashSet::new();
    let mut queue: VecDeque<&str> = VecDeque::new();
    queue.push_back(flow.initial.as_str());

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current) {
            continue;
        }
        if let Some(node) = flow.node(current) {
            for target in node.transitions.values() {
                queue.push_back(target.as_str());
            }
        }
    }

    let mut issues = Vec::new();
    for node_id in flow.node_ids() {
        if !visited.contains(node_id) {
            issues.push(ValidationIssue {
                rule: ValidationRule::UnreachableFromInitial,
                message: format!("node '{node_id}' is not reachable from initial node"),
                node_id: Some(node_id.to_string()),
                action: None,
                target: None,
            });
        }
    }

    issues
}
