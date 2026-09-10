use ai_flow_engine::NodeType;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::flow::FlowFacade;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStats {
    pub node_count: usize,
    pub end_node_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNodeExport {
    pub id: String,
    pub node_type: String,
    pub payload: Value,
    pub transitions: Vec<FlowEdgeExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdgeExport {
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowExport {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<u32>,
    pub initial: String,
    pub stats: FlowStats,
    pub nodes: Vec<FlowNodeExport>,
}

impl FlowExport {
    pub fn from_facade(facade: &FlowFacade) -> Self {
        let flow = facade.inner_flow();
        let mut end_node_count = 0usize;
        let mut nodes = Vec::new();

        for node_id in flow.node_ids() {
            let node = flow.node(node_id).expect("node exists");
            if node.node_type == NodeType::End {
                end_node_count += 1;
            }
            let transitions: Vec<FlowEdgeExport> = node
                .transitions
                .iter()
                .map(|(action, target)| FlowEdgeExport {
                    action: action.clone(),
                    target: target.clone(),
                })
                .collect();
            nodes.push(FlowNodeExport {
                id: node.id.clone(),
                node_type: match node.node_type {
                    NodeType::Flow => "flow".to_string(),
                    NodeType::End => "end".to_string(),
                },
                payload: node.payload.clone(),
                transitions,
            });
        }

        nodes.sort_by(|a, b| a.id.cmp(&b.id));

        FlowExport {
            id: flow.id().to_string(),
            name: flow.meta().name.clone(),
            version: flow.meta().version,
            initial: flow.initial_node().to_string(),
            stats: FlowStats {
                node_count: nodes.len(),
                end_node_count,
            },
            nodes,
        }
    }
}
