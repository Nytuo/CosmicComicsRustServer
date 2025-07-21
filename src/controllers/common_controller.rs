use crate::services::profile_service::resolve_token;
use crate::{
    routes_manager::AppState,
    utils::{darken_color, is_light_color},
};
use axum::extract::Request;
use axum::http::HeaderMap;
use axum::{extract::State, response::IntoResponse};
use reqwest::StatusCode;
use rgb::RGB;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::{env, fs, path::Path, sync::Arc};
use tokio::sync::Mutex;
use zip::write::FileOptions;

#[derive(Serialize)]
struct SerializableRgb {
    r: u8,
    g: u8,
    b: u8,
}

impl From<RGB<u8>> for SerializableRgb {
    fn from(rgb: RGB<u8>) -> Self {
        SerializableRgb {
            r: rgb.r,
            g: rgb.g,
            b: rgb.b,
        }
    }
}

pub async fn get_dirname(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = config.base_path.clone();
    (StatusCode::OK, base_path.clone())
}
pub async fn get_lang(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(lang): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;
    let lang_path = format!("{}/languages/{}.json", base_path, lang);

    if !Path::new(&lang_path).exists() {
        return (StatusCode::NOT_FOUND, "Language file not found".to_string());
    }

    let lang_content = fs::read_to_string(&lang_path).unwrap_or_default();
    (StatusCode::OK, lang_content)
}

pub async fn get_null() -> impl IntoResponse {
    let null_image = format!(
        "{}/public/Images/fileDefault.png",
        env::current_dir().unwrap().to_str().unwrap()
    );
    if !Path::new(&null_image).exists() {
        return (StatusCode::NOT_FOUND, "Image not found").into_response();
    }
    let raw_img = match fs::read(&null_image) {
        Ok(data) => data,
        Err(_) => return (StatusCode::NOT_FOUND, "Image not found").into_response(),
    };
    (StatusCode::OK, raw_img).into_response()
}

pub async fn get_themes(State(state): State<Arc<Mutex<AppState>>>) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;
    let themes_dir = format!("{}/public/themes", base_path);

    let themes_path = Path::new(&themes_dir);
    if !themes_path.exists() {
        return (StatusCode::NOT_FOUND, "Themes directory not found").into_response();
    }
    let mut themes_available = vec![];
    for entry in fs::read_dir(&themes_path).unwrap() {
        let entry = entry.unwrap();
        if entry.path().is_dir() {
            let theme_name = entry.file_name().into_string().unwrap_or_default();
            themes_available.push(theme_name);
        }
    }
    let themes_content = serde_json::to_string(&themes_available).unwrap_or_default();
    let themes_content = format!(r#"{{"themes":{}}}"#, themes_content);
    (StatusCode::OK, themes_content).into_response()
}

pub async fn get_status(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, type_)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;
    let status_progress = state.global_vars.lock().await.progress_status.clone();
    let selected_status = status_progress
        .get(&token)
        .and_then(|status| status.get(&type_))
        .cloned()
        .unwrap_or_default();
    (
        StatusCode::OK,
        axum::Json(serde_json::json!(selected_status)),
    )
        .into_response()
}

pub async fn download_file(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;
    let full_path = format!("{}/{}", base_path, path);
    let path_obj = Path::new(&full_path);

    if path_obj.exists() {
        if path_obj.is_file() {
            let file_content = match fs::read(&full_path) {
                Ok(content) => content,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file")
                        .into_response();
                }
            };
            return (StatusCode::OK, file_content).into_response();
        } else if path_obj.is_dir() {
            let zip_path = format!(
                "{}/{}.zip",
                base_path,
                path_obj.file_name().unwrap().to_str().unwrap()
            );
            let zip_file = match File::create(&zip_path) {
                Ok(file) => file,
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to create zip file",
                    )
                        .into_response();
                }
            };

            let mut zip_writer = zip::ZipWriter::new(BufWriter::new(zip_file));
            let options =
                FileOptions::<()>::default().compression_method(zip::CompressionMethod::Stored);
            for entry in fs::read_dir(path_obj).unwrap() {
                let entry = entry.unwrap();
                let entry_path = entry.path();
                if entry_path.is_file() {
                    let file_name = entry_path.file_name().unwrap().to_str().unwrap();
                    zip_writer.start_file(file_name, options).unwrap();
                    let file_content = fs::read(entry_path).unwrap();
                    zip_writer.write_all(&file_content).unwrap();
                }
            }

            zip_writer.finish().unwrap();

            let zip_content = match fs::read(&zip_path) {
                Ok(content) => content,
                Err(_) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read zip file")
                        .into_response();
                }
            };
            return (StatusCode::OK, zip_content).into_response();
        }
    }

    (StatusCode::NOT_FOUND, "Path not found").into_response()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub id_bookmark: i32,
    pub book_id: String,
    pub path: String,
    pub page: i32,
}

pub async fn get_bookmarks(
    State(state): State<Arc<Mutex<AppState>>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let token = match headers.get("token").and_then(|v| v.to_str().ok()) {
        Some(t) => t.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing token header").into_response(),
    };

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let pool = match crate::repositories::database_repo::get_db(
        &resolved_token,
        &base_path,
        state.global_vars.lock().await.opened_db.clone(),
    )
    .await
    {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Error getting database pool");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting database pool",
            )
                .into_response();
        }
    };

    let query = "SELECT * FROM Bookmarks;";
    let query_builder = sqlx::query(query);
    let stmt = match query_builder.fetch_all(&pool).await {
        Ok(stmt) => stmt,
        Err(e) => {
            eprintln!("Error executing query: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Error executing query").into_response();
        }
    };

    let bookmarks: Vec<Bookmark> = stmt
        .iter()
        .map(|row| Bookmark {
            id_bookmark: row.get("id_bookmark"),
            book_id: row.get("book_id"),
            path: row.get("path"),
            page: row.get("page"),
        })
        .collect();

    (StatusCode::OK, axum::Json(bookmarks)).into_response()
}
