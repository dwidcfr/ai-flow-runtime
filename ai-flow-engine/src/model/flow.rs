use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::Node;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlowMeta {
    pub name: Option<String>,
    pub version: Option<u32>,
}

impl Default for FlowMeta {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Flow {
    pub id: String,
    pub meta: FlowMeta,
    pub initial: String,
    pub nodes: HashMap<String, Node>,
}

impl Flow {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn meta(&self) -> &FlowMeta {
        &self.meta
    }

    pub fn initial_node(&self) -> &str {
        &self.initial
    }

    pub fn contains_node(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }

    pub fn node(&self, node_id: &str) -> Option<&Node> {
        self.nodes.get(node_id)
    }

    pub fn node_ids(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(String::as_str)
    }
}
