use std::collections::HashMap;

use crate::error::{Result, RuntimeError};
use crate::modules::{ModuleFactory, ModuleInfo, ModuleInstance};

pub struct ModuleRegistry {
    factories: HashMap<String, Box<dyn ModuleFactory>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    pub fn register(&mut self, factory: Box<dyn ModuleFactory>) {
        let id = factory.module().id().to_string();
        self.factories.insert(id, factory);
    }

    pub fn get_factory(&self, id: &str) -> Result<&dyn ModuleFactory> {
        self.factories
            .get(id)
            .map(|f| f.as_ref() as &dyn ModuleFactory)
            .ok_or_else(|| RuntimeError::ModuleNotFound(id.to_string()))
    }

    pub fn has_module(&self, id: &str) -> bool {
        self.factories.contains_key(id)
    }

    pub fn list_modules(&self) -> Vec<ModuleInfo> {
        let mut modules: Vec<ModuleInfo> = self
            .factories
            .values()
            .map(|f| {
                let m = f.module();
                ModuleInfo {
                    id: m.id().to_string(),
                    name: m.name().to_string(),
                    description: m.description().to_string(),
                }
            })
            .collect();
        modules.sort_by(|a, b| a.id.cmp(&b.id));
        modules
    }

    pub fn create_instance(&self, id: &str) -> Result<Box<dyn ModuleInstance>> {
        Ok(self.get_factory(id)?.create_instance())
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::conversation::ConversationModuleFactory;

    #[test]
    fn register_and_list() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(ConversationModuleFactory::new()));
        assert!(registry.has_module("conversation"));
        assert_eq!(registry.list_modules().len(), 1);
    }

    #[test]
    fn create_instance() {
        let mut registry = ModuleRegistry::new();
        registry.register(Box::new(ConversationModuleFactory::new()));
        let instance = registry.create_instance("conversation").unwrap();
        assert_eq!(instance.module_id(), "conversation");
    }

    #[test]
    fn unknown_module_fails() {
        let registry = ModuleRegistry::new();
        let err = registry.create_instance("unknown");
        assert!(matches!(err, Err(RuntimeError::ModuleNotFound(_))));
    }
}
