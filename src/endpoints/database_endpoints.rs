use crate::controllers::database_controller::*;
use crate::routes_manager::AppState;
use axum::routing::get;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn database_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/DB/insert/{tokem}/{dbName}", post(insert_db))
        .route("/DB/write/{json_file}", post(write_db))
        .route("/DB/read/{json_file}", get(read_db))
        .route(
            "/DB/update/{token}/{dbName}/{colName}/{value}/{id}",
            get(update_db),
        )
        .route("/DB/update/OneForAll", post(update_db_one_for_all))
        .route("/DB/update", post(update_db_body))
        .route("/DB/lib/update/{token}/{id}", post(update_lib))
        .route("/DB/delete/{token}/{dbName}/{id}/{option}", get(delete_db))
        .route(
            "/DB/delete/truedelete/{token}/{dbName}/{id}",
            get(true_delete_db),
        )
        .route("/DB/lib/delete/{token}/{id}", get(delete_lib))
        .route("/DB/get/{token}/{db_name}", post(get_db))
        .with_state(state)
}
