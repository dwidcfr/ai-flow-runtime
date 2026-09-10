use std::fs;

use ai_flow_runtime::Runtime;
use ai_prompts::bundled_prompts_root;
use chrono::Utc;

use crate::companies::CompanyRegistryEntry;
use crate::config::ApiConfig;
use crate::studio::convert::{export_to_draft_flow, prompt_files_to_draft, yaml_to_knowledge};
use crate::studio::store::StudioStore;
use crate::studio::types::{DraftCompany, DraftKnowledge, DraftManifest};

fn prompts_load_root() -> std::path::PathBuf {
    let root = bundled_prompts_root();
    if root.join("prompts").exists() {
        root.join("prompts")
    } else {
        root
    }
}

pub fn bootstrap_project(
    store: &StudioStore,
    config: &ApiConfig,
    runtime: &Runtime,
    entry: &CompanyRegistryEntry,
) -> Result<DraftManifest, String> {
    let project_id = &entry.id;
    if store.draft_exists(project_id) {
        let mut manifest = store.read_manifest(project_id)?;
        ensure_prompts_bootstrapped(store, &entry.prompt_set, project_id)?;
        if manifest.bootstrapped_at.is_none() {
            manifest.bootstrapped_at = Some(Utc::now().to_rfc3339());
            store.write_manifest(&manifest)?;
        }
        return Ok(manifest);
    }

    store.ensure_studio_dirs().map_err(|e| e.to_string())?;
    let company = StudioStore::company_from_registry(entry);
    store.write_company(project_id, &company)?;

    for flow_id in &entry.flows {
        let export = runtime
            .export_flow(flow_id)
            .map_err(|e| format!("export flow {flow_id}: {e}"))?;
        let draft = export_to_draft_flow(&export);
        store.write_flow(project_id, &draft)?;
    }

    let prompts_root = prompts_load_root();
    let set_id = &entry.prompt_set;
    let identity_path = prompts_root.join(set_id).join("identity.yaml");
    let router_path = prompts_root.join(set_id).join("router.yaml");
    let response_path = prompts_root.join(set_id).join("response.yaml");
    if identity_path.exists() && router_path.exists() && response_path.exists() {
        let identity: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&identity_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let router: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&router_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let response: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&response_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let draft = prompt_files_to_draft(set_id, &identity, &router, &response);
        store.write_prompt(project_id, &draft)?;
    }

    let knowledge_path = StudioStore::knowledge_source_path(config, &entry.knowledge_source);
    if knowledge_path.exists() {
        let content = fs::read_to_string(&knowledge_path).map_err(|e| e.to_string())?;
        let knowledge = yaml_to_knowledge(&content);
        store.write_knowledge(project_id, &knowledge)?;
    }

    let clients_dir = config.data_root.join("clients");
    if clients_dir.exists() {
        for file in fs::read_dir(&clients_dir).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            let path = file.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
                    let data: serde_json::Value =
                        serde_json::from_str(&content).map_err(|e| e.to_string())?;
                    store.write_client(project_id, name, &data)?;
                }
            }
        }
    }

    let manifest = DraftManifest {
        project_id: project_id.clone(),
        dirty: false,
        edited_at: None,
        published_version: store.latest_version(project_id),
        bootstrapped_at: Some(Utc::now().to_rfc3339()),
    };
    store.write_manifest(&manifest)?;
    Ok(manifest)
}

fn ensure_prompts_bootstrapped(
    store: &StudioStore,
    set_id: &str,
    project_id: &str,
) -> Result<(), String> {
    let prompt_path = store.draft_dir(project_id).join("prompts").join(format!("{set_id}.json"));
    if prompt_path.exists() {
        return Ok(());
    }
    let prompts_root = prompts_load_root();
    let identity_path = prompts_root.join(set_id).join("identity.yaml");
    let router_path = prompts_root.join(set_id).join("router.yaml");
    let response_path = prompts_root.join(set_id).join("response.yaml");
    if identity_path.exists() && router_path.exists() && response_path.exists() {
        let identity: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&identity_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let router: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&router_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let response: serde_yaml::Value =
            serde_yaml::from_str(&fs::read_to_string(&response_path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        let draft = prompt_files_to_draft(set_id, &identity, &router, &response);
        store.write_prompt(project_id, &draft)?;
    }
    Ok(())
}

pub fn bootstrap_new_project(store: &StudioStore, company: DraftCompany) -> Result<DraftManifest, String> {
    store.ensure_studio_dirs().map_err(|e| e.to_string())?;
    store.write_company(&company.id, &company)?;
    store.write_knowledge(&company.id, &DraftKnowledge::default())?;

    let manifest = DraftManifest {
        project_id: company.id.clone(),
        dirty: true,
        edited_at: Some(Utc::now().to_rfc3339()),
        published_version: 0,
        bootstrapped_at: Some(Utc::now().to_rfc3339()),
    };
    store.write_manifest(&manifest)?;
    Ok(manifest)
}
