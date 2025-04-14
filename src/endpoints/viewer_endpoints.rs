use crate::controllers::viewer_controller::upload_comic_controller;
use crate::routes_manager::AppState;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::post;
use std::sync::Arc;

pub fn viewer_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/uploadComic", post(upload_comic_controller))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .with_state(state)
}
