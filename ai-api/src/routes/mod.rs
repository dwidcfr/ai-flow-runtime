use axum::routing::{get, post, put};
use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::middleware;
use crate::openapi::ApiDoc;
use crate::state::AppState;

pub fn create_router(state: AppState) -> Router {
    let config = state.config.clone();
    let api = Router::new()
        .route("/health", get(handlers::health))
        .route("/overview", get(handlers::overview))
        .route("/platform/status", get(handlers::platform_status))
        .route("/platform/config", get(handlers::platform_config))
        .route("/companies", get(handlers::list_companies))
        .route("/companies/{company_id}", get(handlers::get_company))
        .route("/flows", get(handlers::list_flows))
        .route("/flows/{flow_id}", get(handlers::get_flow_detail))
        .route("/flows/{flow_id}/graph", get(handlers::get_flow_graph))
        .route("/modules", get(handlers::list_modules))
        .route("/knowledge/sources", get(handlers::list_knowledge_sources))
        .route(
            "/knowledge/sources/{name}",
            get(handlers::get_knowledge_source),
        )
        .route("/clients", get(handlers::list_clients))
        .route("/clients/{name}", get(handlers::get_client))
        .route("/flows/reload", post(handlers::reload_flows))
        .route("/studio/projects", get(handlers::list_projects).post(handlers::create_project))
        .route(
            "/studio/projects/{project_id}",
            get(handlers::get_project)
                .patch(handlers::update_project)
                .delete(handlers::delete_project),
        )
        .route(
            "/studio/projects/{project_id}/status",
            get(handlers::get_project_status),
        )
        .route(
            "/studio/projects/{project_id}/assets",
            get(handlers::get_project_assets),
        )
        .route(
            "/studio/projects/{project_id}/company",
            get(handlers::studio::get_company).put(handlers::update_company),
        )
        .route(
            "/studio/projects/{project_id}/knowledge",
            get(handlers::get_knowledge).put(handlers::put_knowledge),
        )
        .route(
            "/studio/projects/{project_id}/knowledge/documents",
            post(handlers::upsert_document),
        )
        .route(
            "/studio/projects/{project_id}/knowledge/documents/{doc_id}",
            put(handlers::upsert_document).delete(handlers::delete_document),
        )
        .route(
            "/studio/projects/{project_id}/knowledge/reindex",
            post(handlers::knowledge_reindex),
        )
        .route(
            "/studio/projects/{project_id}/knowledge/search",
            post(handlers::knowledge_search),
        )
        .route(
            "/studio/projects/{project_id}/flows",
            get(handlers::studio::list_flows).post(handlers::create_flow),
        )
        .route(
            "/studio/projects/{project_id}/flows/validate",
            post(handlers::validate_flow_handler),
        )
        .route(
            "/studio/projects/{project_id}/flows/{flow_id}",
            get(handlers::get_flow)
                .put(handlers::put_flow)
                .delete(handlers::delete_flow),
        )
        .route(
            "/studio/projects/{project_id}/prompts/{set_id}",
            get(handlers::get_prompt).put(handlers::put_prompt),
        )
        .route(
            "/studio/projects/{project_id}/prompts/{set_id}/preview",
            post(handlers::preview_prompt),
        )
        .route(
            "/studio/projects/{project_id}/clients",
            get(handlers::studio::list_clients).post(handlers::studio::upsert_client),
        )
        .route(
            "/studio/projects/{project_id}/clients/{name}",
            get(handlers::studio::get_client)
                .put(handlers::studio::upsert_client_named)
                .delete(handlers::studio::delete_client),
        )
        .route(
            "/studio/projects/{project_id}/publish",
            post(handlers::publish_project),
        )
        .route(
            "/studio/projects/{project_id}/versions",
            get(handlers::list_versions),
        )
        .route(
            "/studio/projects/{project_id}/versions/{version}",
            get(handlers::get_version),
        )
        .route("/sessions", get(handlers::list_sessions).post(handlers::create_session))
        .route(
            "/sessions/{session_id}",
            get(handlers::get_session).delete(handlers::delete_session),
        )
        .route("/sessions/{session_id}/inspect", get(handlers::inspect_session))
        .route("/sessions/{session_id}/flow", get(handlers::inspect_flow))
        .route("/sessions/{session_id}/messages", post(handlers::send_message))
        .route("/sessions/{session_id}/history", get(handlers::get_history))
        .route("/sessions/{session_id}/search", post(handlers::session_search))
        .route("/search/inspect", post(handlers::search_inspect))
        .route("/prompt-sets", get(handlers::list_prompt_sets))
        .route("/prompt-sets/{set_id}", get(handlers::get_prompt_set))
        .route("/prompt-sets/reload", post(handlers::reload_prompts))
        .route("/evals/scenarios", get(handlers::list_eval_scenarios))
        .route("/evals/runs", post(handlers::run_eval_sync))
        .route("/evals/runs/async", post(handlers::run_eval_async))
        .route("/evals/runs/{run_id}", get(handlers::get_eval_run))
        .merge(SwaggerUi::new("/docs").url("/openapi.json", ApiDoc::openapi()));

    let router = Router::new().merge(api).with_state(state);
    middleware::apply_layers(router, &config)
}
