use crate::controllers::authentication_controller::*;
use crate::routes_manager::AppState;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn authentication_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/createUser", post(create_user))
        .with_state(state)
}
