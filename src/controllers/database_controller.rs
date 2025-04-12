use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::{
    repositories::database_repo::insert_into_db, routes_manager::AppState,
    services::authentification_service::resolve_token,
};

pub async fn insert_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, db_name)): axum::extract::Path<(String, String)>,
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let global = state.global_vars.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let db_name = db_name.replace("'", "''").replace("\"", "\\\"");
    let db_info = payload["into"].as_str().unwrap_or_default();
    let values = payload["val"].as_str().unwrap_or_default();

    let pool = match crate::repositories::database_repo::get_db(
        &resolved_token,
        base_path,
        global.opened_db.clone(),
    )
    .await
    {
        Ok(pool) => pool,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get DB").into_response(),
    };
    insert_into_db(&pool, &db_name, vec![db_info], vec![values])
        .await
        .unwrap_or_else(|_| {
            eprintln!("Failed to insert into DB");
            (StatusCode::INTERNAL_SERVER_ERROR, "Insert failed").into_response();
        });
    println!("Inserted into DB: {} {}", db_info, values);
    (StatusCode::OK, "Insert successful").into_response()
}
