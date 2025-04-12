use std::{fs, path::Path, sync::Arc};

use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde::Deserialize;
use tokio::sync::Mutex;

use crate::{
    routes_manager::AppState,
    services::authentification_service::{CreateUserPayload, create_user_service},
};

pub async fn create_user(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(payload): Json<CreateUserPayload>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    match create_user_service(&payload, &base_path).await {
        Ok(_) => {
            println!("User created successfully");
            StatusCode::OK
        }
        Err(e) => {
            eprintln!("Error creating user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
