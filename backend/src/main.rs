mod api;
mod attestation;
mod domain;
mod execution;
mod gateway;
mod registry;
mod store;
mod verification;

use std::{path::Path, sync::Arc};

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use execution::inference::OnnxSession;

const POLICY_ARTIFACT: &str =
    "approved_threshold=700;review_threshold=580;max_debt_ratio=0.6;max_missed_payments=5;policy_version=v1.0.0";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Overridable so a deployment can mount persistent state (db, signing key)
    // at a custom path instead of the working directory. Defaults match the
    // previous hardcoded values, so local `cargo run` is unaffected.
    let model_path = std::env::var("MISBAR_MODEL_PATH").unwrap_or_else(|_| "../model/model.onnx".into());
    let database_url = std::env::var("MISBAR_DATABASE_URL").unwrap_or_else(|_| "sqlite://misbar.db".into());
    let key_path = std::env::var("MISBAR_KEY_PATH").unwrap_or_else(|_| "misbar.key".into());
    let circuit_dir = std::env::var("MISBAR_CIRCUIT_DIR").unwrap_or_else(|_| "circuit".into());

    let session = OnnxSession::load(Path::new(&model_path))?;

    let store = store::Store::new(&database_url).await?;
    let registry = registry::Registry::new();
    let gw = gateway::Gateway::new(
        session,
        store,
        registry,
        POLICY_ARTIFACT,
        Path::new(&key_path),
        Path::new(&circuit_dir),
    )?;
    let state: api::AppState = Arc::new(gw);

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/decision", post(api::post_decision))
        .route("/receipt/{id}", get(api::get_receipt))
        .route("/verify/{id}", get(api::get_verify))
        .route("/receipts", get(api::list_receipts))
        .route("/stats", get(api::get_stats))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
