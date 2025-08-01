use crate::controllers::api_controller::*;
use crate::routes_manager::AppState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn api_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/api/marvel/searchonly/{name}", get(marvel_search_only))
        .route(
            "/api/marvel/searchonly/{name}/{date}",
            get(marvel_search_only),
        )
        .route(
            "/api/marvel/getComics/{name}/{date}",
            get(marvel_get_comics),
        )
        .route("/api/ol/getComics/{name}", get(openlibrary_search))
        .route("/api/googlebooks/getComics/{name}", get(googlebooks_search))
        .route("/api/marvel", post(marvel_add))
        .route("/api/anilist", post(anilist_add))
        .route("/api/anilist/searchOnly/{name}", get(anilist_search))
        .with_state(state)
}
