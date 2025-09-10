//! HTTP server for ZisK-Zcash validator RPC

use crate::rpc::handlers::RpcHandler;
use crate::rpc::types::*;
use crate::bridge::node_bridge::NodeConfig;
use crate::error::*;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// HTTP server for ZisK-Zcash validator
pub struct HttpServer {
    handler: Arc<RpcHandler>,
}

impl HttpServer {
    /// Create a new HTTP server
    pub fn new(node_config: NodeConfig) -> ZcashResult<Self> {
        let handler = RpcHandler::new(node_config)?;
        Ok(Self {
            handler: Arc::new(handler),
        })
    }

    /// Create the HTTP router
    pub fn create_router(&self) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/validate_transaction", post(validate_transaction))
            .route("/validate_batch", post(validate_batch))
            .route("/generate_proof", post(generate_proof))
            .route("/verify_proof", post(verify_proof))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive())
            )
            .with_state(self.handler.clone())
    }

    /// Start the HTTP server
    pub async fn start(&self, addr: &str) -> ZcashResult<()> {
        let app = self.create_router();
        let listener = tokio::net::TcpListener::bind(addr).await
            .map_err(|e| ZcashValidationError::SystemError(SystemError::NetworkError(format!("Failed to bind to {}: {:?}", addr, e))))?;
        
        println!("🚀 ZisK-Zcash Validator HTTP server starting on {}", addr);
        println!("📡 Available endpoints:");
        println!("  GET  /health - Health check");
        println!("  POST /validate_transaction - Validate single transaction");
        println!("  POST /validate_batch - Validate transaction batch");
        println!("  POST /generate_proof - Generate STARK proof");
        println!("  POST /verify_proof - Verify STARK proof");
        
        axum::serve(listener, app).await
            .map_err(|e| ZcashValidationError::SystemError(SystemError::NetworkError(format!("Server error: {:?}", e))))?;
        
        Ok(())
    }
}

/// Health check endpoint
async fn health_check(State(handler): State<Arc<RpcHandler>>) -> Result<Json<HealthCheckResponse>, StatusCode> {
    let response = handler.health_check().await;
    Ok(Json(response))
}

/// Validate single transaction endpoint
async fn validate_transaction(
    State(handler): State<Arc<RpcHandler>>,
    Json(request): Json<ValidateTransactionRequest>,
) -> Result<Json<ValidateTransactionResponse>, StatusCode> {
    let response = handler.validate_transaction(request).await;
    Ok(Json(response))
}

/// Validate batch endpoint
async fn validate_batch(
    State(handler): State<Arc<RpcHandler>>,
    Json(request): Json<ValidateBatchRequest>,
) -> Result<Json<ValidateBatchResponse>, StatusCode> {
    let response = handler.validate_batch(request).await;
    Ok(Json(response))
}

/// Generate proof endpoint
async fn generate_proof(
    State(handler): State<Arc<RpcHandler>>,
    Json(request): Json<GenerateProofRequest>,
) -> Result<Json<GenerateProofResponse>, StatusCode> {
    let response = handler.generate_proof(request).await;
    Ok(Json(response))
}

/// Verify proof endpoint
async fn verify_proof(
    State(handler): State<Arc<RpcHandler>>,
    Json(request): Json<VerifyProofRequest>,
) -> Result<Json<VerifyProofResponse>, StatusCode> {
    let response = handler.verify_proof(request).await;
    Ok(Json(response))
}

/// Error handler for JSON parsing errors
pub async fn handle_json_error(err: axum::extract::rejection::JsonRejection) -> (StatusCode, Json<serde_json::Value>) {
    let error_response = ErrorResponse {
        code: "INVALID_JSON".to_string(),
        message: format!("Invalid JSON: {}", err),
        details: None,
    };
    
    (StatusCode::BAD_REQUEST, Json(serde_json::to_value(error_response).unwrap()))
}

/// Error handler for general errors
pub async fn handle_error(err: ZcashValidationError) -> (StatusCode, Json<serde_json::Value>) {
    let error_response = ErrorResponse {
        code: "VALIDATION_ERROR".to_string(),
        message: format!("Validation error: {}", err),
        details: None,
    };
    
    (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::to_value(error_response).unwrap()))
}
