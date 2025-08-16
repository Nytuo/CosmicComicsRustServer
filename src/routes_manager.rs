use crate::ApiTokens;
use crate::AppConfig;
use crate::AppGlobalVariables;
use crate::endpoints::api_endpoints::api_routes;
use crate::endpoints::collectionner_endpoints::collectionner_routes;
use crate::endpoints::common_endpoints::common_routes;
use crate::endpoints::database_endpoints::database_routes;
use crate::endpoints::profile_endpoints::authentication_routes;
use crate::endpoints::settings_endpoints::settings_routes;
use crate::endpoints::viewer_endpoints::viewer_routes;
use axum::Router;
use axum::middleware::from_fn;
use axum::{
    body::Body,
    http::{HeaderMap, Request},
    middleware::Next,
    response::IntoResponse,
};
use std::collections::HashSet;
use std::sync::Arc;
use tracing::info;

pub struct AppState {
    pub config: Arc<tokio::sync::Mutex<AppConfig>>,
    pub creds: Arc<tokio::sync::Mutex<ApiTokens>>,
    pub global_vars: Arc<tokio::sync::Mutex<AppGlobalVariables>>,
}

pub async fn log_request(req: Request<Body>, next: Next) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let limit: usize = parts
        .headers
        .get("x-limit")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("1024")
        .parse()
        .unwrap_or(1024);
    let bytes = axum::body::to_bytes(body, limit).await.unwrap_or_default();
    let body_str = String::from_utf8_lossy(&bytes);
    let req = Request::from_parts(parts.clone(), Body::from(bytes.clone()));

    let headers = extract_interesting_headers(&parts.headers);

    info!(
        method = ?parts.method,
        path = %parts.uri.path(),
        headers = ?headers,
        body = %mask_body(&body_str),
        "[REQUEST]"
    );

    next.run(req).await
}

fn extract_interesting_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    let standard_headers: HashSet<&str> = [
        "host",
        "connection",
        "upgrade-insecure-requests",
        "user-agent",
        "accept",
        "accept-language",
        "accept-encoding",
        "cache-control",
        "pragma",
        "referer",
        "cookie",
        "content-length",
        "content-type",
        "origin",
        "sec-fetch-site",
        "sec-fetch-mode",
        "sec-fetch-user",
        "sec-fetch-dest",
        "x-forwarded-for",
        "x-forwarded-proto",
        "x-real-ip",
    ]
    .into_iter()
    .collect();

    headers
        .iter()
        .filter_map(|(name, value)| {
            let key = name.as_str().to_ascii_lowercase();
            if !standard_headers.contains(key.as_str()) {
                Some((key, value.to_str().unwrap_or("<invalid utf8>").to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn mask_body(body: &str) -> String {
    if body.trim().is_empty() {
        "<empty>".to_string()
    } else if body.len() > 1000 {
        format!("{}...[truncated]", &body[..1000])
    } else {
        body.to_string()
    }
}

pub fn create_router(
    config: Arc<tokio::sync::Mutex<AppConfig>>,
    creds: Arc<tokio::sync::Mutex<ApiTokens>>,
    global_vars: Arc<tokio::sync::Mutex<AppGlobalVariables>>,
) -> Router {
    let state = Arc::new(tokio::sync::Mutex::new(AppState {
        config: config.clone(),
        creds: creds.clone(),
        global_vars: global_vars.clone(),
    }));
    Router::new()
        .merge(common_routes(state.clone()))
        .merge(authentication_routes(state.clone()))
        .merge(settings_routes(state.clone()))
        .merge(collectionner_routes(state.clone()))
        .merge(viewer_routes(state.clone()))
        .merge(database_routes(state.clone()))
        .merge(api_routes(state.clone()))
        .fallback(fallback_handler)
        .layer(from_fn(log_request))
}

async fn fallback_handler() -> impl IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        "This endpoint does not exist.",
    )
}
