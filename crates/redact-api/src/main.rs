use redact_api::server::{ApiServer, ServerConfig};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "redact_api=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse configuration from environment
    let config = ServerConfig {
        host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        port: std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080),
        enable_tracing: std::env::var("ENABLE_TRACING")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(true),
    };

    // Create and run server
    let server = ApiServer::new(config);
    server.run().await?;

    Ok(())
}
