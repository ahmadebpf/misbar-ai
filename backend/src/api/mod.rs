use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::domain::{ModelInput, ReceiptRecord};
use crate::gateway::Gateway;
use crate::verification::exports;

pub type AppState = Arc<Gateway>;

pub async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

pub async fn post_decision(
    State(gw): State<AppState>,
    Json(input): Json<ModelInput>,
) -> impl IntoResponse {
    match gw.execute(input).await {
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
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn get_receipt(
    State(gw): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match gw.store().get_receipt(&id).await {
        Ok(Some(receipt)) => {
            let model = gw.registry().get_model(&receipt.model_id);
            let policy = gw.registry().get_policy(&receipt.policy_id);
            let mut value = serde_json::to_value(&receipt).unwrap();
            if let Some(obj) = value.as_object_mut() {
                obj.insert("model_name".into(), json!(model.as_ref().map(|m| &m.name)));
                obj.insert("model_version".into(), json!(model.as_ref().map(|m| &m.version)));
                obj.insert("policy_name".into(), json!(policy.as_ref().map(|p| &p.name)));
                obj.insert("policy_version".into(), json!(policy.as_ref().map(|p| &p.version)));
            }
            (StatusCode::OK, Json(value)).into_response()
        }
        Ok(None) => not_found("receipt"),
        Err(e) => internal_error(e),
    }
}

pub async fn get_verify(
    State(gw): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match gw.verify_receipt(&id).await {
        Ok(Some((receipt, result))) => {
            let package = exports::build(receipt, result);
            (StatusCode::OK, Json(serde_json::to_value(package).unwrap())).into_response()
        }
        Ok(None) => not_found("receipt"),
        Err(e) => internal_error(e),
    }
}

/// Standalone verification: the caller pastes a full receipt (the same
/// shape `GET /receipt/{id}` returns) and it's re-verified without
/// requiring it to exist in this instance's store — matches docs/product.md's
/// portable "Verification Package" concept.
pub async fn post_verify(
    State(gw): State<AppState>,
    Json(receipt): Json<ReceiptRecord>,
) -> impl IntoResponse {
    let (receipt, result) = gw.verify_external(receipt).await;
    let package = exports::build(receipt, result);
    (StatusCode::OK, Json(serde_json::to_value(package).unwrap())).into_response()
}

pub async fn get_trace(
    State(gw): State<AppState>,
    Path(decision_id): Path<Uuid>,
) -> impl IntoResponse {
    match gw.store().list_zk_traces_for_decision(&decision_id).await {
        Ok(traces) => (
            StatusCode::OK,
            Json(json!({ "decision_id": decision_id, "traces": traces })),
        )
            .into_response(),
        Err(e) => internal_error(e),
    }
}

pub async fn list_traces(
    State(gw): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    match gw.store().list_zk_traces(limit, offset).await {
        Ok(traces) => (
            StatusCode::OK,
            Json(json!({ "traces": traces, "limit": limit, "offset": offset })),
        )
            .into_response(),
        Err(e) => internal_error(e),
    }
}

#[derive(Deserialize)]
pub struct ListParams {
    limit: Option<i64>,
    offset: Option<i64>,
}

pub async fn list_receipts(
    State(gw): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let limit = params.limit.unwrap_or(20).clamp(1, 100);
    let offset = params.offset.unwrap_or(0).max(0);

    let store = gw.store();
    let (receipts, total) = tokio::join!(
        store.list_receipts(limit, offset),
        store.count_receipts(),
    );

    match (receipts, total) {
        (Ok(receipts), Ok(total)) => {
            let receipts: Vec<serde_json::Value> = receipts
                .into_iter()
                .map(|r| {
                    let model = gw.registry().get_model(&r.model_id);
                    let policy = gw.registry().get_policy(&r.policy_id);
                    let mut value = serde_json::to_value(&r).unwrap();
                    if let Some(obj) = value.as_object_mut() {
                        obj.insert("model_name".into(), json!(model.as_ref().map(|m| &m.name)));
                        obj.insert("model_version".into(), json!(model.as_ref().map(|m| &m.version)));
                        obj.insert("policy_name".into(), json!(policy.as_ref().map(|p| &p.name)));
                        obj.insert("policy_version".into(), json!(policy.as_ref().map(|p| &p.version)));
                    }
                    value
                })
                .collect();
            (
                StatusCode::OK,
                Json(json!({ "receipts": receipts, "total": total, "limit": limit, "offset": offset })),
            )
                .into_response()
        }
        (Err(e), _) | (_, Err(e)) => internal_error(e),
    }
}

pub async fn get_stats(State(gw): State<AppState>) -> impl IntoResponse {
    match gw.store().count_receipts().await {
        Ok(total) => {
            let info = gw.info();
            let model = gw.registry().get_model(&info.model_id);
            let policy = gw.registry().get_policy(&info.policy_id);
            (
                StatusCode::OK,
                Json(json!({
                    "total_receipts": total,
                    "model_id": info.model_id,
                    "model_hash": info.model_hash,
                    "model_name": model.as_ref().map(|m| &m.name),
                    "model_version": model.as_ref().map(|m| &m.version),
                    "policy_id": info.policy_id,
                    "policy_hash": info.policy_hash,
                    "policy_name": policy.as_ref().map(|p| &p.name),
                    "policy_version": policy.as_ref().map(|p| &p.version),
                })),
            )
                .into_response()
        }
        Err(e) => internal_error(e),
    }
}

fn not_found(resource: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "error": format!("{} not found", resource) })),
    )
        .into_response()
}

fn internal_error(e: anyhow::Error) -> axum::response::Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({ "error": e.to_string() })),
    )
        .into_response()
}
