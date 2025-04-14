use crate::routes_manager::AppState;
use axum::Router;
use std::sync::Arc;
use axum::routing::{get, post};
use crate::controllers::settings_controller::*;

pub fn settings_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new().route("/configServ/{name}/{passcode}",post(create_first_user))
        .route("/getVersion", get(get_version))
        .route("/config/writeConfig/{token}", post(write_config))
        .with_state(state)
}
