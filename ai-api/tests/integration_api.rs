use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use ai_api::routes;
use ai_api::state::AppState;

async fn test_app() -> axum::Router {
    let state = AppState::build_test().await.expect("test state");
    routes::create_router(state)
}

async fn json_body(response: axum::response::Response) -> serde_json::Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("json body")
}

#[tokio::test]
async fn health_returns_ok() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn create_session_success() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["session_id"].as_str().is_some());
    assert_eq!(body["current_node"], "greeting");
    assert!(body["opening_message"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
}

#[tokio::test]
async fn create_session_unknown_flow_returns_404() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "missing_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body = json_body(response).await;
    assert_eq!(body["code"], "FLOW_NOT_FOUND");
}

#[tokio::test]
async fn send_message_and_get_history() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let session_id = json_body(create).await["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let message = app
        .clone()
        .oneshot(
            Request::post(format!("/sessions/{session_id}/messages"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "message": "да, это я" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(message.status(), StatusCode::OK);
    let message_body = json_body(message).await;
    assert!(!message_body["assistant_message"]
        .as_str()
        .unwrap_or("")
        .is_empty());

    let history = app
        .clone()
        .oneshot(
            Request::get(format!("/sessions/{session_id}/history"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(history.status(), StatusCode::OK);
    let history_body = json_body(history).await;
    assert!(history_body["entries"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn delete_session_then_get_returns_404() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let session_id = json_body(create).await["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let delete = app
        .clone()
        .oneshot(
            Request::delete(format!("/sessions/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    let get = app
        .oneshot(
            Request::get(format!("/sessions/{session_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_and_reload_prompt_sets() {
    let app = test_app().await;

    let list = app
        .clone()
        .oneshot(Request::get("/prompt-sets").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    assert_eq!(list_body["default_set"], "demo");
    assert!(list_body["sets"].as_array().unwrap().len() >= 1);

    let reload = app
        .oneshot(
            Request::post("/prompt-sets/reload")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(reload.status(), StatusCode::OK);
    let reload_body = json_body(reload).await;
    assert_eq!(reload_body["status"], "reloaded");
}

#[tokio::test]
async fn create_session_with_demo_prompt_set() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json",
                        "prompt_set": "demo"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["prompt_set"], "demo");
}

#[tokio::test]
async fn list_flows_and_clients() {
    let app = test_app().await;

    let flows = app
        .clone()
        .oneshot(Request::get("/flows").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(flows.status(), StatusCode::OK);
    let flows_body = json_body(flows).await;
    assert!(flows_body["flows"].as_array().unwrap().len() >= 1);

    let clients = app
        .oneshot(Request::get("/clients").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(clients.status(), StatusCode::OK);
    let clients_body = json_body(clients).await;
    assert!(clients_body["clients"].as_array().unwrap().len() >= 1);
}

#[tokio::test]
async fn inspect_session_and_flow() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let session_id = json_body(create).await["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let inspect = app
        .clone()
        .oneshot(
            Request::get(format!("/sessions/{session_id}/inspect"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(inspect.status(), StatusCode::OK);
    let inspect_body = json_body(inspect).await;
    assert_eq!(inspect_body["session_id"], session_id);
    assert_eq!(inspect_body["current_node"], "greeting");

    let flow = app
        .oneshot(
            Request::get(format!("/sessions/{session_id}/flow"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(flow.status(), StatusCode::OK);
    let flow_body = json_body(flow).await;
    assert_eq!(flow_body["current_node"], "greeting");
    assert!(flow_body["available_transitions"].as_array().is_some());
}

#[tokio::test]
async fn send_message_with_debug_trace() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let session_id = json_body(create).await["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let message = app
        .oneshot(
            Request::post(format!("/sessions/{session_id}/messages?debug=true"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({ "message": "да, это я" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(message.status(), StatusCode::OK);
    let body = json_body(message).await;
    assert!(body["debug"].is_object());
    assert!(body["debug"]["runtime"]["timeline"].as_array().is_some());
    assert!(body["debug"]["json"]["router_raw_json"].as_str().is_some());
}

#[tokio::test]
async fn get_prompt_set_detail() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/prompt-sets/demo").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert_eq!(body["id"], "demo");
    assert!(body["router"]["system"].as_str().is_some());
    assert!(body["identity"]["name"].as_str().is_some());
}

#[tokio::test]
async fn openapi_json_is_available() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/openapi.json").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn overview_returns_counts() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/overview").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["api_version"].as_str().is_some());
    assert!(body["counts"]["flows"].as_u64().unwrap_or(0) >= 1);
}

#[tokio::test]
async fn list_companies_and_detail() {
    let app = test_app().await;

    let list = app
        .clone()
        .oneshot(Request::get("/companies").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    let companies = list_body["companies"].as_array().unwrap();
    assert!(!companies.is_empty());

    let company_id = companies[0]["id"].as_str().unwrap();
    let detail = app
        .oneshot(
            Request::get(format!("/companies/{company_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail_body = json_body(detail).await;
    assert_eq!(detail_body["id"], company_id);
    assert!(detail_body["flows"].as_array().is_some());
}

#[tokio::test]
async fn flow_detail_and_graph() {
    let app = test_app().await;

    let detail = app
        .clone()
        .oneshot(
            Request::get("/flows/payment_flow")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(detail.status(), StatusCode::OK);
    let detail_body = json_body(detail).await;
    assert_eq!(detail_body["id"], "payment_flow");
    assert!(detail_body["nodes"].as_array().unwrap().len() >= 1);

    let graph = app
        .oneshot(
            Request::get("/flows/payment_flow/graph")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(graph.status(), StatusCode::OK);
    let graph_body = json_body(graph).await;
    assert_eq!(graph_body["flow_id"], "payment_flow");
    assert!(graph_body["edges"].as_array().is_some());
}

#[tokio::test]
async fn list_sessions_includes_created_session() {
    let app = test_app().await;

    let create = app
        .clone()
        .oneshot(
            Request::post("/sessions")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "flow_id": "payment_flow",
                        "client_source_path": "clients/sample.json"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    let session_id = json_body(create).await["session_id"]
        .as_str()
        .unwrap()
        .to_string();

    let list = app
        .oneshot(Request::get("/sessions").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(list.status(), StatusCode::OK);
    let list_body = json_body(list).await;
    let ids: Vec<_> = list_body["sessions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["session_id"].as_str())
        .collect();
    assert!(ids.contains(&session_id.as_str()));
}

#[tokio::test]
async fn search_inspect_returns_results() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::post("/search/inspect")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "query": "оплата",
                        "top_k": 3
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    assert!(body["results"].as_array().is_some());
}

#[tokio::test]
async fn platform_status_and_config() {
    let app = test_app().await;

    let status = app
        .clone()
        .oneshot(Request::get("/platform/status").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(status.status(), StatusCode::OK);
    assert!(json_body(status).await["components"].as_array().unwrap().len() >= 1);

    let config = app
        .oneshot(Request::get("/platform/config").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(config.status(), StatusCode::OK);
    assert!(json_body(config).await["data_root"].as_str().is_some());
}

#[tokio::test]
async fn eval_sync_run_mock_scenario() {
    let app = test_app().await;

    let scenarios = app
        .clone()
        .oneshot(Request::get("/evals/scenarios").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(scenarios.status(), StatusCode::OK);
    let scenarios_body = json_body(scenarios).await;
    let name = scenarios_body["scenarios"]
        .as_array()
        .and_then(|a| a.first())
        .and_then(|s| s["name"].as_str())
        .unwrap_or("greeting_confirm")
        .to_string();

    let run = app
        .oneshot(
            Request::post("/evals/runs")
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "scenario": name,
                        "llm": "mock"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(run.status(), StatusCode::OK);
    let run_body = json_body(run).await;
    assert_eq!(run_body["status"], "completed");
    assert!(run_body["report"]["metrics"].is_object());
}
