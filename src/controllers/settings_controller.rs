use crate::routes_manager::AppState;
use crate::services::profile_service::{CreateUserPayload, create_user_service, resolve_token};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub async fn create_first_user(
    State(state): State<Arc<Mutex<AppState>>>,
    Path((name, passcode)): Path<(String, String)>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let payload_from_path = CreateUserPayload::new(name, passcode, None);
    match create_user_service(&payload_from_path, &base_path).await {
        Ok(_) => {
            info!("User created successfully");
            StatusCode::OK
        }
        Err(e) => {
            error!("Error creating user: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn get_version(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let version = config.version.clone();
    Json(version).into_response()
}

pub async fn write_config(
    State(state): State<Arc<Mutex<AppState>>>,
    Path(token): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let resolved_token = match resolve_token(&token, &*base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    let file_path = format!("{}/profiles/{}/config.json", base_path, resolved_token);

    if let Err(e) = fs::write(&file_path, serde_json::to_string_pretty(&payload).unwrap()) {
        error!("Failed to write config file: {}", e);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    StatusCode::OK.into_response()
}
