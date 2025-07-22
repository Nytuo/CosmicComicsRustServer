use std::sync::Arc;

use axum::{extract::State, response::IntoResponse};
use reqwest::StatusCode;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    repositories::database_repo::insert_into_db, routes_manager::AppState,
    services::profile_service::resolve_token,
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
    let clean_db_info = db_info
        .trim_matches(|c| c == '(' || c == ')')
        .replace('\'', "");
    let clean_values = values
        .trim_matches(|c| c == '(' || c == ')')
        .replace('\'', "");
    let values_vector: Vec<String> = clean_values
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    let mut columns_vector: Vec<String> = clean_db_info
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();
    columns_vector = columns_vector
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
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

    match insert_into_db(
        &pool,
        &db_name,
        if columns_vector.is_empty() {
            None
        } else {
            Some(columns_vector)
        },
        values_vector,
    )
    .await
    {
        Ok(_) => {
            println!("Inserted into DB: {} {}", db_info, values);
            (StatusCode::OK, "Insert successful").into_response()
        }
        Err(error_msg) => {
            eprintln!("Failed to insert into DB: {}", error_msg);
            (StatusCode::INTERNAL_SERVER_ERROR, "Insert failed").into_response()
        }
    }
}

pub async fn write_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(json_file): axum::extract::Path<String>,
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;
    let json_file = json_file.replace("'", "''").replace("\"", "\\\"");
    let json_file_path = format!("{}/{}.json", base_path, json_file);
    let json_data = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| {
        eprintln!("Failed to serialize JSON");
        "Serialization failed".to_string()
    });
    std::fs::write(&json_file_path, json_data).unwrap_or_else(|_| {
        eprintln!("Failed to write to file");
        (StatusCode::INTERNAL_SERVER_ERROR, "File write failed").into_response();
    });
    println!("Wrote to file: {}", json_file_path);
    (StatusCode::OK, "Write successful").into_response()
}

pub async fn read_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path(json_file): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;
    let json_file = json_file.replace("'", "''").replace("\"", "\\\"");
    let json_file_path = format!("{}/{}.json", base_path, json_file);
    let json_data = std::fs::read_to_string(&json_file_path).unwrap_or_else(|_| {
        eprintln!("Failed to read file");
        "File read failed".to_string()
    });
    println!("Read from file: {}", json_file_path);
    (StatusCode::OK, json_data).into_response()
}

pub async fn update_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, db_name, col_name, value, id)): axum::extract::Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
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
    let col_name = col_name.replace("'", "''").replace("\"", "\\\"");
    let value = value.replace("'", "''").replace("\"", "\\\"");
    let id = id.replace("'", "''").replace("\"", "\\\"");

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

    let col_vec = vec![col_name.clone()];
    let val_vec = vec![value.clone()];

    crate::repositories::database_repo::update_db(
        &pool, "no_edit", col_vec, val_vec, &db_name, "ID_book", &id,
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to update DB");
        (StatusCode::INTERNAL_SERVER_ERROR, "Update failed").into_response();
    });

    println!("Updated DB: {} {} {} {}", db_name, col_name, value, id);
    (StatusCode::OK, "Update successful").into_response()
}

pub async fn update_db_body(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let global = state.global_vars.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let token = payload["token"].as_str().unwrap_or_default();

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    println!("payload: {:?}", payload);

    let type_name = payload["type"].as_str().unwrap_or_default();
    let db_name = payload["table"].as_str().unwrap_or_default();
    let columns = payload["column"].as_array().unwrap_or(&vec![]);
    let values = payload["value"].as_array().unwrap_or(&vec![]);
    let where_ = payload["where"].as_str().unwrap_or_default();
    let where_value = payload["whereEl"].as_str().unwrap_or_default();

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

    let columns: Vec<String> = if payload["column"].is_array() {
        payload["column"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_string())
            .collect()
    } else {
        vec![payload["column"].as_str().unwrap_or_default().to_string()]
    };

    let values: Vec<String> = if payload["value"].is_array() {
        payload["value"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_string())
            .collect()
    } else {
        vec![payload["value"].as_str().unwrap_or_default().to_string()]
    };

    crate::repositories::database_repo::update_db(
        &pool,
        type_name,
        columns,
        values,
        &db_name,
        &where_,
        &where_value,
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to update DB");
        (StatusCode::INTERNAL_SERVER_ERROR, "Update failed").into_response();
    });

    (StatusCode::OK, "Update successful").into_response()
}

pub async fn update_db_one_for_all(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let global = state.global_vars.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let token = payload["token"].as_str().unwrap_or_default();

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let W1 = payload["W1"].as_str().unwrap_or_default();
    let W2 = payload["W2"].as_str().unwrap_or_default();
    let A = payload["A"].as_str().unwrap_or_default();
    let title = payload["title"].as_str().unwrap_or_default();

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

    let book_list = crate::repositories::database_repo::select_from_db(
        &pool,
        "Books",
        vec!["*".to_string()],
        Some(vec![&*W1.to_string(), &*W2.to_string()]),
        Some(vec!["1", "1"]),
        Some("OR"),
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to select from DB");
        Vec::new()
    });

    for book in book_list {
        let path = book.get("PATH").unwrap().to_string().to_lowercase();
        let title_json = serde_json::from_str::<Value>(&title).unwrap();
        let en_title = title_json["english"]
            .as_str()
            .unwrap_or_default()
            .to_string()
            .to_lowercase()
            .replace('"', "");
        if path.contains(&en_title) {
            let asso = json!({
                A: 1,
                W1: 0,
                W2: 0
            });
            let columns = asso
                .as_object()
                .unwrap()
                .keys()
                .cloned()
                .collect::<Vec<_>>();
            let values = asso
                .as_object()
                .unwrap()
                .values()
                .cloned()
                .collect::<Vec<_>>();
            let values_vec_string = values.iter().map(|v| v.to_string()).collect::<Vec<_>>();
            crate::repositories::database_repo::update_db(
                &pool,
                "edit",
                columns,
                values_vec_string,
                "Books",
                "PATH",
                path.as_str(),
            )
            .await
            .unwrap_or_else(|_| {
                eprintln!("Failed to update DB");
                (StatusCode::INTERNAL_SERVER_ERROR, "Update failed").into_response();
            });
        }
    }

    println!("Updated DB: {} {} {} {}", W1, W2, A, title);
    (StatusCode::OK, "Update successful").into_response()
}

pub async fn update_lib(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, id)): axum::extract::Path<(String, String)>,
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

    let name = payload["name"].as_str().unwrap_or_default();
    let path = payload["path"].as_str().unwrap_or_default();
    let api = payload["api"].as_str().unwrap_or_default();

    let col_vec = vec!["NAME".to_string(), "PATH".to_string(), "API_ID".to_string()];
    let val_vec = vec![name.to_string(), path.to_string(), api.to_string()];

    crate::repositories::database_repo::update_db(
        &pool,
        "no_edit",
        col_vec,
        val_vec,
        "Libraries",
        "ID_LIBRARY",
        &id,
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to update library");
        (StatusCode::INTERNAL_SERVER_ERROR, "Update failed").into_response();
    });

    println!("Updated library: {}", id);
    (StatusCode::OK, "Update successful").into_response()
}

pub async fn delete_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, db_name, id, option)): axum::extract::Path<(
        String,
        String,
        String,
        String,
    )>,
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
    let id = id.replace("'", "''").replace("\"", "\\\"");
    let option = option.replace("'", "''").replace("\"", "\\\"");

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

    crate::repositories::database_repo::delete_from_db(
        &pool,
        &db_name,
        "ID_book",
        &*id,
        Some(&*option.clone()),
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to delete from DB");
        (StatusCode::INTERNAL_SERVER_ERROR, "Delete failed").into_response();
    });

    println!("Deleted from DB: {} {}", db_name, id);
    (StatusCode::OK, "Delete successful").into_response()
}

pub async fn true_delete_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, db_name, id)): axum::extract::Path<(String, String, String)>,
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
    let id = id.replace("'", "''").replace("\"", "\\\"");

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

    crate::repositories::database_repo::delete_from_db(
        &pool,
        &db_name,
        "ID_book",
        &*id,
        Option::None,
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to true delete from DB");
        (StatusCode::INTERNAL_SERVER_ERROR, "True delete failed").into_response();
    });

    println!("True deleted from DB: {} {}", db_name, id);
    (StatusCode::OK, "True delete successful").into_response()
}

pub async fn delete_lib(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, id)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let global = state.global_vars.lock().await;
    let config = state.config.lock().await;
    let base_path = &config.base_path;

    let resolved_token = match resolve_token(&token, base_path) {
        Some(t) => t,
        None => return (StatusCode::UNAUTHORIZED, "Invalid token").into_response(),
    };

    let id = id.replace("'", "''").replace("\"", "\\\"");

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

    crate::repositories::database_repo::delete_from_db(
        &pool,
        "Libraries",
        "ID_LIBRARY",
        &*id,
        Option::None,
    )
    .await
    .unwrap_or_else(|_| {
        eprintln!("Failed to delete from library");
        (StatusCode::INTERNAL_SERVER_ERROR, "Delete failed").into_response();
    });

    println!("Deleted from library: {}", id);
    (StatusCode::OK, "Delete successful").into_response()
}

pub async fn get_db(
    State(state): State<Arc<Mutex<AppState>>>,
    axum::extract::Path((token, dbName)): axum::extract::Path<(String, String)>,
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

    let request = payload["request"].as_str().unwrap_or_default();

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

    let result = crate::repositories::database_repo::select_from_db_with_options(&pool, request)
        .await
        .unwrap_or_else(|_| {
            eprintln!("Failed to select from DB");
            Vec::new()
        });

    println!("Selected from DB: {} {}", dbName, request);
    match serde_json::to_string(&result) {
        Ok(json_result) => (StatusCode::OK, json_result).into_response(),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to serialize result",
        )
            .into_response(),
    }
}
