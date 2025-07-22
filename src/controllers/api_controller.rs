use crate::repositories::database_repo::insert_into_db;
use crate::routes_manager::AppState;
use crate::services::anilist_service::{api_anilist_get, api_anilist_get_search};
use crate::services::googlebooks_service::search_gbapi_comics_by_name;
use crate::services::marvel_service::{
    api_marvel_get, get_marvel_api_characters, get_marvel_api_comics, get_marvel_api_creators,
    get_marvel_api_relations, get_marvel_api_search,
};
use crate::services::openlibrary_service::get_olapi_search;
use crate::services::profile_service::resolve_token;
use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

pub async fn marvel_search_only(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path((name, date)): axum::extract::Path<(String, Option<String>)>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let api_tokens = state.creds.clone();
    let marvel_public_key = api_tokens.lock().await.marvel_public_key.clone();
    let marvel_private_key = api_tokens.lock().await.marvel_private_key.clone();
    let name = name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }

    if let Some(date) = date.clone() {
        if date.is_empty() {
            return (StatusCode::BAD_REQUEST, "Date cannot be empty".to_string()).into_response();
        }
    }

    match get_marvel_api_search(
        &*name,
        date.clone(),
        &*marvel_private_key,
        &*marvel_public_key,
    )
    .await
    {
        Ok(response) => {
            info!("marvel API search returned");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error fetching data from Marvel API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from Marvel API",
            )
                .into_response()
        }
    }
}

pub async fn marvel_get_comics(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path((name, date)): axum::extract::Path<(String, String)>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let api_tokens = state.creds.clone();
    let marvel_public_key = api_tokens.lock().await.marvel_public_key.clone();
    let marvel_private_key = api_tokens.lock().await.marvel_private_key.clone();
    let name = name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }

    if date.is_empty() {
        return (StatusCode::BAD_REQUEST, "Date cannot be empty".to_string()).into_response();
    }

    match get_marvel_api_comics(&*name, &*date, &*marvel_private_key, &*marvel_public_key).await {
        Ok(response) => {
            info!("marvel API search returned");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error fetching data from Marvel API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from Marvel API",
            )
                .into_response()
        }
    }
}

pub async fn openlibrary_search(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let name = name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }

    match get_olapi_search(&*name).await {
        Ok(response) => {
            info!("openlibrary API search returned");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error fetching data from OpenLibrary API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from OpenLibrary API",
            )
                .into_response()
        }
    }
}

pub async fn googlebooks_search(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let api_tokens = state.creds.clone();
    let google_books_api_key = api_tokens.lock().await.google_books_api_key.clone();
    let name = name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }

    match search_gbapi_comics_by_name(&*name, google_books_api_key).await {
        Ok(response) => {
            info!("googlebooks API search returned");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error fetching data from GoogleBooks API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from GoogleBooks API",
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct apiSeriesAdd {
    name: String,
    token: String,
    path: String,
}

pub async fn marvel_add(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Json(marvel): Json<apiSeriesAdd>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let api_tokens = state.creds.clone();
    let marvel_public_key = api_tokens.lock().await.marvel_public_key.clone();
    let marvel_private_key = api_tokens.lock().await.marvel_private_key.clone();
    let base_path = state.config.lock().await.base_path.clone();

    let name = marvel.name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }
    let token = marvel.token.clone();
    if token.is_empty() {
        return (StatusCode::BAD_REQUEST, "Token cannot be empty".to_string()).into_response();
    }
    let path = marvel.path.clone();
    if path.is_empty() {
        return (StatusCode::BAD_REQUEST, "Path cannot be empty".to_string()).into_response();
    }

    let pool = match crate::repositories::database_repo::get_db(
        &token,
        &base_path,
        state.global_vars.lock().await.opened_db.clone(),
    )
    .await
    {
        Ok(pool) => pool,
        Err(_) => {
            error!("Error getting database pool");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting database pool",
            )
                .into_response();
        }
    };

    match api_marvel_get(&*name, &*marvel_private_key, &*marvel_public_key).await {
        Ok(data) => {
            let columns = "ID_Series,title,note,start_date,end_date,description,Score,cover,BG,CHARACTERS,STAFF,SOURCE,volumes,chapters,favorite,PATH,lock";
            if (data
                .get("data")
                .unwrap()
                .get("total")
                .unwrap()
                .as_str()
                .unwrap()
                == "0")
            {
                let rand_id = crate::utils::generate_random_id();
                let values_vec = vec![
                    (rand_id.to_string() + "U_1"),
                    name.trim().replace("'", "''").to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "0".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "0".to_string(),
                    path.to_string(),
                    "false".to_string(),
                ];
                let columns_to_vec = columns
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>();
                insert_into_db(&pool, "Series", Some(columns_to_vec), values_vec)
                    .await
                    .expect("Failed to insert into Series");
            } else {
                let temp_data = data
                    .get("data")
                    .unwrap()
                    .get("results")
                    .unwrap()
                    .get(0)
                    .unwrap();
                let values_vec = vec![
                    (temp_data.get("id").unwrap().to_string() + "_1"),
                    temp_data
                        .get("title")
                        .unwrap()
                        .to_string()
                        .replace("'", "''"),
                    "null".to_string(),
                    temp_data
                        .get("startYear")
                        .unwrap()
                        .to_string()
                        .replace("'", "''"),
                    temp_data
                        .get("endYear")
                        .unwrap()
                        .to_string()
                        .replace("'", "''"),
                    temp_data
                        .get("description")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string()
                        .replace("'", "''"),
                    temp_data
                        .get("rating")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string()
                        .replace("'", "''"),
                    temp_data
                        .get("thumbnail")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string(),
                    temp_data
                        .get("thumbnail")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string(),
                    temp_data
                        .get("characters")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string()
                        .replace("'", "''"),
                    temp_data
                        .get("creators")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string()
                        .replace("'", "''"),
                    temp_data
                        .get("urls")
                        .and_then(|urls| urls.get(0))
                        .and_then(|url| Some(url))
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string(),
                    temp_data
                        .get("comics")
                        .and_then(|comics| comics.get("items"))
                        .and_then(|items| Some(items.to_string()))
                        .unwrap_or("".to_string())
                        .replace("'", "''"),
                    temp_data
                        .get("comics")
                        .and_then(|comics| comics.get("available"))
                        .and_then(|available| available.as_i64())
                        .map_or(0, |v| v as i32)
                        .to_string(),
                    "0".to_string(),
                    path.to_string(),
                    "false".to_string(),
                ];
                let columns_to_vec = columns
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>();
                insert_into_db(&pool, "Series", Some(columns_to_vec), values_vec)
                    .await
                    .expect("Failed to insert into Series");

                match get_marvel_api_creators(
                    temp_data.get("id").unwrap().as_str().unwrap(),
                    Option::Some("series"),
                    &*marvel_private_key,
                    &*marvel_public_key,
                )
                .await
                {
                    Ok(creators_data) => {
                        let creators_data =
                            creators_data.get("data").unwrap().get("results").unwrap();
                        let creators = creators_data.as_array().unwrap();
                        for creator in creators {
                            let values_vec = vec![
                                (creator.get("id").unwrap().to_string() + "_1"),
                                creator
                                    .get("fullName")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                creator
                                    .get("thumbnail")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                "null".to_string(),
                                creator
                                    .get("urls")
                                    .unwrap_or(&serde_json::Value::Null)
                                    .to_string(),
                            ];
                            insert_into_db(&pool, "Creators", Option::None, values_vec)
                                .await
                                .expect("Failed to insert into Creators");
                        }
                    }
                    Err(e) => {
                        error!("Error fetching data from Marvel API: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Error fetching data from Marvel API",
                        )
                            .into_response();
                    }
                }

                match get_marvel_api_characters(
                    temp_data.get("id").unwrap().as_str().unwrap(),
                    Option::Some("series"),
                    &*marvel_private_key,
                    &*marvel_public_key,
                )
                .await
                {
                    Ok(characters_data) => {
                        let characters_data =
                            characters_data.get("data").unwrap().get("results").unwrap();
                        let characters = characters_data.as_array().unwrap();
                        for character in characters {
                            let values_vec = vec![
                                (character.get("id").unwrap().to_string() + "_1"),
                                character
                                    .get("name")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                character
                                    .get("thumbnail")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                character
                                    .get("description")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                character
                                    .get("urls")
                                    .unwrap_or(&serde_json::Value::Null)
                                    .to_string(),
                            ];
                            insert_into_db(&pool, "Characters", Option::None, values_vec)
                                .await
                                .expect("Failed to insert into Characters");
                        }
                    }
                    Err(e) => {
                        error!("Error fetching data from Marvel API: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Error fetching data from Marvel API",
                        )
                            .into_response();
                    }
                }

                match get_marvel_api_relations(
                    temp_data.get("id").unwrap().as_str().unwrap(),
                    &*marvel_private_key,
                    &*marvel_public_key,
                )
                .await
                {
                    Ok(relations_data) => {
                        let relations_data =
                            relations_data.get("data").unwrap().get("results").unwrap();
                        let relations = relations_data.as_array().unwrap();
                        for relation in relations {
                            let values_vec = vec![
                                (relation.get("id").unwrap().to_string() + "_1"),
                                relation
                                    .get("title")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                relation
                                    .get("thumbnail")
                                    .unwrap()
                                    .to_string()
                                    .replace("'", "''"),
                                relation
                                    .get("description")
                                    .unwrap_or(&serde_json::Value::Null)
                                    .to_string()
                                    .replace("'", "''"),
                                relation
                                    .get("urls")
                                    .unwrap_or(&serde_json::Value::Null)
                                    .to_string(),
                                (temp_data.get("id").unwrap().to_string() + "_1"),
                            ];
                            insert_into_db(&pool, "relations", Option::None, values_vec)
                                .await
                                .expect("Failed to insert into relations");
                        }
                    }
                    Err(e) => {
                        error!("Error fetching data from Marvel API: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Error fetching data from Marvel API",
                        )
                            .into_response();
                    }
                }
            }
        }
        Err(e) => {
            error!("Error fetching data from Marvel API: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from Marvel API",
            )
                .into_response();
        }
    }

    (StatusCode::OK, "Series added successfully".to_string()).into_response()
}

pub async fn anilist_add(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Json(anilist): Json<apiSeriesAdd>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let base_path = state.config.lock().await.base_path.clone();

    let name = anilist.name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }
    let token = anilist.token.clone();
    if token.is_empty() {
        return (StatusCode::BAD_REQUEST, "Token cannot be empty".to_string()).into_response();
    }
    let path = anilist.path.clone();
    if path.is_empty() {
        return (StatusCode::BAD_REQUEST, "Path cannot be empty".to_string()).into_response();
    }
    let resolved_token = match resolve_token(&token, &base_path) {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                "Error resolving token".to_string(),
            )
                .into_response();
        }
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
            error!("Error getting database pool");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error getting database pool",
            )
                .into_response();
        }
    };

    match api_anilist_get(name.as_str()).await {
        Ok(data) => {
            let columns = "ID_Series,title,note,statut,start_date,end_date,description,Score,genres,cover,BG,CHARACTERS,TRENDING,STAFF,SOURCE,volumes,chapters,favorite,PATH,lock";
            if (data.is_none()) {
                let rand_id = crate::utils::generate_random_id();
                let values_vec = vec![
                    (rand_id.to_string() + "U_2"),
                    name.trim().replace("'", "''").to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "0".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "null".to_string(),
                    "0".to_string(),
                    path.to_string(),
                    "false".to_string(),
                ];
                let columns_to_vec = columns
                    .split(",")
                    .map(|s| s.trim().to_string())
                    .collect::<Vec<String>>();
                insert_into_db(&pool, "Series", Some(columns_to_vec), values_vec)
                    .await
                    .expect("Failed to insert into Series");
            }
            let base_data = data.clone().unwrap().get("base").unwrap().clone();
            let relations_data = data.clone().unwrap().get("relations").unwrap().clone();
            let characters_data = data.clone().unwrap().get("characters").unwrap().clone();
            let staff_data = data.clone().unwrap().get("staff").unwrap().clone();
            let values_vec = vec![
                (base_data.get("id").unwrap().to_string() + "_2"),
                base_data
                    .get("title")
                    .unwrap()
                    .to_string()
                    .replace("'", "''"),
                "null".to_string(),
                base_data.get("status").unwrap().to_string(),
                base_data.get("startDate").unwrap().to_string(),
                base_data.get("endDate").unwrap().to_string(),
                base_data
                    .get("description")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string()
                    .replace("'", "''"),
                base_data
                    .get("meanScore")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string(),
                base_data
                    .get("genres")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string()
                    .replace("'", "''"),
                base_data
                    .get("coverImage")
                    .unwrap_or(&serde_json::Value::Null)
                    .get("large")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string(),
                base_data
                    .get("bannerImage")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string(),
                base_data
                    .get("characters")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string()
                    .replace("'", "''"),
                base_data
                    .get("trending")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string(),
                base_data
                    .get("staff")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string()
                    .replace("'", "''"),
                base_data
                    .get("siteUrl")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string()
                    .replace("'", "''"),
                base_data
                    .get("volumes")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string(),
                base_data
                    .get("chapters")
                    .unwrap_or(&serde_json::Value::Null)
                    .to_string(),
                "0".to_string(),
                path.to_string(),
                "false".to_string(),
            ];
            let columns_to_vec = columns
                .split(",")
                .map(|s| s.trim().to_string())
                .collect::<Vec<String>>();
            insert_into_db(&pool, "Series", Some(columns_to_vec), values_vec)
                .await
                .expect("Failed to insert into Series");
            let staff = staff_data.as_array().unwrap();
            for staff in staff {
                let values_vec = vec![
                    (staff.get("id").unwrap().to_string() + "_2"),
                    staff
                        .get("name")
                        .unwrap()
                        .get("full")
                        .unwrap()
                        .to_string()
                        .replace("'", "''"),
                    staff
                        .get("image")
                        .unwrap()
                        .get("medium")
                        .unwrap()
                        .to_string(),
                    staff
                        .get("description")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string()
                        .replace("'", "''"),
                    staff
                        .get("siteUrl")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string(),
                ];
                insert_into_db(&pool, "Creators", Option::None, values_vec)
                    .await
                    .expect("Failed to insert into Staff");
            }
            let characters = characters_data.as_array().unwrap();
            for character in characters {
                let values_vec = vec![
                    (character.get("id").unwrap().to_string() + "_2"),
                    character
                        .get("name")
                        .unwrap()
                        .get("full")
                        .unwrap()
                        .to_string()
                        .replace("'", "''"),
                    character
                        .get("image")
                        .unwrap()
                        .get("medium")
                        .unwrap()
                        .to_string(),
                    character
                        .get("description")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string()
                        .replace("'", "''"),
                    character
                        .get("siteUrl")
                        .unwrap_or(&serde_json::Value::Null)
                        .to_string(),
                ];
                insert_into_db(&pool, "Characters", Option::None, values_vec)
                    .await
                    .expect("Failed to insert into Characters");
            }
            let relations = relations_data.as_array().unwrap();
            for relation in relations {
                let values_vec = vec![
                    (relation.get("id").unwrap().to_string() + "_2"),
                    relation
                        .get("title")
                        .unwrap()
                        .get("english")
                        .unwrap_or(
                            relation
                                .get("title")
                                .unwrap()
                                .get("romaji")
                                .expect("No title found"),
                        )
                        .to_string()
                        .replace("'", "''"),
                    relation
                        .get("coverImage")
                        .unwrap()
                        .get("large")
                        .unwrap()
                        .to_string(),
                    (relation.get("type").unwrap().to_string()
                        + " / "
                        + &*relation.get("relationType").unwrap().to_string()
                        + " / "
                        + &*relation.get("format").unwrap().to_string())
                        .to_string()
                        .replace("'", "''"),
                    "null".to_string(),
                    (base_data.get("id").unwrap().to_string() + "_2"),
                ];
                insert_into_db(&pool, "relations", Option::None, values_vec)
                    .await
                    .expect("Failed to insert into relations");
            }
        }
        Err(e) => {
            error!("Error fetching data from Anilist API: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from Anilist API",
            )
                .into_response();
        }
    }
    (StatusCode::OK, "Series added successfully".to_string()).into_response()
}

pub async fn anilist_search(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path(name): axum::extract::Path<String>,
) -> impl IntoResponse {
    let name = name.clone();
    if name.is_empty() {
        return (StatusCode::BAD_REQUEST, "Name cannot be empty".to_string()).into_response();
    }

    match api_anilist_get_search(name.as_str()).await {
        Ok(response) => {
            info!("Anilist API search returned");
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Error fetching data from Anilist API: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching data from Anilist API",
            )
                .into_response()
        }
    }
}
