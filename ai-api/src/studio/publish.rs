use std::fs;
use std::path::Path;

use ai_evals::{EvalRunRequest, run_evals_request};
use ai_prompts::bundled_prompts_root;
use chrono::Utc;
use serde_json::json;

use crate::companies::CompaniesRegistry;
use crate::config::ApiConfig;
use crate::dto::studio::PublishResponse;
use crate::errors::ApiError;
use crate::state::{with_runtime, AppState};
use crate::studio::convert::{
    draft_flow_to_yaml, knowledge_to_yaml, validate_draft_flow,
};
use crate::studio::store::StudioStore;
use crate::studio::types::{DraftManifest, VersionManifest};

fn prompts_publish_root() -> std::path::PathBuf {
    let root = bundled_prompts_root();
    if root.join("prompts").exists() {
        root.join("prompts")
    } else {
        root
    }
}

pub async fn execute_publish(state: &AppState, project_id: &str) -> Result<PublishResponse, ApiError> {
    let store = &state.studio;
    let config = &state.config;

    let company = store
        .read_company(project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let flow_ids = store
        .list_flow_ids(project_id)
        .map_err(|e| ApiError::invalid_request(e))?;
    let knowledge = store
        .read_knowledge(project_id)
        .map_err(|e| ApiError::invalid_request(e))?;

    for flow_id in &flow_ids {
        let flow = store
            .read_flow(project_id, flow_id)
            .map_err(|e| ApiError::invalid_request(e))?;
        let (valid, issues) = validate_draft_flow(&flow);
        if !valid {
            return Ok(PublishResponse {
                status: "failed".into(),
                version: None,
                stage: Some("validate".into()),
                error: Some(format!("flow {} invalid: {:?}", flow.id, issues)),
                report: None,
            });
        }
    }

    let backup_dir = store
        .root()
        .join("backups")
        .join(format!("{project_id}-{}", Utc::now().timestamp()));
    if let Err(e) = backup_published(config, &company, &backup_dir) {
        return Ok(PublishResponse {
            status: "failed".into(),
            version: None,
            stage: Some("backup".into()),
            error: Some(e),
            report: None,
        });
    }

    if let Err(e) = write_published(config, store, project_id, &company, &flow_ids, &knowledge) {
        let _ = restore_backup(config, &company, &backup_dir);
        return Ok(PublishResponse {
            status: "failed".into(),
            version: None,
            stage: Some("save".into()),
            error: Some(e),
            report: None,
        });
    }

    let search_ok = smoke_search_draft(state, project_id, &knowledge).await;
    if !search_ok {
        let _ = restore_backup(config, &company, &backup_dir);
        return Ok(PublishResponse {
            status: "failed".into(),
            version: None,
            stage: Some("reindex".into()),
            error: Some("search smoke test failed".into()),
            report: None,
        });
    }

    if let Err(e) = with_runtime(state, |rt| rt.reload_prompts()).await {
        let _ = restore_backup(config, &company, &backup_dir);
        return Ok(PublishResponse {
            status: "failed".into(),
            version: None,
            stage: Some("reload_prompts".into()),
            error: Some(e.to_string()),
            report: None,
        });
    }

    let flows_dir = config.flows_dir.clone();
    if let Err(e) = with_runtime(state, move |rt| rt.reload_flows_from_dir(&flows_dir)).await {
        let _ = restore_backup(config, &company, &backup_dir);
        return Ok(PublishResponse {
            status: "failed".into(),
            version: None,
            stage: Some("reload_flows".into()),
            error: Some(e.to_string()),
            report: None,
        });
    }

    let eval_report = tokio::task::spawn_blocking(|| {
        run_evals_request(EvalRunRequest {
            scenario: None,
            domain: Some("payment".into()),
            llm: ai_evals::scenario::LlmMode::Mock,
        })
    })
    .await
    .map_err(|e| ApiError::internal(format!("eval task failed: {e}")))?;

    match eval_report {
        Ok(report) if report.metrics.failed > 0 => {
            let _ = restore_backup(config, &company, &backup_dir);
            let flows_dir = config.flows_dir.clone();
            let _ = with_runtime(state, move |rt| rt.reload_flows_from_dir(&flows_dir)).await;
            let _ = with_runtime(state, |rt| rt.reload_prompts()).await;
            return Ok(PublishResponse {
                status: "failed".into(),
                version: None,
                stage: Some("evals".into()),
                error: Some(format!(
                    "evals failed: {}/{}",
                    report.metrics.failed, report.metrics.total
                )),
                report: Some(serde_json::to_value(&report).unwrap_or(json!({}))),
            });
        }
        Err(e) => {
            let _ = restore_backup(config, &company, &backup_dir);
            return Ok(PublishResponse {
                status: "failed".into(),
                version: None,
                stage: Some("evals".into()),
                error: Some(e.to_string()),
                report: None,
            });
        }
        Ok(report) => {
            let new_version = store.latest_version(project_id) + 1;
            if let Err(e) = snapshot_version(store, project_id, new_version, config, &company) {
                return Ok(PublishResponse {
                    status: "failed".into(),
                    version: None,
                    stage: Some("version".into()),
                    error: Some(e),
                    report: None,
                });
            }

            let mut manifest = store
                .read_manifest(project_id)
                .unwrap_or_else(|_| DraftManifest::new(project_id.to_string()));
            manifest.dirty = false;
            manifest.published_version = new_version;
            manifest.edited_at = Some(Utc::now().to_rfc3339());
            let _ = store.write_manifest(&manifest);

            let _ = fs::remove_dir_all(&backup_dir);

            Ok(PublishResponse {
                status: "completed".into(),
                version: Some(new_version),
                stage: Some("done".into()),
                error: None,
                report: Some(serde_json::to_value(&report).unwrap_or(json!({}))),
            })
        }
    }
}

fn backup_published(
    config: &ApiConfig,
    company: &crate::studio::types::DraftCompany,
    backup_dir: &Path,
) -> Result<(), String> {
    fs::create_dir_all(backup_dir).map_err(|e| e.to_string())?;
    let registry = config.data_root.join("companies/registry.yaml");
    if registry.exists() {
        copy_file(&registry, &backup_dir.join("registry.yaml"))?;
    }
    for flow_id in &company.flows {
        let src = config.flows_dir.join(format!("{flow_id}.yaml"));
        if src.exists() {
            copy_file(&src, &backup_dir.join(format!("{flow_id}.yaml")))?;
        }
    }
    let knowledge = StudioStore::knowledge_source_path(config, &company.knowledge_source);
    if knowledge.exists() {
        copy_file(&knowledge, &backup_dir.join("knowledge.yaml"))?;
    }
    Ok(())
}

fn restore_backup(
    config: &ApiConfig,
    company: &crate::studio::types::DraftCompany,
    backup_dir: &Path,
) -> Result<(), String> {
    let registry = backup_dir.join("registry.yaml");
    if registry.exists() {
        copy_file(&registry, &config.data_root.join("companies/registry.yaml"))?;
    }
    for flow_id in &company.flows {
        let src = backup_dir.join(format!("{flow_id}.yaml"));
        if src.exists() {
            copy_file(&src, &config.flows_dir.join(format!("{flow_id}.yaml")))?;
        }
    }
    let knowledge_backup = backup_dir.join("knowledge.yaml");
    if knowledge_backup.exists() {
        copy_file(
            &knowledge_backup,
            &StudioStore::knowledge_source_path(config, &company.knowledge_source),
        )?;
    }
    Ok(())
}

fn write_published(
    config: &ApiConfig,
    store: &StudioStore,
    project_id: &str,
    company: &crate::studio::types::DraftCompany,
    flow_ids: &[String],
    knowledge: &crate::studio::types::DraftKnowledge,
) -> Result<(), String> {
    let mut registry = CompaniesRegistry::load(&config.data_root).map_err(|e| e.to_string())?;
    let entry = StudioStore::registry_entry_from_company(company);
    if let Some(existing) = registry
        .list()
        .iter()
        .position(|c| c.id == company.id)
    {
        *registry.entries_mut().get_mut(existing).unwrap() = entry.clone();
    } else {
        registry.entries_mut().push(entry.clone());
    }
    registry.save(&config.data_root).map_err(|e| e.to_string())?;

    fs::create_dir_all(&config.flows_dir).map_err(|e| e.to_string())?;
    for flow_id in flow_ids {
        let flow = store.read_flow(project_id, flow_id)?;
        let yaml = draft_flow_to_yaml(&flow);
        fs::write(
            config.flows_dir.join(format!("{flow_id}.yaml")),
            yaml,
        )
        .map_err(|e| e.to_string())?;
    }

    let knowledge_yaml = knowledge_to_yaml(knowledge);
    let knowledge_path = StudioStore::knowledge_source_path(config, &company.knowledge_source);
    if let Some(parent) = knowledge_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(knowledge_path, knowledge_yaml).map_err(|e| e.to_string())?;

    if let Ok(prompt) = store.read_prompt(project_id, &company.prompt_set) {
        write_prompt_files(&company.prompt_set, &prompt)?;
    }

    let clients_dir = config.data_root.join("clients");
    fs::create_dir_all(&clients_dir).map_err(|e| e.to_string())?;
    for name in store.list_client_names(project_id)? {
        let data = store.read_client(project_id, &name)?;
        let content = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
        fs::write(clients_dir.join(format!("{name}.json")), content)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn write_prompt_files(
    set_id: &str,
    prompt: &crate::studio::types::DraftPromptSet,
) -> Result<(), String> {
    let root = prompts_publish_root().join(set_id);
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;

    let identity = json!({
        "meta": { "id": prompt.id, "name": prompt.name, "version": prompt.version },
        "identity": prompt.identity,
    });
    let router = json!({
        "meta": { "id": prompt.id, "name": prompt.name, "version": prompt.version },
        "template": {
            "system": prompt.router.system,
            "developer": prompt.router.developer,
            "assembly": prompt.router.assembly,
        }
    });
    let response = json!({
        "meta": { "id": prompt.id, "name": prompt.name, "version": prompt.version },
        "template": {
            "system": prompt.response.system,
            "developer": prompt.response.developer,
            "assembly": prompt.response.assembly,
        }
    });

    fs::write(
        root.join("identity.yaml"),
        serde_yaml::to_string(&identity).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        root.join("router.yaml"),
        serde_yaml::to_string(&router).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    fs::write(
        root.join("response.yaml"),
        serde_yaml::to_string(&response).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn snapshot_version(
    store: &StudioStore,
    project_id: &str,
    version: u32,
    config: &ApiConfig,
    company: &crate::studio::types::DraftCompany,
) -> Result<(), String> {
    let version_dir = store.versions_dir(project_id).join(format!("v{version}"));
    fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;
    let mut files = Vec::new();

    let registry = config.data_root.join("companies/registry.yaml");
    if registry.exists() {
        let dest = version_dir.join("registry.yaml");
        copy_file(&registry, &dest)?;
        files.push("registry.yaml".into());
    }
    for flow_id in &company.flows {
        let src = config.flows_dir.join(format!("{flow_id}.yaml"));
        if src.exists() {
            let name = format!("{flow_id}.yaml");
            copy_file(&src, &version_dir.join(&name))?;
            files.push(name);
        }
    }
    let knowledge = StudioStore::knowledge_source_path(config, &company.knowledge_source);
    if knowledge.exists() {
        copy_file(&knowledge, &version_dir.join("knowledge.yaml"))?;
        files.push("knowledge.yaml".into());
    }

    let manifest = VersionManifest {
        version,
        published_at: Utc::now().to_rfc3339(),
        project_id: project_id.to_string(),
        files,
    };
    store.write_version_manifest(&manifest)
}

fn copy_file(src: &Path, dest: &Path) -> Result<(), String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::copy(src, dest).map_err(|e| e.to_string())?;
    Ok(())
}

async fn smoke_search_draft(
    state: &AppState,
    project_id: &str,
    knowledge: &crate::studio::types::DraftKnowledge,
) -> bool {
    if knowledge.documents.is_empty() {
        return true;
    }
    let store = &state.studio;
    let yaml = knowledge_to_yaml(knowledge);
    let temp_path = store
        .draft_dir(project_id)
        .join("_publish_search_smoke.yaml");
    if fs::write(&temp_path, &yaml).is_err() {
        return false;
    }
    let temp_str = temp_path.to_string_lossy().to_string();
    let flow_id = state
        .flow_registry
        .list()
        .first()
        .map(|f| f.id.clone())
        .unwrap_or_else(|| "payment_flow".into());
    let client_path = state
        .config
        .resolve_data_path("clients/sample.json")
        .to_string_lossy()
        .to_string();

    let result = with_runtime(state, move |rt| {
        let session_id = rt.create_session(&flow_id, &client_path)?;
        rt.load_session_module(&session_id, "company", temp_str.as_str())?;
        rt.build_session_index(&session_id)?;
        let _ = rt.search_session(&session_id, "оплата", None, 3)?;
        rt.destroy_session(&session_id)?;
        Ok(())
    })
    .await;

    let _ = fs::remove_file(&temp_path);
    result.is_ok()
}
