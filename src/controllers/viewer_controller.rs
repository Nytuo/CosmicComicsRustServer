use crate::routes_manager::AppState;
use axum::body::Body;
use axum::{extract::State, http::StatusCode, response::IntoResponse};
use futures_util::TryStreamExt;
use std::path::PathBuf;
use std::{fs, sync::Arc};
use tokio::{fs::File, io::AsyncWriteExt};
use tokio_util::io::ReaderStream;
use tracing::{error, info};

use crate::services::archive_service::unzip_and_process;
use crate::services::profile_service::resolve_token;
use crate::utils::{
    VALID_BOOK_EXTENSION, VALID_IMAGE_EXTENSION, get_list_of_images, replace_html_address_path,
};
use axum::extract::{Multipart as AxumMultipart, Path};
use axum::http::{HeaderMap, Response};
use axum_macros::debug_handler;

pub async fn upload_comic_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    mut multipart: AxumMultipart,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field.file_name().unwrap_or("default_name");
        let upload_path = std::path::Path::new(base_path)
            .join("uploads")
            .join(file_name);

        if let Some(parent) = upload_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create directory: {}", e),
                );
            }
        }

        if upload_path.exists() {
            error!("File {} already exists, skipping upload.", file_name);
            continue;
        }

        let mut file = match File::create(&upload_path).await {
            Ok(f) => f,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Create error: {}", e),
                );
            }
        };

        let mut field_stream = field.into_stream();
        while let Some(chunk) = field_stream.try_next().await.unwrap_or(None) {
            if let Err(e) = file.write_all(&chunk).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Write error: {}", e),
                );
            }
        }
    }

    (StatusCode::OK, "Upload successful".to_string())
}
#[debug_handler]
pub async fn unzip_controller(
    axum::extract::Path((path, token)): axum::extract::Path<(String, String)>,
    axum::extract::State(state): axum::extract::State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let current_path = replace_html_address_path(&path);

    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let output_dir = format!("{}/profiles/{}/current_book", base_path, resolved_token);
    let ext = std::path::Path::new(&current_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default();

    if !VALID_BOOK_EXTENSION.contains(&ext) {
        error!("This extensions is not (yet) supported");
        return (
            StatusCode::NOT_ACCEPTABLE,
            "This extensions is not (yet) supported",
        )
            .into_response();
    }

    let unzip_result = unzip_and_process(
        &current_path,
        &output_dir,
        ext,
        token.clone(),
        &state.global_vars,
    )
    .await;
    match unzip_result {
        Ok(_) => {
            let response = format!(
                "Unzipped {} to {}",
                current_path,
                replace_html_address_path(&output_dir)
            );
            (StatusCode::OK, response).into_response()
        }
        Err(e) => {
            error!("Error unzipping file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to unzip file: {}", e),
            )
                .into_response()
        }
    }
}

pub async fn view_current_controller(
    axum::extract::Path(token): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let current_book_path = format!("{}/profiles/{}/current_book", base_path, resolved_token);

    let list_of_images = get_list_of_images(current_book_path.as_ref(), VALID_IMAGE_EXTENSION);

    if list_of_images.is_empty() {
        return (StatusCode::OK, "false").into_response();
    }

    let response = match serde_json::to_string(&list_of_images) {
        Ok(json) => json,
        Err(e) => {
            error!("Error serializing JSON: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to serialize JSON",
            )
                .into_response();
        }
    };
    (StatusCode::OK, response).into_response()
}

pub async fn viewer_is_dir(
    axum::extract::Path(path): axum::extract::Path<String>,
    axum::extract::State(_): axum::extract::State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let full_path = format!("{}", path);
    let is_dir = tokio::fs::metadata(&full_path)
        .await
        .map(|m| m.is_dir())
        .unwrap_or(false);

    (StatusCode::OK, is_dir.to_string()).into_response()
}

pub async fn get_config_controller(
    axum::extract::Path(token): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;

    let resolved_token = match resolve_token(&token, &config.base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };
    let profile = format!(
        "{}/profiles/{}/config.json",
        config.base_path, resolved_token
    );
    match tokio::fs::read_to_string(&profile).await {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(e) => {
            error!("Error reading config file: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read config file: {}", e),
            )
                .into_response()
        }
    }
}

pub async fn read_image(
    headers: HeaderMap,
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let met = headers.get("met").and_then(|v| v.to_str().ok());
    let page = headers.get("page").and_then(|v| v.to_str().ok());

    let file_path = match met {
        Some("DL") => {
            let path = headers.get("path").and_then(|v| v.to_str().ok());
            match (path, page) {
                (Some(p), Some(pg)) => Some(PathBuf::from(format!("{}/{}", p, pg))),
                _ => None,
            }
        }
        Some("CLASSIC") => {
            let token = headers.get("token").and_then(|v| v.to_str().ok());
            let token_unwrapped = token.unwrap_or("");
            match (token, page) {
                (Some(_), Some(pg)) => {
                    let resolved_token = match resolve_token(&token_unwrapped, &config.base_path) {
                        Some(t) => t,
                        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
                    };
                    Some(PathBuf::from(format!(
                        "{}/profiles/{}/current_book/{}",
                        base_path, resolved_token, pg
                    )))
                }
                _ => None,
            }
        }
        _ => None,
    };

    if let Some(path) = file_path {
        match File::open(&path).await {
            Ok(file) => {
                let stream = ReaderStream::new(file);
                Response::builder()
                    .status(StatusCode::OK)
                    .header("Content-Type", "image/png")
                    .body(Body::from_stream(stream))
                    .unwrap()
            }
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("File not found"))
                .unwrap(),
        }
    } else {
        Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Invalid headers"))
            .unwrap()
    }
}

pub async fn viewer_view_controller(
    headers: HeaderMap,
    State(_): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let path = headers.get("path").and_then(|v| v.to_str().ok());
    if let Some(path) = path {
        let param = replace_html_address_path(path);
        info!("Received path: {}", param);
        let mut tosend = get_list_of_images((&param).as_ref(), crate::utils::VALID_IMAGE_EXTENSION);
        tosend.sort();
        info!("Sending list of images: {:?}", tosend);
        return match serde_json::to_string(&tosend) {
            Ok(json) => (StatusCode::OK, json).into_response(),
            Err(e) => {
                error!("Error serializing JSON: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to serialize JSON",
                )
                    .into_response()
            }
        };
    }
    (StatusCode::BAD_REQUEST, "Missing or invalid 'path' header").into_response()
}

pub async fn view_current_page_controller(
    Path((page, token)): Path<(usize, String)>,
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let current_book_path = format!("{}/profiles/{}/current_book", base_path, resolved_token);
    let list_of_images = get_list_of_images(
        (&current_book_path).as_ref(),
        crate::utils::VALID_IMAGE_EXTENSION,
    );

    if let Some(image) = list_of_images.get(page) {
        let image_path = format!("{}/{}", current_book_path, image);
        return (StatusCode::OK, image_path).into_response();
    }

    (StatusCode::NOT_FOUND, "Image not found").into_response()
}

pub async fn view_exist_controller(
    Path(path): Path<String>,
    State(_): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let full_path = replace_html_address_path(&path);

    let exists = tokio::fs::metadata(&full_path).await.is_ok();
    info!("File exists: {}", exists);

    (StatusCode::OK, exists.to_string()).into_response()
}

pub async fn view_read_file_controller(
    Path(path): Path<String>,
    State(_): State<Arc<tokio::sync::Mutex<AppState>>>,
) -> impl IntoResponse {
    let sanitized_path = replace_html_address_path(&path);
    let full_path = format!("{}", sanitized_path);

    match fs::read_to_string(&full_path) {
        Ok(content) => {
            let trimmed = content.trim_end();
            (
                StatusCode::OK,
                serde_json::to_string(trimmed).unwrap_or_default(),
            )
                .into_response()
        }
        Err(e) => {
            error!("Error reading file: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}
