use crate::controllers::collectionner_controller::{
    fill_blank_images_controller, first_images_of_all_image_getter, insert_anilist_book,
    insert_googlebooks_book, insert_marvel_book, insert_olib_book, refresh_meta_controller,
    scrape_images_from_webpage_controller,
};
use crate::routes_manager::AppState;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;

pub fn collectionner_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/fillBlankImage", post(fill_blank_images_controller))
        .route("/insert/anilist/book", post(insert_anilist_book))
        .route("/insert/marvel/book", get(insert_marvel_book))
        .route("/insert/googlebooks/book", get(insert_googlebooks_book))
        .route("/insert/ol/book", get(insert_olib_book))
        .route("/refreshMeta", post(refresh_meta_controller))
        .route("/downloadBook", post(scrape_images_from_webpage_controller))
        .route(
            "/FirstImagesOfAll/{image_name}",
            get(first_images_of_all_image_getter),
        )
        .with_state(state)
}
