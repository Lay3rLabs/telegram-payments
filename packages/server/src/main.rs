use std::net::SocketAddr;

use anyhow::Context;
use axum::{response::IntoResponse, routing::get, Json, Router};
use serde_json::json;
use tg_utils::tracing::tracing_init;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

// TODO - make this globally configurable
const DEFAULT_PORT: u16 = 3000;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_init();

    // Install rustls crypto provider before any TLS operations
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], DEFAULT_PORT));
    tracing::info!("Starting server on http://{addr}");

    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind to address {addr}"))?;

    axum::serve(listener, app)
        .await
        .context("server encountered an unrecoverable error")?;

    Ok(())
}

async fn root() -> impl IntoResponse {
    Json(json!({
        "message": "telegram-payments CLI server is running"
    }))
}

async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}
