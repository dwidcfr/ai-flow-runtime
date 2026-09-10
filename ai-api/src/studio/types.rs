use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftManifest {
    pub project_id: String,
    pub dirty: bool,
    pub edited_at: Option<String>,
    pub published_version: u32,
    pub bootstrapped_at: Option<String>,
}

impl DraftManifest {
    pub fn new(project_id: String) -> Self {
        Self {
            project_id,
            dirty: false,
            edited_at: None,
            published_version: 0,
            bootstrapped_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftCompany {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub prompt_set: String,
    pub knowledge_source: String,
    pub flows: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftFlowTransition {
    pub action: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftFlowNode {
    pub id: String,
    pub node_type: String,
    pub payload: Value,
    pub transitions: Vec<DraftFlowTransition>,
    #[serde(default)]
    pub position: Option<NodePosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftFlow {
    pub id: String,
    pub name: Option<String>,
    pub version: u32,
    pub initial: String,
    pub nodes: Vec<DraftFlowNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeCategory {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub category_id: String,
    pub title: String,
    pub section: String,
    pub content: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftKnowledge {
    pub categories: Vec<KnowledgeCategory>,
    pub documents: Vec<KnowledgeDocument>,
}

impl Default for DraftKnowledge {
    fn default() -> Self {
        Self {
            categories: vec![KnowledgeCategory {
                id: "general".into(),
                name: "General".into(),
            }],
            documents: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPromptIdentity {
    #[serde(flatten)]
    pub fields: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPromptTemplate {
    pub system: String,
    pub developer: String,
    pub assembly: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftPromptSet {
    pub id: String,
    pub name: String,
    pub version: String,
    pub identity: Value,
    pub router: DraftPromptTemplate,
    pub response: DraftPromptTemplate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub version: u32,
    pub published_at: String,
    pub project_id: String,
    pub files: Vec<String>,
}
