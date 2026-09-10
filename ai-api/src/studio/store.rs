use std::fs;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde_json;

use crate::companies::CompanyRegistryEntry;
use crate::config::ApiConfig;
use crate::studio::types::{
    DraftCompany, DraftFlow, DraftKnowledge, DraftManifest, DraftPromptSet, VersionManifest,
};

#[derive(Debug, Clone)]
pub struct StudioStore {
    root: PathBuf,
}

impl StudioStore {
    pub fn new(data_root: &Path) -> Self {
        Self {
            root: data_root.join("studio"),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn draft_dir(&self, project_id: &str) -> PathBuf {
        self.root.join("drafts").join(project_id)
    }

    pub fn versions_dir(&self, project_id: &str) -> PathBuf {
        self.root.join("versions").join(project_id)
    }

    pub fn ensure_studio_dirs(&self) -> std::io::Result<()> {
        fs::create_dir_all(self.root.join("drafts"))?;
        fs::create_dir_all(self.root.join("versions"))?;
        Ok(())
    }

    pub fn draft_exists(&self, project_id: &str) -> bool {
        self.draft_dir(project_id).join("manifest.json").exists()
    }

    pub fn read_manifest(&self, project_id: &str) -> Result<DraftManifest, String> {
        let path = self.draft_dir(project_id).join("manifest.json");
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_manifest(&self, manifest: &DraftManifest) -> Result<(), String> {
        let dir = self.draft_dir(&manifest.project_id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let path = dir.join("manifest.json");
        let content = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    pub fn touch_dirty(&self, project_id: &str) -> Result<(), String> {
        let mut manifest = self
            .read_manifest(project_id)
            .unwrap_or_else(|_| DraftManifest::new(project_id.to_string()));
        manifest.dirty = true;
        manifest.edited_at = Some(Utc::now().to_rfc3339());
        self.write_manifest(&manifest)
    }

    pub fn read_company(&self, project_id: &str) -> Result<DraftCompany, String> {
        let path = self.draft_dir(project_id).join("company.json");
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_company(&self, project_id: &str, company: &DraftCompany) -> Result<(), String> {
        let dir = self.draft_dir(project_id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(company).map_err(|e| e.to_string())?;
        fs::write(dir.join("company.json"), content).map_err(|e| e.to_string())?;
        self.touch_dirty(project_id)
    }

    pub fn read_flow(&self, project_id: &str, flow_id: &str) -> Result<DraftFlow, String> {
        let path = self.draft_dir(project_id).join("flows").join(format!("{flow_id}.json"));
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_flow(&self, project_id: &str, flow: &DraftFlow) -> Result<(), String> {
        let dir = self.draft_dir(project_id).join("flows");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(flow).map_err(|e| e.to_string())?;
        fs::write(dir.join(format!("{}.json", flow.id)), content).map_err(|e| e.to_string())?;
        self.touch_dirty(project_id)
    }

    pub fn list_flow_ids(&self, project_id: &str) -> Result<Vec<String>, String> {
        let dir = self.draft_dir(project_id).join("flows");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub fn read_knowledge(&self, project_id: &str) -> Result<DraftKnowledge, String> {
        let path = self.draft_dir(project_id).join("knowledge.json");
        if !path.exists() {
            return Ok(DraftKnowledge::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_knowledge(&self, project_id: &str, knowledge: &DraftKnowledge) -> Result<(), String> {
        let dir = self.draft_dir(project_id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(knowledge).map_err(|e| e.to_string())?;
        fs::write(dir.join("knowledge.json"), content).map_err(|e| e.to_string())?;
        self.touch_dirty(project_id)
    }

    pub fn read_prompt(&self, project_id: &str, set_id: &str) -> Result<DraftPromptSet, String> {
        let path = self
            .draft_dir(project_id)
            .join("prompts")
            .join(format!("{set_id}.json"));
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_prompt(&self, project_id: &str, prompt: &DraftPromptSet) -> Result<(), String> {
        let dir = self.draft_dir(project_id).join("prompts");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(prompt).map_err(|e| e.to_string())?;
        fs::write(dir.join(format!("{}.json", prompt.id)), content).map_err(|e| e.to_string())?;
        self.touch_dirty(project_id)
    }

    pub fn list_client_names(&self, project_id: &str) -> Result<Vec<String>, String> {
        let dir = self.draft_dir(project_id).join("clients");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut names = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    names.push(stem.to_string());
                }
            }
        }
        names.sort();
        Ok(names)
    }

    pub fn read_client(&self, project_id: &str, name: &str) -> Result<serde_json::Value, String> {
        let path = self
            .draft_dir(project_id)
            .join("clients")
            .join(format!("{name}.json"));
        let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| e.to_string())
    }

    pub fn write_client(
        &self,
        project_id: &str,
        name: &str,
        data: &serde_json::Value,
    ) -> Result<(), String> {
        let dir = self.draft_dir(project_id).join("clients");
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
        fs::write(dir.join(format!("{name}.json")), content).map_err(|e| e.to_string())?;
        self.touch_dirty(project_id)
    }

    pub fn delete_client(&self, project_id: &str, name: &str) -> Result<(), String> {
        let path = self
            .draft_dir(project_id)
            .join("clients")
            .join(format!("{name}.json"));
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        self.touch_dirty(project_id)
    }

    pub fn delete_flow(&self, project_id: &str, flow_id: &str) -> Result<(), String> {
        let path = self
            .draft_dir(project_id)
            .join("flows")
            .join(format!("{flow_id}.json"));
        if path.exists() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        if let Ok(mut company) = self.read_company(project_id) {
            company.flows.retain(|f| f != flow_id);
            let _ = self.write_company(project_id, &company);
        }
        self.touch_dirty(project_id)
    }

    pub fn list_prompt_ids(&self, project_id: &str) -> Result<Vec<String>, String> {
        let dir = self.draft_dir(project_id).join("prompts");
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut ids = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = entry.path().file_stem().and_then(|s| s.to_str()) {
                    ids.push(stem.to_string());
                }
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub fn delete_project(&self, project_id: &str) -> Result<(), String> {
        let draft = self.draft_dir(project_id);
        if draft.exists() {
            fs::remove_dir_all(&draft).map_err(|e| e.to_string())?;
        }
        let versions = self.versions_dir(project_id);
        if versions.exists() {
            fs::remove_dir_all(&versions).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn latest_version(&self, project_id: &str) -> u32 {
        let dir = self.versions_dir(project_id);
        if !dir.exists() {
            return 0;
        }
        let mut max = 0u32;
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(num) = name.strip_prefix('v').and_then(|s| s.parse::<u32>().ok()) {
                    max = max.max(num);
                }
            }
        }
        max
    }

    pub fn list_versions(&self, project_id: &str) -> Result<Vec<VersionManifest>, String> {
        let dir = self.versions_dir(project_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut versions = Vec::new();
        for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path().join("manifest.json");
            if path.exists() {
                let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                if let Ok(m) = serde_json::from_str::<VersionManifest>(&content) {
                    versions.push(m);
                }
            }
        }
        versions.sort_by_key(|v| v.version);
        Ok(versions)
    }

    pub fn write_version_manifest(&self, manifest: &VersionManifest) -> Result<(), String> {
        let dir = self
            .versions_dir(&manifest.project_id)
            .join(format!("v{}", manifest.version));
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
        fs::write(dir.join("manifest.json"), content).map_err(|e| e.to_string())
    }

    pub fn company_from_registry(entry: &CompanyRegistryEntry) -> DraftCompany {
        DraftCompany {
            id: entry.id.clone(),
            name: entry.name.clone(),
            description: None,
            prompt_set: entry.prompt_set.clone(),
            knowledge_source: entry.knowledge_source.clone(),
            flows: entry.flows.clone(),
        }
    }

    pub fn registry_entry_from_company(company: &DraftCompany) -> CompanyRegistryEntry {
        CompanyRegistryEntry {
            id: company.id.clone(),
            name: company.name.clone(),
            prompt_set: company.prompt_set.clone(),
            knowledge_source: company.knowledge_source.clone(),
            flows: company.flows.clone(),
        }
    }

    pub fn knowledge_source_path(config: &ApiConfig, knowledge_source: &str) -> PathBuf {
        config.resolve_data_path(knowledge_source)
    }

    pub fn prompts_root() -> PathBuf {
        std::env::var("PROMPTS_ROOT")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ai-prompts/prompts")
            })
    }
}
