use axum::Json;
use axum::extract::State;

use crate::dto::{
    GeminiStatusDto, OverviewCountsDto, OverviewResponse, PlatformComponentStatus,
    PlatformConfigResponse, PlatformStatusResponse,
};
use crate::errors::ApiError;
use crate::state::{with_runtime_read, AppState};

const RUNTIME_VERSION: &str = env!("CARGO_PKG_VERSION");

fn gemini_status() -> GeminiStatusDto {
    let configured = std::env::var("GEMINI_API_KEY")
        .map(|k| !k.trim().is_empty())
        .unwrap_or(false);
    GeminiStatusDto {
        enabled: cfg!(feature = "gemini"),
        configured,
        model_router: std::env::var("GEMINI_MODEL_ROUTER")
            .unwrap_or_else(|_| "gemini-2.5-flash".into()),
        model_response: std::env::var("GEMINI_MODEL_RESPONSE")
            .unwrap_or_else(|_| "gemini-2.5-flash".into()),
    }
}

#[utoipa::path(
    get,
    path = "/overview",
    responses((status = 200, description = "Platform overview", body = OverviewResponse)),
    tag = "platform"
)]
pub async fn overview(State(state): State<AppState>) -> Result<Json<OverviewResponse>, ApiError> {
    let flow_count = state.flow_registry.list().len();
    let gemini = gemini_status();

    let (active_sessions, prompt_sets, modules, search_docs) = with_runtime_read(&state, |rt| {
        let registry_ref = rt.prompt_registry();
        let sets = registry_ref.read().map_err(|e| {
            ai_flow_runtime::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            }
        })?;
        let prompt_sets = sets.list_sets().len();
        let modules = rt.list_modules().len();
        let sessions = rt.list_sessions()?;
        let active_sessions = sessions.len();
        let mut search_docs = 0usize;
        for s in &sessions {
            if let Ok(n) = rt.session_search_index_len(&s.session_id) {
                search_docs += n;
            }
        }
        Ok((active_sessions, prompt_sets, modules, search_docs))
    })
    .await?;

    Ok(Json(OverviewResponse {
        api_version: state.config.version.clone(),
        runtime_version: RUNTIME_VERSION.into(),
        uptime_secs: state.uptime_secs(),
        gemini,
        counts: OverviewCountsDto {
            active_sessions,
            flows: flow_count,
            prompt_sets,
            modules,
            search_index_documents: search_docs,
        },
    }))
}

#[utoipa::path(
    get,
    path = "/platform/status",
    responses((status = 200, description = "Component health", body = PlatformStatusResponse)),
    tag = "platform"
)]
pub async fn platform_status(
    State(state): State<AppState>,
) -> Result<Json<PlatformStatusResponse>, ApiError> {
    let gemini = gemini_status();
    let prompt_ok = with_runtime_read(&state, |rt| {
        let registry_ref = rt.prompt_registry();
        registry_ref
            .read()
            .map(|r| !r.list_sets().is_empty())
            .map_err(|e| ai_flow_runtime::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            })
    })
    .await?;

    let components = vec![
        PlatformComponentStatus {
            id: "runtime".into(),
            name: "Runtime".into(),
            status: "ok".into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "prompt_registry".into(),
            name: "Prompt Registry".into(),
            status: if prompt_ok { "ok" } else { "degraded" }.into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "flow_registry".into(),
            name: "Flow Registry".into(),
            status: if state.flow_registry.list().is_empty() {
                "degraded"
            } else {
                "ok"
            }
            .into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "gemini".into(),
            name: "Gemini".into(),
            status: if gemini.enabled && gemini.configured {
                "ok"
            } else if gemini.enabled {
                "not_configured"
            } else {
                "mock"
            }
            .into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "http_api".into(),
            name: "HTTP API".into(),
            status: "ok".into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "search".into(),
            name: "Search".into(),
            status: "ok".into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "modules".into(),
            name: "Modules".into(),
            status: "ok".into(),
            detail: None,
        },
        PlatformComponentStatus {
            id: "evaluation".into(),
            name: "Evaluation".into(),
            status: "ok".into(),
            detail: None,
        },
    ];

    Ok(Json(PlatformStatusResponse { components }))
}

#[utoipa::path(
    get,
    path = "/platform/config",
    responses((status = 200, description = "Platform configuration", body = PlatformConfigResponse)),
    tag = "platform"
)]
pub async fn platform_config(
    State(state): State<AppState>,
) -> Result<Json<PlatformConfigResponse>, ApiError> {
    let (default_prompt_set, loaded_prompt_sets) = with_runtime_read(&state, |rt| {
        let registry_ref = rt.prompt_registry();
        let registry = registry_ref.read().map_err(|e| {
            ai_flow_runtime::RuntimeError::Configuration {
                message: format!("prompt registry lock poisoned: {e}"),
            }
        })?;
        Ok((
            registry.default_set_id().to_string(),
            registry
                .list_sets()
                .into_iter()
                .map(|s| s.id)
                .collect::<Vec<_>>(),
        ))
    })
    .await?;

    let gemini = gemini_status();
    let loaded_flows: Vec<String> = state
        .flow_registry
        .list()
        .iter()
        .map(|f| f.id.clone())
        .collect();

    Ok(Json(PlatformConfigResponse {
        api_url_hint: format!("http://{}:{}", state.config.host, state.config.port),
        data_root: state.config.data_root.display().to_string(),
        flows_dir: state.config.flows_dir.display().to_string(),
        request_timeout_secs: state.config.request_timeout.as_secs(),
        max_body_size: state.config.max_body_size,
        default_prompt_set,
        loaded_flows,
        loaded_prompt_sets,
        default_modules: state
            .config
            .default_modules
            .iter()
            .map(|m| format!("{}:{}", m.module_id, m.path))
            .collect(),
        gemini_model_router: gemini.model_router,
        gemini_model_response: gemini.model_response,
        include_router_decision: state.config.include_router_decision,
    }))
}
