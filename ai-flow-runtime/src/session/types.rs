use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Result, RuntimeError};
use crate::modules::{ModuleInstance, ModuleRef};
use crate::state::RuntimeState;

pub type ClientData = HashMap<String, Value>;
pub type ConversationContext = HashMap<String, String>;
pub type SessionMetadata = HashMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistoryRole {
    System,
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub role: HistoryRole,
    pub content: String,
    pub event: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct Session {
    pub session_id: String,
    pub flow_id: Option<String>,
    pub current_node: Option<String>,
    pub paused_node: Option<String>,
    pub runtime_state: RuntimeState,
    pub client_data: ClientData,
    pub active_module: Option<ModuleRef>,
    #[allow(dead_code)]
    module_instances: HashMap<String, Box<dyn ModuleInstance>>,
    pub history: Vec<HistoryEntry>,
    pub metadata: SessionMetadata,
}

impl Session {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            flow_id: None,
            current_node: None,
            paused_node: None,
            runtime_state: RuntimeState::Idle,
            client_data: ClientData::new(),
            active_module: None,
            module_instances: HashMap::new(),
            history: Vec::new(),
            metadata: SessionMetadata::new(),
        }
    }

    pub fn record_event(&mut self, event: &str, content: &str) {
        self.history.push(HistoryEntry {
            role: HistoryRole::System,
            content: content.to_string(),
            event: Some(event.to_string()),
            timestamp: Utc::now(),
        });
    }

    pub fn record_message(&mut self, role: HistoryRole, content: &str) {
        self.history.push(HistoryEntry {
            role,
            content: content.to_string(),
            event: None,
            timestamp: Utc::now(),
        });
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: Value) {
        self.metadata.insert(key.into(), value);
    }

    pub fn get_metadata(&self, key: &str) -> Option<&Value> {
        self.metadata.get(key)
    }

    pub fn module_instance(&self, module_id: &str) -> Option<&dyn ModuleInstance> {
        self.module_instances
            .get(module_id)
            .map(|i| i.as_ref() as &dyn ModuleInstance)
    }

    pub fn ensure_module_instance(
        &mut self,
        module_id: &str,
        instance: Box<dyn ModuleInstance>,
    ) {
        self.module_instances.insert(module_id.to_string(), instance);
    }

    pub fn has_module_instance(&self, module_id: &str) -> bool {
        self.module_instances.contains_key(module_id)
    }

    pub fn with_module_instance_mut<F, T>(&mut self, module_id: &str, f: F) -> Result<T>
    where
        F: FnOnce(&mut Box<dyn ModuleInstance>) -> Result<T>,
    {
        let instance = self
            .module_instances
            .get_mut(module_id)
            .ok_or_else(|| RuntimeError::ModuleInstanceNotLoaded(module_id.to_string()))?;
        f(instance)
    }

    pub fn clear_modules(&mut self) {
        for instance in self.module_instances.values_mut() {
            instance.clear();
        }
        self.module_instances.clear();
        self.client_data.clear();
        self.active_module = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_message_and_metadata() {
        let mut session = Session::new("s1".to_string());
        session.record_message(HistoryRole::User, "Привет");
        session.set_metadata("channel", Value::String("test".to_string()));

        assert_eq!(session.history.len(), 1);
        assert_eq!(session.history[0].role, HistoryRole::User);
        assert_eq!(
            session.get_metadata("channel").and_then(|v| v.as_str()),
            Some("test")
        );
    }
}
