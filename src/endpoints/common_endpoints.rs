use crate::controllers::collectionner_controller::{
    get_list_of_files_and_folders_controller, get_list_of_folders_controller,
};
use crate::controllers::common_controller::*;
use crate::routes_manager::AppState;
use axum::routing::get;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn common_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/dirname", get(get_dirname))
        .route("/CosmicDataLoc", get(get_dirname))
        .route("/lang/{lang}", post(get_lang))
        .route("/null", get(get_null))
        .route("/getThemes", get(get_themes))
        .route("/getStatus/{token}/{type}", get(get_status))
        .route(
            "/getListOfFilesAndFolders/{path}",
            get(get_list_of_files_and_folders_controller),
        )
        .route(
            "/getListOfFolder/{path}",
            get(get_list_of_folders_controller),
        )
        .route("/download/{path}", get(download_file))
        .route("/BM/getBM", get(get_bookmarks))
        .with_state(state)
}
