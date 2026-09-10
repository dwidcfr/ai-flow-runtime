use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;

use ai_api::routes;
use ai_api::state::AppState;

static TEST_LOCK: Mutex<()> = Mutex::new(());

fn test_lock() -> MutexGuard<'static, ()> {
    TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn studio_data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ai-flow-runtime/data")
}

fn reset_project_draft(project_id: &str) {
    let draft_dir = studio_data_root().join("studio/drafts").join(project_id);
    let _ = fs::remove_dir_all(draft_dir);
}

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

fn json_request(method: &str, path: &str, body: serde_json::Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

async fn bootstrap_project(app: &axum::Router, project_id: &str) -> serde_json::Value {
    let response = app
        .clone()
        .oneshot(
            Request::get(format!("/studio/projects/{project_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    json_body(response).await
}

#[tokio::test]
async fn studio_list_projects() {
    let _guard = test_lock();
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/studio/projects").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = json_body(response).await;
    let projects = body["projects"].as_array().unwrap();
    assert!(!projects.is_empty());
    assert!(projects.iter().any(|p| p["id"] == "insurance_demo"));
}

#[tokio::test]
async fn studio_bootstrap_project_draft() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    let body = bootstrap_project(&app, "insurance_demo").await;
    assert_eq!(body["id"], "insurance_demo");
    assert!(body["bootstrapped_at"].as_str().is_some());
    assert!(body["flows"].as_array().unwrap().iter().any(|f| f == "payment_flow"));
}

#[tokio::test]
async fn studio_company_crud() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let get = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/company")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::OK);

    let update = app
        .clone()
        .oneshot(json_request(
            "PUT",
            "/studio/projects/insurance_demo/company",
            serde_json::json!({
                "id": "insurance_demo",
                "name": "Insurance Studio",
                "description": "Updated via studio test",
                "prompt_set": "demo",
                "knowledge_source": "company/acme.yaml",
                "flows": ["payment_flow"]
            }),
        ))
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    let body = json_body(update).await;
    assert_eq!(body["name"], "Insurance Studio");
}

#[tokio::test]
async fn studio_knowledge_crud_and_search() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let doc = serde_json::json!({
        "id": "faq_test_studio",
        "category_id": "payment",
        "title": "Studio FAQ",
        "section": "faq_test_studio",
        "content": "Оплата картой доступна в мобильном приложении Acme Insurance.",
        "metadata": { "tags": ["test"] }
    });

    let upsert = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/knowledge/documents",
            doc.clone(),
        ))
        .await
        .unwrap();
    assert_eq!(upsert.status(), StatusCode::OK);

    let knowledge = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/knowledge")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = json_body(knowledge).await;
    let docs = body["documents"].as_array().unwrap();
    assert!(docs.iter().any(|d| d["id"] == "faq_test_studio"));

    let search = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/knowledge/search",
            serde_json::json!({ "query": "оплата", "top_k": 3 }),
        ))
        .await
        .unwrap();
    assert_eq!(search.status(), StatusCode::OK);
    let search_body = json_body(search).await;
    assert!(search_body["results"].as_array().is_some());

    let delete = app
        .clone()
        .oneshot(
            Request::delete("/studio/projects/insurance_demo/knowledge/documents/faq_test_studio")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::OK);
}

#[tokio::test]
async fn studio_flow_validate_valid_and_invalid() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let flow = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/flows/payment_flow")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let flow_body = json_body(flow).await;

    let valid = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/flows/validate",
            flow_body.clone(),
        ))
        .await
        .unwrap();
    assert_eq!(valid.status(), StatusCode::OK);
    let valid_body = json_body(valid).await;
    assert_eq!(valid_body["valid"], true);

    let invalid = serde_json::json!({
        "id": "bad_flow",
        "name": "Bad",
        "version": 1,
        "initial": "start",
        "nodes": [
            {
                "id": "start",
                "node_type": "flow",
                "payload": { "task": "No end" },
                "transitions": [{ "action": "go", "target": "missing" }]
            },
            {
                "id": "end",
                "node_type": "end",
                "payload": {},
                "transitions": []
            }
        ]
    });
    let invalid_resp = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/flows/validate",
            invalid,
        ))
        .await
        .unwrap();
    assert_eq!(invalid_resp.status(), StatusCode::OK);
    let invalid_body = json_body(invalid_resp).await;
    assert_eq!(invalid_body["valid"], false);
    assert!(!invalid_body["issues"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn studio_prompt_preview() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let preview = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/prompts/demo/preview",
            serde_json::json!({ "template": "router" }),
        ))
        .await
        .unwrap();
    assert_eq!(preview.status(), StatusCode::OK);
    let body = json_body(preview).await;
    assert!(body["rendered"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
}

#[tokio::test]
async fn studio_publish_happy_path() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let publish = app
        .clone()
        .oneshot(
            Request::post("/studio/projects/insurance_demo/publish")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::OK);
    let body = json_body(publish).await;
    assert_eq!(body["status"], "completed", "publish error: {:?}", body);
    assert!(body["version"].as_u64().unwrap_or(0) >= 1);

    let versions = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/versions")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(versions.status(), StatusCode::OK);
    let versions_body = json_body(versions).await;
    assert!(!versions_body["versions"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn studio_publish_rollback_on_eval_failure() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let flow_path = studio_data_root().join("flows/payment_flow.yaml");
    let original_flow = fs::read_to_string(&flow_path).expect("read published flow");

    let flow_resp = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/flows/payment_flow")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let mut flow = json_body(flow_resp).await;

    flow["nodes"] = serde_json::json!([
        {
            "id": "greeting",
            "node_type": "flow",
            "payload": { "task": "Greeting" },
            "transitions": [
                { "action": "confirm", "target": "end" },
                { "action": "wrong_person", "target": "end" }
            ],
            "position": { "x": 120.0, "y": 80.0 }
        },
        {
            "id": "end",
            "node_type": "end",
            "payload": {},
            "transitions": [],
            "position": { "x": 120.0, "y": 220.0 }
        }
    ]);

    let save = app
        .clone()
        .oneshot(json_request(
            "PUT",
            "/studio/projects/insurance_demo/flows/payment_flow",
            flow,
        ))
        .await
        .unwrap();
    assert_eq!(save.status(), StatusCode::OK);

    let publish = app
        .clone()
        .oneshot(
            Request::post("/studio/projects/insurance_demo/publish")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(publish.status(), StatusCode::OK);
    let body = json_body(publish).await;
    if body["status"] != "failed" {
        eprintln!("rollback publish: {:?}", body);
    }
    assert_eq!(body["status"], "failed", "expected publish rollback: {:?}", body);
    assert!(
        body["stage"] == "evals" || body["stage"] == "reload_flows",
        "unexpected failure stage: {:?}",
        body
    );

    let restored = fs::read_to_string(&flow_path).expect("read restored flow");
    assert!(restored.contains("confirm: payment"));
    assert!(!restored.contains("confirm: end"));

    let _ = fs::write(&flow_path, original_flow);
}

#[tokio::test]
async fn studio_clients_crud() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let upsert = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/clients",
            serde_json::json!({
                "name": "test_client",
                "data": { "name": "Test", "phone": "+123" }
            }),
        ))
        .await
        .unwrap();
    assert_eq!(upsert.status(), StatusCode::OK);

    let list = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/clients")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let body = json_body(list).await;
    assert!(body["clients"]
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c == "test_client"));

    let delete = app
        .clone()
        .oneshot(
            Request::delete("/studio/projects/insurance_demo/clients/test_client")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn studio_update_project_and_status() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let update = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            "/studio/projects/insurance_demo",
            serde_json::json!({ "name": "Insurance Demo Renamed", "description": "Test desc" }),
        ))
        .await
        .unwrap();
    assert_eq!(update.status(), StatusCode::OK);
    let body = json_body(update).await;
    assert_eq!(body["name"], "Insurance Demo Renamed");

    let status = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(status.status(), StatusCode::OK);
    let status_body = json_body(status).await;
    assert_eq!(status_body["draft_exists"], true);
}

#[tokio::test]
async fn studio_project_assets_and_delete_flow() {
    let _guard = test_lock();
    reset_project_draft("insurance_demo");
    let app = test_app().await;
    bootstrap_project(&app, "insurance_demo").await;

    let assets = app
        .clone()
        .oneshot(
            Request::get("/studio/projects/insurance_demo/assets")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(assets.status(), StatusCode::OK);
    let body = json_body(assets).await;
    assert!(body["flows"].as_array().unwrap().iter().any(|f| f == "payment_flow"));

    let create = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects/insurance_demo/flows",
            serde_json::json!({ "id": "temp_flow", "name": "Temp" }),
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    let delete = app
        .clone()
        .oneshot(
            Request::delete("/studio/projects/insurance_demo/flows/temp_flow")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn studio_create_and_delete_project() {
    let _guard = test_lock();
    let app = test_app().await;
    let id = "test_proj_temp_delete";

    let _ = app
        .clone()
        .oneshot(
            Request::delete(format!("/studio/projects/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

    let create = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/studio/projects",
            serde_json::json!({
                "id": id,
                "name": "Test Project",
                "description": "temporary"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::CREATED);

    let delete = app
        .clone()
        .oneshot(
            Request::delete(format!("/studio/projects/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    let list = app
        .clone()
        .oneshot(Request::get("/studio/projects").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let body = json_body(list).await;
    assert!(!body["projects"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["id"] == id));
}
