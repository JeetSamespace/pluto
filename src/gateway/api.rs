use axum::{
    routing::{get, post},
    Router, Json, extract::State,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppState {
    routes: Arc<RwLock<HashMap<String, (String, u16)>>>,
}

#[derive(Deserialize, Serialize)]
struct Route {
    path: String,
    ip: String,
    port: u16,
}

async fn add_route(
    State(state): State<AppState>,
    Json(route): Json<Route>,
) -> Json<HashMap<String, (String, u16)>> {
    let mut routes = state.routes.write().await;
    routes.insert(route.path, (route.ip, route.port));
    Json(routes.clone())
}

async fn get_routes(
    State(state): State<AppState>,
) -> Json<HashMap<String, (String, u16)>> {
    let routes = state.routes.read().await;
    Json(routes.clone())
}

pub async fn run_api_server(
    routes: Arc<RwLock<HashMap<String, (String, u16)>>>,
    ip: &String,
    port: &u16,
) {
    let app_state = AppState { routes };

    let app = Router::new()
        .route("/addRoute", post(add_route))
        .route("/routes", get(get_routes))
        .with_state(app_state);

    let addr: SocketAddr = format!("{}:{}", ip, port)
        .parse()
        .expect("Failed to parse socket address");
        
    info!("API server listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
