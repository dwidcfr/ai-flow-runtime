use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tokio::signal;

use crate::config::ApiConfig;
use crate::errors::ApiStartupError;
use crate::routes;
use crate::state::AppState;

pub async fn run(config: ApiConfig) -> Result<(), ApiStartupError> {
    let state = AppState::build(config)
        .await
        .map_err(|e| ApiStartupError::Startup(e.message))?;
    let addr = state.config.socket_addr().map_err(ApiStartupError::Config)?;
    let app = routes::create_router(state);

    let listener = TcpListener::bind(addr).await.map_err(|e| {
        ApiStartupError::Startup(format!("failed to bind {addr}: {e}"))
    })?;

    let bound = listener.local_addr().map_err(|e| {
        ApiStartupError::Startup(format!("failed to read local addr: {e}"))
    })?;

    tracing::info!(%bound, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| ApiStartupError::Startup(format!("server error: {e}")))?;

    Ok(())
}

pub async fn serve_test(app: Router, addr: SocketAddr) -> tokio::task::JoinHandle<()> {
    let listener = TcpListener::bind(addr).await.expect("bind test listener");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test");
    })
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
