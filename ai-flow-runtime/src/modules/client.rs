use std::collections::HashMap;
use std::fs;

use serde_json::{json, Value};

use crate::error::{Result, RuntimeError};
use crate::modules::{
    Module, ModuleContent, ModuleFactory, ModuleInstance, CLIENT_MODULE_ID,
};

#[derive(Debug, Clone)]
struct ClientModuleMeta;

impl Module for ClientModuleMeta {
    fn id(&self) -> &str {
        CLIENT_MODULE_ID
    }

    fn name(&self) -> &str {
        "Client"
    }

    fn description(&self) -> &str {
        "Client-specific data loaded from JSON"
    }
}

pub struct ClientModuleFactory {
    meta: ClientModuleMeta,
}

impl ClientModuleFactory {
    pub fn new() -> Self {
        Self {
            meta: ClientModuleMeta,
        }
    }
}

impl Default for ClientModuleFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleFactory for ClientModuleFactory {
    fn module(&self) -> &dyn Module {
        &self.meta
    }

    fn create_instance(&self) -> Box<dyn ModuleInstance> {
        Box::new(ClientModuleInstance {
            data: HashMap::new(),
        })
    }
}

struct ClientModuleInstance {
    data: HashMap<String, Value>,
}

impl ModuleInstance for ClientModuleInstance {
    fn module_id(&self) -> &str {
        CLIENT_MODULE_ID
    }

    fn load(&mut self, source: &str) -> Result<()> {
        let content = fs::read_to_string(source).map_err(|e| RuntimeError::ModuleLoadError {
            module_id: CLIENT_MODULE_ID.to_string(),
            message: format!("failed to read {source}: {e}"),
        })?;
        let parsed: HashMap<String, Value> =
            serde_json::from_str(&content).map_err(|e| RuntimeError::ModuleLoadError {
                module_id: CLIENT_MODULE_ID.to_string(),
                message: format!("failed to parse JSON: {e}"),
            })?;
        self.data = parsed;
        Ok(())
    }

    fn get_content(&self, section: Option<&str>) -> Result<ModuleContent> {
        match section {
            None => Ok(ModuleContent {
                data: Value::Object(
                    self.data
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect(),
                ),
            }),
            Some(key) => {
                let value = self.data.get(key).ok_or_else(|| RuntimeError::ModuleSectionNotFound {
                    module_id: CLIENT_MODULE_ID.to_string(),
                    section: key.to_string(),
                })?;
                Ok(ModuleContent {
                    data: json!({ key: value }),
                })
            }
        }
    }

    fn set_entry(&mut self, _key: &str, _value: &str) -> Result<()> {
        Err(RuntimeError::ModuleOperationNotSupported {
            module_id: CLIENT_MODULE_ID.to_string(),
            operation: "set_entry".to_string(),
        })
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/clients/sample.json")
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn load_and_get_content() {
        let factory = ClientModuleFactory::new();
        let mut instance = factory.create_instance();
        instance.load(&fixture()).unwrap();
        let content = instance.get_content(None).unwrap();
        assert_eq!(content.data["client_name"], "Sodiq");
    }

    #[test]
    fn get_section() {
        let factory = ClientModuleFactory::new();
        let mut instance = factory.create_instance();
        instance.load(&fixture()).unwrap();
        let content = instance.get_content(Some("client_name")).unwrap();
        assert_eq!(content.data["client_name"], "Sodiq");
    }
}
