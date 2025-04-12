use crate::controllers::common_controller::*;
use crate::routes_manager::AppState;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn common_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/dirname", post(get_dirname))
        .route("/CosmicDataLoc", post(get_dirname))
        .route("/lang/{lang}", post(get_lang))
        .route("/null", post(get_null))
        .route("/img/getColor/{img}/{token}", post(get_color))
        .route("/img/getPalette/{token}", post(get_palette_color))
        .with_state(state)
}
