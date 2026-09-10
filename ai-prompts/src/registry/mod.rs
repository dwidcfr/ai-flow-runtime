use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ai_response_engine::AgentIdentity;

use crate::error::{PromptError, Result};
use crate::loader::{discover_set_dirs, load_prompt_set, load_registry_config};
use crate::types::{PromptMeta, PromptSet, PromptTemplate};

pub struct PromptRegistry {
    source_root: PathBuf,
    default_set_id: String,
    sets: HashMap<String, PromptSet>,
}

impl PromptRegistry {
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let source_root = root.as_ref().to_path_buf();
        let load_root = if source_root.join("prompts").exists() {
            source_root.join("prompts")
        } else {
            source_root.clone()
        };

        let config = load_registry_config(&load_root)?;
        let mut sets = HashMap::new();

        for set_dir in discover_set_dirs(&load_root)? {
            let set = load_prompt_set(&set_dir)?;
            sets.insert(set.meta.id.clone(), set);
        }

        if !sets.contains_key(&config.default_set) {
            return Err(PromptError::DefaultSetNotFound(config.default_set));
        }

        Ok(Self {
            source_root,
            default_set_id: config.default_set,
            sets,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        let reloaded = Self::load(&self.source_root)?;
        self.default_set_id = reloaded.default_set_id;
        self.sets = reloaded.sets;
        Ok(())
    }

    pub fn default_set_id(&self) -> &str {
        &self.default_set_id
    }

    pub fn list_sets(&self) -> Vec<PromptMeta> {
        let mut metas: Vec<_> = self.sets.values().map(|s| s.meta.clone()).collect();
        metas.sort_by(|a, b| a.id.cmp(&b.id));
        metas
    }

    pub fn has_set(&self, set_id: &str) -> bool {
        self.sets.contains_key(set_id)
    }

    pub fn identity(&self, set_id: &str) -> Result<AgentIdentity> {
        self.get_set(set_id).map(|s| s.identity.clone())
    }

    pub fn router_template(&self, set_id: &str) -> Result<&PromptTemplate> {
        Ok(&self.get_set(set_id)?.router)
    }

    pub fn response_template(&self, set_id: &str) -> Result<&PromptTemplate> {
        Ok(&self.get_set(set_id)?.response)
    }

    fn get_set(&self, set_id: &str) -> Result<&PromptSet> {
        self.sets
            .get(set_id)
            .ok_or_else(|| PromptError::SetNotFound(set_id.into()))
    }
}

pub fn bundled_prompts_root() -> PathBuf {
    if let Ok(custom) = std::env::var("PROMPTS_ROOT") {
        return PathBuf::from(custom);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
