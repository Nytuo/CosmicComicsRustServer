use crate::routes_manager::AppState;
use axum::Router;
use std::sync::Arc;

pub fn collectionner_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new().with_state(state)
}
