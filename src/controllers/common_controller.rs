use std::{env, fs, path::Path, sync::Arc};

use axum::{extract::State, response::IntoResponse};
use color_thief::get_palette;
use reqwest::StatusCode;
use rgb::RGB;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::services::authentification_service::resolve_token;
use crate::{
    routes_manager::AppState,
    utils::{darken_color, is_light_color},
};

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

pub async fn get_color(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((img, token)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let img_path = format!(
        "{}/profiles/{}/current_book/{}",
        base_path, resolved_token, img
    );

    let raw_img = match fs::read(&img_path) {
        Ok(data) => data,
        Err(_) => return (StatusCode::NOT_FOUND, "Image not found").into_response(),
    };
    let palette = match get_palette(&raw_img, color_thief::ColorFormat::Rgb, 5, 1) {
        Ok(p) => p,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get palette").into_response();
        }
    };
    let serializable_palette: Vec<SerializableRgb> =
        palette.into_iter().map(SerializableRgb::from).collect();

    let palette_json = serde_json::to_string(&serializable_palette).unwrap_or_default();
    (StatusCode::OK, palette_json).into_response()
}

pub async fn get_palette_color(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(token): axum::extract::Path<String>,
    axum::extract::Json(headers): axum::extract::Json<serde_json::Value>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let mut img = match headers.get("img").and_then(|v| v.as_str()) {
        Some(img) => img.to_string(),
        None => return (StatusCode::BAD_REQUEST, "Missing 'img' header").into_response(),
    };

    if img.contains("localhost") {
        img = format!(
            "{}/public/{}",
            base_path,
            img.split("localhost").nth(1).unwrap_or("")
        );
    } else if img.contains("fileDefault") {
        img = format!("{}/public/Images/fileDefault.png", base_path);
    } else {
        img = format!(
            "{}/profiles/{}/current_book/{}",
            base_path, resolved_token, img
        );
    }

    let raw_img = match fs::read(&img) {
        Ok(data) => data,
        Err(_) => return (StatusCode::NOT_FOUND, "Image not found").into_response(),
    };

    let palette = match get_palette(&raw_img, color_thief::ColorFormat::Rgb, 5, 2) {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!(["rgb(33,33,33)", "rgb(33,33,33)"])),
            )
                .into_response();
        }
    };

    let mut adjusted_palette: Vec<String> = palette
        .into_iter()
        .take(2)
        .map(|color| {
            let rgb = format!("rgb({},{},{})", color.r, color.g, color.b);
            if is_light_color(color.r, color.g, color.b) {
                darken_color(color.r, color.g, color.b)
            } else {
                rgb
            }
        })
        .collect();

    while adjusted_palette.len() < 2 {
        adjusted_palette.push("rgb(33,33,33)".to_string());
    }

    (
        StatusCode::OK,
        axum::Json(serde_json::json!(adjusted_palette)),
    )
        .into_response()
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
