mod app;
mod bonjour;
mod config;
mod driver;
mod ipp;
mod output;

use anyhow::Context;
use axum::routing::{get, post};
use axum::Router;
use config::Config;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = Config::parse();
    let _bonjour = bonjour::Advertiser::start(&config)?;
    let state = Arc::new(app::AppState::new(config.clone()));

    let router = Router::new()
        .route("/health", get(app::health))
        .route("/ipp/print", post(ipp::handler::handle_ipp))
        .with_state(state);

    let listener = TcpListener::bind(config.listen_addr)
        .await
        .with_context(|| format!("failed to bind {}", config.listen_addr))?;

    info!(addr = %config.listen_addr, "starting Vevor printer app");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}
