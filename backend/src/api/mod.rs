use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::ModelInput;
use crate::gateway::Gateway;
use crate::verification::{exports, verifier};

pub type AppState = Arc<Gateway>;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

pub async fn post_decision(
    State(gw): State<AppState>,
    Json(input): Json<ModelInput>,
) -> impl IntoResponse {
    match gw.execute(input) {
        Ok(result) => (
            StatusCode::OK,
            Json(json!({
                "decision_id": result.decision_id,
                "receipt_id": result.receipt_id,
                "output": result.output,
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_receipt(
    State(gw): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match gw.store().get_receipt(&id) {
        Some(receipt) => (StatusCode::OK, Json(serde_json::to_value(receipt).unwrap())).into_response(),
        None => not_found("receipt"),
    }
}

pub async fn get_verify(
    State(gw): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match gw.store().get_receipt(&id) {
        Some(receipt) => {
            let result = verifier::verify(&receipt, gw.keypair());
            let package = exports::build(receipt, result);
            (StatusCode::OK, Json(serde_json::to_value(package).unwrap())).into_response()
        }
        None => not_found("receipt"),
    }
}

fn not_found(resource: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("{} not found", resource) })),
    )
        .into_response()
}
