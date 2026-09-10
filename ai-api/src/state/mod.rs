use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use ai_flow_runtime::Runtime;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::companies::{CompaniesError, CompaniesRegistry};
use crate::config::ApiConfig;
use crate::errors::{ApiError, ApiErrorCode};
use crate::eval_store::EvalRunStore;
use crate::studio::StudioStore;

#[derive(Debug, Clone)]
pub struct FlowInfo {
    pub id: String,
    pub name: Option<String>,
    pub version: Option<String>,
    pub source_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FlowRegistry {
    flows: Vec<FlowInfo>,
}

impl FlowRegistry {
    pub fn contains(&self, flow_id: &str) -> bool {
        self.flows.iter().any(|f| f.id == flow_id)
    }

    pub fn list(&self) -> &[FlowInfo] {
        &self.flows
    }

    pub fn replace(&mut self, flows: Vec<FlowInfo>) {
        self.flows = flows;
    }
}

#[derive(Clone)]
pub struct AppState {
    pub runtime: Arc<Mutex<Runtime>>,
    pub flow_registry: FlowRegistry,
    pub companies_registry: CompaniesRegistry,
    pub eval_runs: EvalRunStore,
    pub studio: StudioStore,
    pub config: Arc<ApiConfig>,
    pub started_at: Instant,
}

impl AppState {
    pub async fn build(config: ApiConfig) -> Result<Self, ApiError> {
        let config = Arc::new(config);
        let (runtime, flow_registry, companies_registry) =
            tokio::task::spawn_blocking({
                let config = Arc::clone(&config);
                move || build_runtime_and_flows(&config)
            })
            .await
            .map_err(|e| ApiError::internal(format!("startup task failed: {e}")))??;

        Ok(Self {
            runtime: Arc::new(Mutex::new(runtime)),
            flow_registry,
            companies_registry,
            eval_runs: EvalRunStore::new(),
            studio: StudioStore::new(&config.data_root),
            config,
            started_at: Instant::now(),
        })
    }

    pub async fn build_test() -> Result<Self, ApiError> {
        let data_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../ai-flow-runtime/data");
        Self::build(ApiConfig::for_test(data_root)).await
    }

    pub fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }
}

fn build_runtime_and_flows(
    config: &ApiConfig,
) -> Result<(Runtime, FlowRegistry, CompaniesRegistry), ApiError> {
    #[cfg(feature = "gemini")]
    let mut runtime = {
        use ai_flow_runtime::flow::provider::YamlFlowProvider;
        let _ = dotenvy::dotenv();
        Runtime::with_gemini_from_env(YamlFlowProvider::new()).map_err(|e| {
            ApiError::new(
                ApiErrorCode::InternalError,
                format!("failed to initialize gemini runtime: {e}"),
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?
    };

    #[cfg(not(feature = "gemini"))]
    let mut runtime = Runtime::new();

    let flow_registry = load_flows(&mut runtime, &config.flows_dir)?;
    let companies_registry = CompaniesRegistry::load(&config.data_root).map_err(|e| match e {
        CompaniesError::NotFound(msg) => internal_err(msg),
        CompaniesError::Invalid(msg) => internal_err(msg),
    })?;
    Ok((runtime, flow_registry, companies_registry))
}

fn load_flows(runtime: &mut Runtime, flows_dir: &Path) -> Result<FlowRegistry, ApiError> {
    if !flows_dir.exists() {
        return Err(ApiError::new(
            ApiErrorCode::InternalError,
            format!("flows directory not found: {}", flows_dir.display()),
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(flows_dir)
        .map_err(|e| internal_err(format!("cannot read flows dir: {e}")))?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("yaml"))
        .collect();
    entries.sort();

    if entries.is_empty() {
        return Err(internal_err(format!(
            "no flow yaml files in {}",
            flows_dir.display()
        )));
    }

    let mut flows = Vec::new();
    for path in entries {
        let meta = read_flow_meta(&path)?;
        let flow_id = runtime
            .load_flow(path.to_str().ok_or_else(|| {
                internal_err(format!("invalid flow path: {}", path.display()))
            })?)
            .map_err(ApiError::from)?;

        if flow_id != meta.id {
            return Err(internal_err(format!(
                "flow id mismatch: yaml id={} loaded={}",
                meta.id, flow_id
            )));
        }

        flows.push(FlowInfo {
            id: flow_id,
            name: meta.name,
            version: meta.version.map(|v| v.to_string()),
            source_path: path,
        });
    }

    Ok(FlowRegistry { flows })
}

#[derive(Debug, Deserialize)]
struct FlowMetaYaml {
    id: String,
    name: Option<String>,
    version: Option<serde_json::Value>,
}

fn read_flow_meta(path: &Path) -> Result<FlowMetaYaml, ApiError> {
    let content = fs::read_to_string(path)
        .map_err(|e| internal_err(format!("cannot read {}: {e}", path.display())))?;
    serde_yaml::from_str(&content)
        .map_err(|e| internal_err(format!("invalid flow yaml {}: {e}", path.display())))
}

fn internal_err(message: String) -> ApiError {
    ApiError::new(
        ApiErrorCode::InternalError,
        message,
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    )
}

pub async fn with_runtime<F, T>(state: &AppState, f: F) -> Result<T, ApiError>
where
    F: FnOnce(&mut Runtime) -> Result<T, ai_flow_runtime::RuntimeError> + Send + 'static,
    T: Send + 'static,
{
    let runtime = Arc::clone(&state.runtime);
    tokio::task::spawn_blocking(move || {
        let mut rt = runtime.blocking_lock();
        f(&mut rt).map_err(ApiError::from)
    })
    .await
    .map_err(|e| ApiError::internal(format!("runtime task failed: {e}")))?
}

pub async fn with_runtime_read<F, T>(state: &AppState, f: F) -> Result<T, ApiError>
where
    F: FnOnce(&Runtime) -> Result<T, ai_flow_runtime::RuntimeError> + Send + 'static,
    T: Send + 'static,
{
    let runtime = Arc::clone(&state.runtime);
    tokio::task::spawn_blocking(move || {
        let rt = runtime.blocking_lock();
        f(&rt).map_err(ApiError::from)
    })
    .await
    .map_err(|e| ApiError::internal(format!("runtime task failed: {e}")))?
}
