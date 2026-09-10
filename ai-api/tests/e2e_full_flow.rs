use std::time::Duration;

use ai_api::routes;
use ai_api::state::AppState;
use reqwest::StatusCode;
use tokio::net::TcpListener;

#[tokio::test]
async fn full_http_flow() {
    let state = AppState::build_test().await.expect("test state");
    let app = routes::create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("local addr");

    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("client");
    let base = format!("http://{addr}");

    let health = client
        .get(format!("{base}/health"))
        .send()
        .await
        .expect("health");
    assert_eq!(health.status(), StatusCode::OK);

    let session = client
        .post(format!("{base}/sessions"))
        .json(&serde_json::json!({
            "flow_id": "payment_flow",
            "client_source_path": "clients/sample.json"
        }))
        .send()
        .await
        .expect("create session");
    assert_eq!(session.status(), StatusCode::OK);
    let session_body: serde_json::Value = session.json().await.expect("session json");
    let session_id = session_body["session_id"]
        .as_str()
        .expect("session_id")
        .to_string();

    let message = client
        .post(format!("{base}/sessions/{session_id}/messages"))
        .json(&serde_json::json!({ "message": "да, это я" }))
        .send()
        .await
        .expect("send message");
    assert_eq!(message.status(), StatusCode::OK);
    let message_body: serde_json::Value = message.json().await.expect("message json");
    assert!(!message_body["assistant_message"]
        .as_str()
        .unwrap_or("")
        .is_empty());

    let history = client
        .get(format!("{base}/sessions/{session_id}/history"))
        .send()
        .await
        .expect("history");
    assert_eq!(history.status(), StatusCode::OK);
    let history_body: serde_json::Value = history.json().await.expect("history json");
    assert!(history_body["entries"].as_array().unwrap().len() >= 2);

    let delete = client
        .delete(format!("{base}/sessions/{session_id}"))
        .send()
        .await
        .expect("delete");
    assert_eq!(delete.status(), StatusCode::NO_CONTENT);

    let gone = client
        .get(format!("{base}/sessions/{session_id}"))
        .send()
        .await
        .expect("get deleted");
    assert_eq!(gone.status(), StatusCode::NOT_FOUND);
}
