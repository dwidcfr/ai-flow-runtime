use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{Result, RuntimeError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleContent {
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleRef {
    pub module_id: String,
    pub section: Option<String>,
}

impl ModuleRef {
    pub fn parse(module_ref: &str) -> Result<Self> {
        let parts: Vec<&str> = module_ref.split('.').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return Err(RuntimeError::InvalidModuleRef(module_ref.to_string()));
        }

        Ok(Self {
            module_id: parts[0].to_lowercase(),
            section: if parts.len() > 1 {
                Some(parts[1..].join("."))
            } else {
                None
            },
        })
    }

    pub fn id(&self) -> String {
        match &self.section {
            Some(section) => format!("{}.{}", self.module_id, section),
            None => self.module_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_module_ref_with_section() {
        let m = ModuleRef::parse("company.payment_methods").unwrap();
        assert_eq!(m.module_id, "company");
        assert_eq!(m.section.as_deref(), Some("payment_methods"));
    }

    #[test]
    fn parse_module_ref_without_section() {
        let m = ModuleRef::parse("client").unwrap();
        assert_eq!(m.module_id, "client");
        assert!(m.section.is_none());
    }
}
