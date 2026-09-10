use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{EvalError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmMode {
    #[default]
    Mock,
    Gemini,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationTurn {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum PostStep {
    CloseModule,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum SetupStep {
    Transition { transition: String },
    GotoNode { goto_node: String },
    ConversationEntry { conversation_entry: ConversationEntry },
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedRouter {
    #[serde(rename = "type")]
    pub decision_type: String,
    pub action: Option<String>,
    pub module: Option<String>,
    pub section: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedResponse {
    #[serde(default)]
    pub contains: Vec<String>,
    #[serde(default)]
    pub not_contains: Vec<String>,
    pub finish_conversation: Option<bool>,
    pub non_empty: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedFlow {
    pub current_node: Option<String>,
    pub paused_node: Option<String>,
    pub runtime_state: Option<String>,
    pub finished: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedSearchTarget {
    pub module: String,
    pub section: Option<String>,
    pub document_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedSearch {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    pub expect: ExpectedSearchTarget,
}

fn default_top_k() -> usize {
    3
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ExpectedSession {
    pub user_message_recorded: Option<bool>,
    pub assistant_message_recorded: Option<bool>,
    pub history_min_len: Option<usize>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Expected {
    pub router: Option<ExpectedRouter>,
    pub response: Option<ExpectedResponse>,
    pub flow: Option<ExpectedFlow>,
    pub search: Option<ExpectedSearch>,
    pub session: Option<ExpectedSession>,
    pub finish_conversation: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvalScenario {
    pub name: String,
    pub domain: Option<String>,
    #[serde(default)]
    pub llm: LlmMode,
    pub flow: String,
    pub client: String,
    #[serde(default)]
    pub modules: HashMap<String, String>,
    #[serde(default)]
    pub setup: Vec<SetupStep>,
    #[serde(default)]
    pub conversation: Vec<ConversationTurn>,
    pub user_message: String,
    #[serde(default)]
    pub steps: Vec<PostStep>,
    pub expected: Expected,
    #[serde(skip)]
    pub source_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FixturesManifest {
    pub flows: HashMap<String, String>,
    pub clients: HashMap<String, String>,
    pub modules: HashMap<String, String>,
}

impl FixturesManifest {
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let base = path.parent().unwrap_or(Path::new("."));
        let manifest: FixturesManifest = serde_yaml::from_str(&content)?;
        Ok(manifest.resolve_paths(base))
    }

    fn resolve_paths(mut self, base: &Path) -> Self {
        self.flows = resolve_map(self.flows, base);
        self.clients = resolve_map(self.clients, base);
        self.modules = resolve_map(self.modules, base);
        self
    }

    pub fn flow_path(&self, name: &str) -> Result<PathBuf> {
        self.flows
            .get(name)
            .map(|p| PathBuf::from(p))
            .ok_or_else(|| EvalError::FixtureNotFound {
                kind: "flows".into(),
                name: name.into(),
            })
    }

    pub fn client_path(&self, name: &str) -> Result<PathBuf> {
        self.clients
            .get(name)
            .map(|p| PathBuf::from(p))
            .ok_or_else(|| EvalError::FixtureNotFound {
                kind: "clients".into(),
                name: name.into(),
            })
    }

    pub fn module_path(&self, name: &str) -> Result<PathBuf> {
        self.modules
            .get(name)
            .map(|p| PathBuf::from(p))
            .ok_or_else(|| EvalError::FixtureNotFound {
                kind: "modules".into(),
                name: name.into(),
            })
    }
}

fn resolve_map(map: HashMap<String, String>, base: &Path) -> HashMap<String, String> {
    map.into_iter()
        .map(|(k, v)| {
            let path = Path::new(&v);
            let resolved = if path.is_absolute() {
                path.to_path_buf()
            } else {
                base.join(path)
            };
            (k, resolved.to_string_lossy().into_owned())
        })
        .collect()
}

impl EvalScenario {
    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(EvalError::Load {
                path: self.source_path.display().to_string(),
                message: "name must not be empty".into(),
            });
        }
        if self.user_message.trim().is_empty() {
            return Err(EvalError::Load {
                path: self.source_path.display().to_string(),
                message: "user_message must not be empty".into(),
            });
        }
        if let Some(router) = &self.expected.router {
            validate_router_type(router, &self.source_path)?;
        }
        Ok(())
    }
}

fn validate_router_type(router: &ExpectedRouter, path: &Path) -> Result<()> {
    let valid = ["flow", "module", "clarify", "end", "error"];
    if !valid.contains(&router.decision_type.as_str()) {
        return Err(EvalError::Load {
            path: path.display().to_string(),
            message: format!(
                "invalid router type '{}', expected one of {:?}",
                router.decision_type, valid
            ),
        });
    }
    Ok(())
}
