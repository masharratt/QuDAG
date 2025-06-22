//! QuDAG Exchange HTTP API Server

use axum::{
    routing::{get, post},
    Router,
    Json,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::info;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct BalanceResponse {
    account_id: String,
    balance: u64,
}

#[derive(Deserialize)]
struct TransferRequest {
    from: String,
    to: String,
    amount: u64,
}

#[derive(Serialize)]
struct TransferResponse {
    transaction_id: String,
    status: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: qudag_exchange_core::version().to_string(),
    })
}

async fn get_balance(account_id: String) -> Json<BalanceResponse> {
    // TODO: Implement actual balance query
    Json(BalanceResponse {
        account_id,
        balance: 1000,
    })
}

async fn transfer(Json(req): Json<TransferRequest>) -> Result<Json<TransferResponse>, StatusCode> {
    // TODO: Implement actual transfer
    Ok(Json(TransferResponse {
        transaction_id: uuid::Uuid::new_v4().to_string(),
        status: "pending".to_string(),
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Build router
    let app = Router::new()
        .route("/health", get(health))
        .route("/balance/:account_id", get(get_balance))
        .route("/transfer", post(transfer))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("QuDAG Exchange server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}