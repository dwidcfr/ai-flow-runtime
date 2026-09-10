use std::collections::HashMap;

use serde_json::{json, Value};

use crate::error::{Result, RuntimeError};
use crate::modules::{
    Module, ModuleContent, ModuleFactory, ModuleInstance, CONVERSATION_MODULE_ID,
};

#[derive(Debug, Clone)]
struct ConversationModuleMeta;

impl Module for ConversationModuleMeta {
    fn id(&self) -> &str {
        CONVERSATION_MODULE_ID
    }

    fn name(&self) -> &str {
        "Conversation"
    }

    fn description(&self) -> &str {
        "In-memory conversation context for the current session"
    }
}

pub struct ConversationModuleFactory {
    meta: ConversationModuleMeta,
}

impl ConversationModuleFactory {
    pub fn new() -> Self {
        Self {
            meta: ConversationModuleMeta,
        }
    }
}

impl Default for ConversationModuleFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleFactory for ConversationModuleFactory {
    fn module(&self) -> &dyn Module {
        &self.meta
    }

    fn create_instance(&self) -> Box<dyn ModuleInstance> {
        Box::new(ConversationModuleInstance::default())
    }
}

#[derive(Debug, Default)]
struct ConversationModuleInstance {
    entries: HashMap<String, String>,
}

impl ModuleInstance for ConversationModuleInstance {
    fn module_id(&self) -> &str {
        CONVERSATION_MODULE_ID
    }

    fn load(&mut self, _source: &str) -> Result<()> {
        Ok(())
    }

    fn get_content(&self, section: Option<&str>) -> Result<ModuleContent> {
        match section {
            None => Ok(ModuleContent {
                data: Value::Object(
                    self.entries
                        .iter()
                        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
                        .collect(),
                ),
            }),
            Some(key) => {
                let value = self.entries.get(key).ok_or_else(|| {
                    RuntimeError::ModuleSectionNotFound {
                        module_id: CONVERSATION_MODULE_ID.to_string(),
                        section: key.to_string(),
                    }
                })?;
                Ok(ModuleContent {
                    data: json!({ key: value }),
                })
            }
        }
    }

    fn set_entry(&mut self, key: &str, value: &str) -> Result<()> {
        self.entries.insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn clear(&mut self) {
        self.entries.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_get_entry() {
        let factory = ConversationModuleFactory::new();
        let mut instance = factory.create_instance();
        instance
            .set_entry("payment_promise", "вечером")
            .unwrap();
        let content = instance.get_content(Some("payment_promise")).unwrap();
        assert_eq!(content.data["payment_promise"], "вечером");
    }

    #[test]
    fn clear_removes_entries() {
        let factory = ConversationModuleFactory::new();
        let mut instance = factory.create_instance();
        instance.set_entry("key", "value").unwrap();
        instance.clear();
        let content = instance.get_content(None).unwrap();
        assert!(content.data.as_object().unwrap().is_empty());
    }
}
