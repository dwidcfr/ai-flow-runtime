use std::collections::HashMap;
use std::fs;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::{Result, RuntimeError};
use crate::modules::{
    Module, ModuleContent, ModuleFactory, ModuleInstance, SMALLTALK_MODULE_ID,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallTalkSection {
    description: String,
    content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SmallTalkFile {
    sections: HashMap<String, SmallTalkSection>,
}

#[derive(Debug, Clone)]
struct SmallTalkModuleMeta;

impl Module for SmallTalkModuleMeta {
    fn id(&self) -> &str {
        SMALLTALK_MODULE_ID
    }

    fn name(&self) -> &str {
        "SmallTalk"
    }

    fn description(&self) -> &str {
        "Small talk scenarios and neutral responses"
    }
}

pub struct SmallTalkModuleFactory {
    meta: SmallTalkModuleMeta,
}

impl SmallTalkModuleFactory {
    pub fn new() -> Self {
        Self {
            meta: SmallTalkModuleMeta,
        }
    }
}

impl Default for SmallTalkModuleFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleFactory for SmallTalkModuleFactory {
    fn module(&self) -> &dyn Module {
        &self.meta
    }

    fn create_instance(&self) -> Box<dyn ModuleInstance> {
        Box::new(SmallTalkModuleInstance::default())
    }
}

#[derive(Debug, Default)]
struct SmallTalkModuleInstance {
    sections: HashMap<String, SmallTalkSection>,
}

impl ModuleInstance for SmallTalkModuleInstance {
    fn module_id(&self) -> &str {
        SMALLTALK_MODULE_ID
    }

    fn load(&mut self, source: &str) -> Result<()> {
        let content = fs::read_to_string(source).map_err(|e| RuntimeError::ModuleLoadError {
            module_id: SMALLTALK_MODULE_ID.to_string(),
            message: format!("failed to read {source}: {e}"),
        })?;
        let parsed: SmallTalkFile =
            serde_yaml::from_str(&content).map_err(|e| RuntimeError::ModuleLoadError {
                module_id: SMALLTALK_MODULE_ID.to_string(),
                message: format!("failed to parse YAML: {e}"),
            })?;
        self.sections = parsed.sections;
        Ok(())
    }

    fn get_content(&self, section: Option<&str>) -> Result<ModuleContent> {
        match section {
            None => {
                let map: serde_json::Map<String, Value> = self
                    .sections
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            json!({
                                "description": v.description,
                                "content": v.content,
                            }),
                        )
                    })
                    .collect();
                Ok(ModuleContent {
                    data: Value::Object(map),
                })
            }
            Some(key) => {
                let sec = self
                    .sections
                    .get(key)
                    .ok_or_else(|| RuntimeError::ModuleSectionNotFound {
                        module_id: SMALLTALK_MODULE_ID.to_string(),
                        section: key.to_string(),
                    })?;
                Ok(ModuleContent {
                    data: json!({
                        "description": sec.description,
                        "content": sec.content,
                    }),
                })
            }
        }
    }

    fn set_entry(&mut self, _key: &str, _value: &str) -> Result<()> {
        Err(RuntimeError::ModuleOperationNotSupported {
            module_id: SMALLTALK_MODULE_ID.to_string(),
            operation: "set_entry".to_string(),
        })
    }

    fn clear(&mut self) {
        self.sections.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> String {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data/modules/smalltalk.yaml")
            .to_string_lossy()
            .to_string()
    }

    #[test]
    fn load_stub_content() {
        let factory = SmallTalkModuleFactory::new();
        let mut instance = factory.create_instance();
        instance.load(&fixture()).unwrap();
        let content = instance.get_content(Some("greetings")).unwrap();
        assert!(content.data["content"]
            .as_str()
            .unwrap()
            .contains("Добрый день"));
    }
}
