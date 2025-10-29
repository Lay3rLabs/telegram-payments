mod args;
mod error;
mod handlers;
mod state;
use std::net::SocketAddr;

use anyhow::Context;
use axum::{extract::DefaultBodyLimit, routing::post};
use clap::Parser;
use tg_utils::tracing::tracing_init;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

use crate::{args::ServerArgs, handlers::tg_webhook::handle_tg_webhook, state::HttpState};

const MAX_SIZE_MB: u64 = 50;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    if dotenvy::dotenv().is_err() {
        tracing::debug!("Failed to load .env file");
    }

    tracing_init();

    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let args = ServerArgs::parse();

    let router = make_router().await?;

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    tracing::info!("Starting server on http://{addr}");

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to address {addr}"))?;

    axum::serve(listener, router)
        .await
        .context("server encountered an unrecoverable error")?;

    Ok(())
}

// this is called from main and tests
pub async fn make_router() -> anyhow::Result<axum::Router> {
    let state = HttpState::new();

    // public routes
    let router = axum::Router::new()
        .route("/telegram-webhook", post(handle_tg_webhook))
        .with_state(state.clone());

    // apply global body size limit
    let body_limit_bytes = (MAX_SIZE_MB as usize) * 1024 * 1024;
    let router = router.layer(DefaultBodyLimit::max(body_limit_bytes));

    let router = router.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    Ok(router)
}
