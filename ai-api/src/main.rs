use ai_api::{run, ApiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();

    let config = ApiConfig::from_env().map_err(|e| format!("config error: {e}"))?;
    ai_api::middleware::logging::init_tracing(&config.log_level);

    tracing::info!(
        host = %config.host,
        port = config.port,
        "starting ai-api server"
    );

    run(config).await?;
    Ok(())
}
