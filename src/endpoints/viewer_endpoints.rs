use crate::controllers::viewer_controller::{get_config_controller, isDir, read_image, unzip_controller, upload_comic_controller, view_current_controller, view_current_page_controller, view_exist_controller, view_read_file_controller, viewer_view_controller};
use crate::routes_manager::AppState;
use axum::Router;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn viewer_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/Unzip/{path}/{token}", get(unzip_controller))
        .route("/viewer/view/current/{token}",get(view_current_controller))
        .route("/viewer/view", get(viewer_view_controller))
        .route("/viewer/view/current/{page}/{token}", get(view_current_page_controller))
        .route("/config/getConfig/{token}",get(get_config_controller))
        .route("/view/isDir/{path}",get(isDir))
        .route("/view/exist/{path}",get(view_exist_controller))
        .route("/view/readFile/{path}",get(view_read_file_controller))
        .route("/view/readImage",get(read_image))
        .route("/uploadComic", post(upload_comic_controller))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 1024))
        .with_state(state)
}
