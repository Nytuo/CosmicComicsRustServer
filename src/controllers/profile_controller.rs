use std::{fs, path::Path, sync::Arc};

use axum::http::{HeaderMap, Request};
use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::Mutex;

use crate::{
    routes_manager::AppState,
    services::profile_service::{CreateUserPayload, create_user_service, resolve_token},
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

pub async fn get_profile_picture(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let name_from_token = match resolve_token(&token, &base_path) {
        Some(name) => Some(name),
        None => {
            eprintln!("Token not found in serverconfig.json");
            None
        }
    };
    let token = name_from_token.unwrap_or(token);

    let file_path = format!("{}/profiles/{}/pp.png", base_path, token);

    if Path::new(&file_path).exists() {
        (StatusCode::OK, Json(file_path))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Profile picture not found".to_string()),
        )
    }
}

pub async fn get_profile_picture_by_name(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let file_path = format!("{}/profiles/{}/pp.png", base_path, name);

    if Path::new(&file_path).exists() {
        (StatusCode::OK, Json(file_path))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Profile picture not found".to_string()),
        )
    }
}

pub async fn get_custom_number(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let dir_path = format!("{}/public/Images/account_default", base_path);
    let files = match fs::read_dir(dir_path) {
        Ok(entries) => entries
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|_| vec![]),
        Err(_) => vec![],
    };

    let count = files.len();
    let response = json!({ "length": count });
    (StatusCode::OK, Json(response))
}

pub async fn delete_account(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();
    let mut global = state.global_vars.lock().await;

    let token = payload["token"].as_str().unwrap_or_default();
    let token = match resolve_token(token, &base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    match crate::services::profile_service::delete_account_service(&token, &base_path, &mut global)
        .await
    {
        Ok(_) => {
            println!("Account deleted successfully");
            return (StatusCode::OK, "Account deleted successfully").into_response();
        }
        Err(e) => {
            eprintln!("Error deleting account: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to delete account",
            )
                .into_response();
        }
    }
}

pub async fn modify_profile(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let token = payload["token"].as_str().unwrap_or_default();
    let token = match resolve_token(token, &base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let new_pass: Option<&str> = payload["npass"].as_str();
    let new_pp: Option<&str> = payload["npp"].as_str();
    let new_user: Option<&str> = payload["nuser"].as_str();

    match crate::services::profile_service::modify_profile_service(
        &token, new_pass, new_pp, new_user, &base_path,
    )
    .await
    {
        Ok(_) => {
            println!("Profile modified successfully");
            return (StatusCode::OK, "Profile modified successfully").into_response();
        }
        Err(e) => {
            eprintln!("Error modifying profile: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to modify profile",
            )
                .into_response();
        }
    }
}

pub async fn login(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((name, passcode)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    match crate::services::profile_service::login_service(&name, &passcode, &base_path).await {
        Ok(token) => {
            println!("Login successful");
            (StatusCode::OK, Json(token)).into_response()
        }
        Err(e) => {
            eprintln!("Error logging in: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response()
        }
    }
}

pub async fn login_check(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    match crate::services::profile_service::login_check_service(&token, &base_path).await {
        Ok(name) => {
            println!("Token is valid");
            (StatusCode::OK, Json(name)).into_response()
        }
        Err(e) => {
            eprintln!("Error checking token: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
        }
    }
}

pub async fn discover_profiles(
    header_map: HeaderMap,
    State(state): State<Arc<Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let headers = header_map.clone();
    let protocol = headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("http");

    let host = headers
        .get("host")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("localhost");

    match crate::services::profile_service::discover_profiles_service(&base_path, protocol, host)
        .await
    {
        Ok(profiles) => {
            println!("Profiles discovered successfully");
            (StatusCode::OK, Json(profiles)).into_response()
        }
        Err(e) => {
            eprintln!("Error discovering profiles: {}", e);
            (StatusCode::OK, Json("[]")).into_response()
        }
    }
}

pub async fn download_database(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    let token = match resolve_token(&token, &base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let file_path = format!("{}/profiles/{}/CosmicComics.db", base_path, token);

    if Path::new(&file_path).exists() {
        match fs::read(&file_path) {
            Ok(file) => {
                let file_size = file.len();
                let mut headers_for_response_download = HeaderMap::new();
                headers_for_response_download.insert(
                    "Content-Disposition",
                    "attachment; filename=\"CosmicComics.db\"".parse().unwrap(),
                );
                headers_for_response_download
                    .insert("Content-Type", "application/octet-stream".parse().unwrap());
                headers_for_response_download
                    .insert("Content-Length", file_size.to_string().parse().unwrap());
                headers_for_response_download.insert("Accept-Ranges", "bytes".parse().unwrap());
                headers_for_response_download
                    .insert("Content-Transfer-Encoding", "binary".parse().unwrap());
                headers_for_response_download.insert("Cache-Control", "no-cache".parse().unwrap());
                headers_for_response_download.insert("Pragma", "no-cache".parse().unwrap());
                headers_for_response_download.insert("Expires", "0".parse().unwrap());
                headers_for_response_download.insert("Connection", "keep-alive".parse().unwrap());
                headers_for_response_download
                    .insert("Accept-Encoding", "gzip, deflate".parse().unwrap());
                headers_for_response_download.insert("Accept", "*/*".parse().unwrap());

                // If a custom content-length header is present, log a warning (not set in response)
                if headers_for_response_download.contains_key("custom-content-length") {
                    eprintln!(
                        "Warning: custom-content-length header present, ignoring in favor of standard Content-Length."
                    );
                    headers_for_response_download.remove("custom-content-length");
                }

                let response = (StatusCode::OK, headers_for_response_download, file);
                return response.into_response();
            }
            Err(_) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json("Failed to read database file".to_string()),
                )
                    .into_response();
            }
        }
    } else {
        (
            StatusCode::NOT_FOUND,
            Json("Database file not found".to_string()),
        )
            .into_response()
    }
}

pub async fn logout(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(token): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();

    match crate::services::profile_service::logout_service(&token, &base_path).await {
        Ok(_) => {
            println!("Logout successful");
            (StatusCode::OK, "Logout successful").into_response()
        }
        Err(e) => {
            eprintln!("Error logging out: {}", e);
            (StatusCode::UNAUTHORIZED, "Invalid token").into_response()
        }
    }
}
