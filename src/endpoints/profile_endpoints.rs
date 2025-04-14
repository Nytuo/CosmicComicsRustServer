use crate::controllers::profile_controller::*;
use crate::routes_manager::AppState;
use axum::routing::get;
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn authentication_routes(state: Arc<tokio::sync::Mutex<AppState>>) -> Router {
    Router::new()
        .route("/createUser", post(create_user))
        .route("/profile/getPP/{token}", get(get_profile_picture))
        .route("/profile/getPPBN/{name}", get(get_profile_picture_by_name))
        .route("/profile/custo/getNumber", get(get_custom_number))
        .route("/profile/modification", post(modify_profile))
        .route("/profile/deleteAccount", post(delete_account))
        .route("/profile/login/{name}/{passcode}", get(login))
        .route("/profile/logcheck/{token}", get(login_check))
        .route("/profile/discover", get(discover_profiles))
        .route("/profile/DLBDD/{token}", get(download_database))
        .route("/profile/logout/{token}", post(logout))
        .with_state(state)
}
