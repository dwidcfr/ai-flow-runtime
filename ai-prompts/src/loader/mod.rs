use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{PromptError, Result};
use crate::types::{
    IdentityFile, PromptMeta, PromptSet, RegistryConfig, TemplateFile,
};

const IDENTITY_FILE: &str = "identity.yaml";
const ROUTER_FILE: &str = "router.yaml";
const RESPONSE_FILE: &str = "response.yaml";

pub fn load_registry_config(root: &Path) -> Result<RegistryConfig> {
    let path = root.join("registry.yaml");
    let content = fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            PromptError::MissingFile(path.display().to_string())
        } else {
            PromptError::Io(e)
        }
    })?;
    serde_yaml::from_str(&content).map_err(|e| PromptError::InvalidYaml {
        path: path.display().to_string(),
        message: e.to_string(),
    })
}

pub fn discover_set_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !name.starts_with('.') {
                dirs.push(path);
            }
        }
    }
    dirs.sort();
    Ok(dirs)
}

pub fn load_prompt_set(set_dir: &Path) -> Result<PromptSet> {
    let set_id = set_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    for file in [IDENTITY_FILE, ROUTER_FILE, RESPONSE_FILE] {
        let path = set_dir.join(file);
        if !path.exists() {
            return Err(PromptError::MissingFile(path.display().to_string()));
        }
    }

    let identity = load_identity_file(&set_dir.join(IDENTITY_FILE))?;
    let router = load_template_file(&set_dir.join(ROUTER_FILE))?;
    let response = load_template_file(&set_dir.join(RESPONSE_FILE))?;

    validate_meta_id(&set_id, &identity.meta)?;
    validate_meta_id(&set_id, &router.meta)?;
    validate_meta_id(&set_id, &response.meta)?;
    validate_versions(&set_id, &identity.meta, &router.meta, &response.meta)?;

    crate::validation::validate_template(&router.template, &set_dir.join(ROUTER_FILE), true)?;
    crate::validation::validate_template(
        &response.template,
        &set_dir.join(RESPONSE_FILE),
        false,
    )?;

    Ok(PromptSet {
        meta: identity.meta.clone(),
        identity: identity.identity.to_agent_identity(),
        router: router.template,
        response: response.template,
    })
}

fn load_identity_file(path: &Path) -> Result<IdentityFile> {
    let content = fs::read_to_string(path)?;
    let file: IdentityFile = serde_yaml::from_str(&content).map_err(|e| PromptError::InvalidYaml {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    if file.identity.name.trim().is_empty() {
        return Err(PromptError::MissingField {
            field: "identity.name".into(),
            path: path.display().to_string(),
        });
    }
    Ok(file)
}

fn load_template_file(path: &Path) -> Result<TemplateFile> {
    let content = fs::read_to_string(path)?;
    let file: TemplateFile = serde_yaml::from_str(&content).map_err(|e| PromptError::InvalidYaml {
        path: path.display().to_string(),
        message: e.to_string(),
    })?;
    if file.template.system.trim().is_empty() {
        return Err(PromptError::MissingField {
            field: "template.system".into(),
            path: path.display().to_string(),
        });
    }
    if file.template.assembly.trim().is_empty() {
        return Err(PromptError::MissingField {
            field: "template.assembly".into(),
            path: path.display().to_string(),
        });
    }
    Ok(file)
}

fn validate_meta_id(dir_name: &str, meta: &PromptMeta) -> Result<()> {
    if meta.id != dir_name {
        return Err(PromptError::MetaIdMismatch {
            dir: dir_name.into(),
            meta_id: meta.id.clone(),
        });
    }
    if meta.version.trim().is_empty() {
        return Err(PromptError::MissingField {
            field: "meta.version".into(),
            path: dir_name.into(),
        });
    }
    Ok(())
}

fn validate_versions(
    set_id: &str,
    identity: &PromptMeta,
    router: &PromptMeta,
    response: &PromptMeta,
) -> Result<()> {
    let versions: HashSet<_> = [&identity.version, &router.version, &response.version]
        .into_iter()
        .collect();
    if versions.len() > 1 {
        return Err(PromptError::VersionMismatch {
            set_id: set_id.into(),
            expected: identity.version.clone(),
            found: router.version.clone(),
        });
    }
    Ok(())
}
