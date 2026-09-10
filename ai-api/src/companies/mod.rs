use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompaniesError {
    #[error("companies registry not found: {0}")]
    NotFound(String),
    #[error("invalid companies registry: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct CompaniesRegistryFile {
    pub companies: Vec<CompanyRegistryEntry>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct CompanyRegistryEntry {
    pub id: String,
    pub name: String,
    pub prompt_set: String,
    pub knowledge_source: String,
    pub flows: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CompaniesRegistry {
    entries: Vec<CompanyRegistryEntry>,
    path: PathBuf,
}

impl CompaniesRegistry {
    pub fn load(data_root: &Path) -> Result<Self, CompaniesError> {
        let path = data_root.join("companies/registry.yaml");
        if !path.exists() {
            return Err(CompaniesError::NotFound(path.display().to_string()));
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| CompaniesError::Invalid(format!("read {}: {e}", path.display())))?;
        let file: CompaniesRegistryFile = serde_yaml::from_str(&content)
            .map_err(|e| CompaniesError::Invalid(e.to_string()))?;
        Ok(Self {
            entries: file.companies,
            path,
        })
    }

    pub fn list(&self) -> &[CompanyRegistryEntry] {
        &self.entries
    }

    pub fn get(&self, id: &str) -> Option<&CompanyRegistryEntry> {
        self.entries.iter().find(|c| c.id == id)
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn reload(data_root: &Path) -> Result<Self, CompaniesError> {
        Self::load(data_root)
    }

    pub fn save(&self, data_root: &Path) -> Result<(), CompaniesError> {
        let path = data_root.join("companies/registry.yaml");
        let file = CompaniesRegistryFile {
            companies: self.entries.clone(),
        };
        let content = serde_yaml::to_string(&file)
            .map_err(|e| CompaniesError::Invalid(e.to_string()))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| CompaniesError::Invalid(format!("mkdir: {e}")))?;
        }
        fs::write(&path, content)
            .map_err(|e| CompaniesError::Invalid(format!("write {}: {e}", path.display())))?;
        Ok(())
    }

    pub fn entries_mut(&mut self) -> &mut Vec<CompanyRegistryEntry> {
        &mut self.entries
    }

    pub fn from_entries(entries: Vec<CompanyRegistryEntry>, path: PathBuf) -> Self {
        Self { entries, path }
    }
}
