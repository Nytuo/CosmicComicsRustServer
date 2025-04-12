use crate::controllers::database_controller::*;
use crate::routes_manager::AppState;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn database_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/DB/insert/{tokem}/{dbName}", post(insert_db))
        .with_state(state)
}
