use crate::orbit::config::ApiServerConfig;
use axum::{
    routing::{get, post, put, delete},
    Router,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use hyper::Error;
use tracing::info;

// Define request/response structures
#[derive(Deserialize)]
struct GatewayRequest {
    name: String,
    // Add other fields as needed
}

#[derive(Serialize)]
struct GatewayResponse {
    id: String,
    name: String,
    status: String,
}

/// Creates and returns the API router with all routes configured
pub fn create_router() -> Router {
    Router::new()
        // Gateway routes
        .route("/gateways", get(list_gateways))
        .route("/gateways", post(create_gateway))
        .route("/gateways/:id", get(get_gateway))
        .route("/gateways/:id", put(update_gateway))
        .route("/gateways/:id", delete(delete_gateway))
        // Stats routes
        .route("/stats", get(get_stats))
        .route("/stats/:gateway_id", get(get_gateway_stats))
        // Health check route
        .route("/health", get(health_check))
}

/// Starts the API server using the provided configuration
pub async fn start_server(config: &ApiServerConfig) -> Result<(), Error> {
    let app = create_router();
    
    let addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Failed to parse API server address");
    
    info!("API server listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// Gateway CRUD Handlers
async fn list_gateways() -> impl IntoResponse {
    Json(vec![
        GatewayResponse {
            id: "1".to_string(),
            name: "Gateway 1".to_string(),
            status: "active".to_string(),
        }
    ])
}

async fn create_gateway(Json(payload): Json<GatewayRequest>) -> impl IntoResponse {
    Json(GatewayResponse {
        id: "new-id".to_string(),
        name: payload.name,
        status: "created".to_string(),
    })
}

async fn get_gateway(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    Json(GatewayResponse {
        id,
        name: "Example Gateway".to_string(),
        status: "active".to_string(),
    })
}

async fn update_gateway(
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<GatewayRequest>,
) -> impl IntoResponse {
    Json(GatewayResponse {
        id,
        name: payload.name,
        status: "updated".to_string(),
    })
}

async fn delete_gateway(axum::extract::Path(id): axum::extract::Path<String>) -> impl IntoResponse {
    format!("Gateway {} deleted", id)
}

// Stats Handlers
async fn get_stats() -> impl IntoResponse {
    "Global Stats"
}

async fn get_gateway_stats(axum::extract::Path(gateway_id): axum::extract::Path<String>) -> impl IntoResponse {
    format!("Stats for gateway {}", gateway_id)
}

// Health Check Handler
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}
