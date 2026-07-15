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
use uuid::Uuid;

use execution::inference::OnnxSession;

/// Canonical policy artifact for the MVP.
/// Changing any content changes policy_hash, making the change auditable
/// in every subsequent receipt.
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

    let model_path = Path::new("../model/model.onnx");
    let session = OnnxSession::load(model_path, Uuid::new_v4())?;

    let store = store::Store::new();
    let registry = registry::Registry::new();
    let gw = gateway::Gateway::new(session, store, registry, POLICY_ARTIFACT)?;
    let state: api::AppState = Arc::new(gw);

    let app = Router::new()
        .route("/health", get(api::health))
        .route("/decision", post(api::post_decision))
        .route("/receipt/{id}", get(api::get_receipt))
        .route("/verify/{id}", get(api::get_verify))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
