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
use axum::http::Request;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use std::sync::Arc;

pub struct AppState {
    pub config: Arc<tokio::sync::Mutex<AppConfig>>,
    pub creds: Arc<tokio::sync::Mutex<ApiTokens>>,
    pub global_vars: Arc<tokio::sync::Mutex<AppGlobalVariables>>,
}

pub async fn log_request(
    req: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    println!("[REQUEST] {} {}", req.method(), req.uri().path());
    println!("[HEADERS]");
    for (name, value) in req.headers().iter() {
        println!("{}: {:?}", name, value);
    }
    let (parts, body) = req.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .unwrap_or_default();
    let body_str = String::from_utf8_lossy(&bytes);
    println!("[BODY] {}", body_str);
    let req = Request::from_parts(parts, axum::body::Body::from(bytes));
    next.run(req).await
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
