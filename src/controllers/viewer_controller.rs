use axum::{Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use futures_util::TryStreamExt;
use multer::Multipart;
use std::{convert::Infallible, path::Path, sync::Arc};
use tokio::{fs::File, io::AsyncWriteExt};
use tower_http::limit::RequestBodyLimitLayer;

use crate::routes_manager::AppState;

use axum::extract::Multipart as AxumMultipart;

pub async fn upload_comic_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    mut multipart: AxumMultipart,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field.file_name().unwrap_or("default_name");
        let upload_path = Path::new(base_path).join("uploads").join(file_name);

        if let Some(parent) = upload_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create directory: {}", e),
                );
            }
        }

        if upload_path.exists() {
            eprintln!("File {} already exists, skipping upload.", file_name);
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
