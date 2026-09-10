use std::time::Duration;

use axum::middleware;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::Level;

use crate::config::ApiConfig;
use crate::middleware::request_id;

pub fn init_tracing(log_level: &str) {
    let filter = format!("ai_api={log_level},tower_http=info");
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}

pub fn apply_layers(router: axum::Router, config: &ApiConfig) -> axum::Router {
    let timeout = TimeoutLayer::with_status_code(
        axum::http::StatusCode::REQUEST_TIMEOUT,
        config.request_timeout,
    );

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    router
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(timeout)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    let request_id = request_id::request_id_from_headers(request.headers())
                        .unwrap_or_else(|| "pending".to_string());
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        path = %request.uri().path(),
                        request_id = %request_id,
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: Duration,
                     _span: &tracing::Span| {
                        tracing::info!(
                            status = %response.status().as_u16(),
                            latency_ms = latency.as_millis() as u64,
                            "request completed"
                        );
                    },
                ),
        )
        .layer(middleware::from_fn(request_id::set_request_id))
}
