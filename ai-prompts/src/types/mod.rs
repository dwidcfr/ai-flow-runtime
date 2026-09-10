use std::collections::HashMap;

use ai_response_engine::AgentIdentity;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct PromptMeta {
    pub id: String,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryConfig {
    pub default_set: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MissingPlaceholderPolicy {
    #[default]
    Empty,
    Keep,
    Error,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PlaceholderConfig {
    #[serde(default)]
    pub declared: Vec<String>,
    #[serde(default)]
    pub missing_policy: MissingPlaceholderPolicy,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptTemplate {
    pub system: String,
    pub developer: String,
    pub assembly: String,
    #[serde(default)]
    pub placeholders: PlaceholderConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TemplateFile {
    pub meta: PromptMeta,
    pub template: PromptTemplate,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentityFile {
    pub meta: PromptMeta,
    pub identity: IdentityYaml,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IdentityYaml {
    pub name: String,
    #[serde(default)]
    pub bank_name: String,
    pub role: String,
    pub language: String,
    pub communication_style: String,
    #[serde(default)]
    pub tone: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub conversation_rules: Vec<String>,
    #[serde(default)]
    pub additional_instructions: String,
}

impl IdentityYaml {
    pub fn to_agent_identity(&self) -> AgentIdentity {
        AgentIdentity {
            name: self.name.clone(),
            bank_name: self.bank_name.clone(),
            role: self.role.clone(),
            communication_style: self.communication_style.clone(),
            language: self.language.clone(),
            tone: self.tone.clone(),
            constraints: self.constraints.clone(),
            conversation_rules: self.conversation_rules.clone(),
            additional_instructions: self.additional_instructions.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PromptSet {
    pub meta: PromptMeta,
    pub identity: AgentIdentity,
    pub router: PromptTemplate,
    pub response: PromptTemplate,
}

#[derive(Debug, Clone, Default)]
pub struct RenderVars {
    pub values: HashMap<String, String>,
}

impl RenderVars {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.values.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }
}
