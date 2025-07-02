use std::sync::Arc;
use axum::extract::{State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::Value;
use sqlx::Row;
use crate::repositories::database_repo::get_db;
use crate::routes_manager::AppState;
use crate::services::collectionner_service::{get_list_of_files_and_folders, get_list_of_folders, handle_anilist_series, handle_google_book, handle_marvel_book, handle_marvel_series, handle_openlibrary_book};
use crate::services::googlebooks_service::search_gbapi_comics_by_name;
use crate::services::marvel_service::{get_marvel_api_characters, get_marvel_api_comics, get_marvel_api_creators};
use crate::services::openlibrary_service::{get_olapi_book, get_olapi_search};
use crate::services::profile_service::resolve_token;

#[derive(Deserialize)]
pub struct FillBlankImagePayload {
    token: String,
}

#[derive(Deserialize)]
pub struct InsertAnilistBookPayload {
    token: String,
    path: String,
    realname: String,
}

#[derive(Deserialize)]
pub struct InsertMarvelBookPayload {
    token: String,
    realname: String,
    date: String,
    path: String,
}

#[derive(serde::Deserialize)]
pub struct InsertGoogleBooksPayload {
    token: String,
    name: String,
    path: String,
}

#[derive(serde::Deserialize)]
pub struct InsertOLBookPayload {
    token: String,
    name: String,
    path: String,
}

pub async fn fill_blank_images_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Json(payload): Json<FillBlankImagePayload>,
) -> impl IntoResponse {
    let state = &state.lock().await;
    let config = state.config.lock().await;
    let global = state.global_vars.lock().await;

    let base_path = config.base_path.clone();
    let token = payload.token.clone();

    let pool = match crate::repositories::database_repo::get_db(
        &token,
        &base_path,
        global.opened_db.clone(),
    )
        .await
    {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Error getting database pool");
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    match crate::services::book_service::fill_blank_images(
        pool,
        crate::utils::VALID_IMAGE_EXTENSION,
        Option::None
    )
        .await
    {
        Ok(_) => {
            println!("Fill blank images completed successfully");
            StatusCode::OK
        }
        Err(e) => {
            eprintln!("Error filling blank images: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub async fn insert_anilist_book(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Json(payload): Json<InsertAnilistBookPayload>,
)-> impl IntoResponse {
    let token = payload.token.clone();
    let path = payload.path.clone();
    let realname = payload.realname.clone();

    let state = state.lock().await;
    let global = state.global_vars.lock().await;
    let base_path = state.config.lock().await.base_path.clone();

    let resolved_token = match resolve_token(&token, &base_path) {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let pool = match crate::repositories::database_repo::get_db(
        &resolved_token,
        &base_path,
        global.opened_db.clone(),
    )
        .await
    {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Error getting database pool");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let query = "SELECT title FROM Series;";
    let rows = match sqlx::query(query).fetch_all(&pool).await {
        Ok(rows) => rows,
        Err(err) => {
            eprintln!("Error executing query: {}", err);
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut series_name = String::new();
    for row in rows.iter() {
        let title: String = row.get("title");
        let el: Value = match serde_json::from_str(&title) {
            Ok(json) => json,
            Err(err) => {
                eprintln!("Error parsing JSON: {}", err);
                continue;
            }
        };

        for ele in path.split('/') {
            if ele == el["english"].as_str().unwrap_or_default()
                || ele == el["romaji"].as_str().unwrap_or_default()
                || ele == el["native"].as_str().unwrap_or_default()
            {
                series_name = el["english"]
                    .as_str()
                    .or_else(|| el["romaji"].as_str())
                    .or_else(|| el["native"].as_str())
                    .unwrap_or_default()
                    .to_string();
                break;
            }
        }

        if !series_name.is_empty() {
            break;
        }
    }

    let random_id = format!("{}_2", rand::random::<u32>());
    let insert_query = format!(
        "INSERT INTO Books VALUES ('{}', '{}', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}', NULL, NULL, NULL, NULL, NULL, NULL, '{}', NULL, NULL, NULL, NULL, NULL, NULL, NULL, false);",
        random_id,
        2,
        realname,
        path,
        format!("Anilist_{}_{}", realname.replace(" ", "$"), series_name.replace(" ", "$"))
    );

    match sqlx::query(&insert_query).execute(&pool).await {
        Ok(_) => {
            println!("Book inserted successfully");
            StatusCode::OK.into_response()
        }
        Err(err) => {
            eprintln!("Error inserting book: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn insert_marvel_book(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    headers: HeaderMap
) -> impl IntoResponse {
    let token = headers.get("token").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let realname = headers.get("realname").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let date = headers.get("date").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let path = headers.get("path").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();

    let state = state.lock().await;
    let base_path = state.config.lock().await.base_path.clone();
    let global = state.global_vars.lock().await;

    let pool = match crate::repositories::database_repo::get_db(
        &token,
        &base_path,
        global.opened_db.clone(),
    )
        .await
    {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Error getting database pool");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let marvel_pub_api_key = state.creds.lock().await.marvel_public_key.clone();
    let marvel_priv_api_key = state.creds.lock().await.marvel_private_key.clone();

    match get_marvel_api_comics(&realname, &date,&marvel_priv_api_key,&marvel_pub_api_key).await {
        Ok(cdata) => {
            let total = cdata["data"]["total"].as_u64().unwrap_or(0);
            if total > 0 {
                let comic = &cdata["data"]["results"][0];
                let thumbnail = format!(
                    "{}/detail.{}",
                    comic["thumbnail"]["path"].as_str().unwrap_or_default(),
                    comic["thumbnail"]["extension"].as_str().unwrap_or_default()
                );
                let description = comic["description"]
                    .as_str()
                    .unwrap_or("")
                    .replace("'", "''");
                let format = comic["format"].as_str().unwrap_or_default();
                let page_count = comic["pageCount"].as_u64().unwrap_or(0);

                let insert_query = format!(
                    "INSERT INTO Books VALUES ('{}_1', '1', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}', '{}', '{}', '{}', '{}', {}, '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', '{}', false);",
                    comic["id"].as_u64().unwrap_or(0),
                    realname,
                    path,
                    thumbnail,
                    comic["issueNumber"].as_u64().unwrap_or(0),
                    description,
                    format,
                    page_count,
                    comic["urls"].to_string(),
                    comic["series"].to_string(),
                    comic["creators"].to_string(),
                    comic["characters"].to_string(),
                    comic["prices"].to_string(),
                    comic["dates"].to_string(),
                    comic["collectedIssues"].to_string(),
                    comic["collections"].to_string(),
                    comic["variants"].to_string()
                );

                if let Err(err) = sqlx::query(&insert_query).execute(&pool).await {
                    eprintln!("Error inserting book: {}", err);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                if let Ok(creators) = get_marvel_api_creators(comic["id"].as_str().unwrap(), Option::from("comics"), &*marvel_priv_api_key, &*marvel_pub_api_key).await {
                    for creator in creators["data"]["results"].as_array().unwrap_or(&vec![]) {
                        let creator_query = format!(
                            "INSERT INTO Creators VALUES ('{}_1', '{}', '{}', NULL, '{}');",
                            creator["id"].as_u64().unwrap_or(0),
                            creator["fullName"].as_str().unwrap_or("").replace("'", "''"),
                            creator["thumbnail"].to_string(),
                            creator["urls"].to_string()
                        );

                        if let Err(err) = sqlx::query(&creator_query).execute(&pool).await {
                            eprintln!("Error inserting creator: {}", err);
                        }
                    }
                }

                if let Ok(characters) = get_marvel_api_characters(comic["id"].as_str().unwrap(), Option::from("comics"), &*marvel_priv_api_key, &*marvel_pub_api_key).await {
                    for character in characters["data"]["results"].as_array().unwrap_or(&vec![]) {
                        let character_query = format!(
                            "INSERT INTO Characters VALUES ('{}_1', '{}', '{}', '{}', '{}');",
                            character["id"].as_u64().unwrap_or(0),
                            character["name"].as_str().unwrap_or("").replace("'", "''"),
                            character["thumbnail"].to_string(),
                            character["description"].as_str().unwrap_or("").replace("'", "''"),
                            character["urls"].to_string()
                        );

                        if let Err(err) = sqlx::query(&character_query).execute(&pool).await {
                            eprintln!("Error inserting character: {}", err);
                        }
                    }
                }
            } else {
                let random_id = format!("{}_1", rand::random::<u32>());
                let default_query = format!(
                    "INSERT INTO Books VALUES ('{}', '1', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, false);",
                    random_id,
                    realname,
                    path
                );

                if let Err(err) = sqlx::query(&default_query).execute(&pool).await {
                    eprintln!("Error inserting default book: {}", err);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
            let response = serde_json::to_string(&cdata).unwrap_or_default();
            (StatusCode::OK, response).into_response()

        }
        Err(err) => {
            eprintln!("Error fetching Marvel API data: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        _ => {
            eprintln!("Unexpected error occurred");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn insert_googlebooks_book(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    headers: HeaderMap
) -> impl IntoResponse {
    let token = headers.get("token").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let realname = headers.get("name").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let path = headers.get("path").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();

    let state = state.lock().await;
    let base_path = state.config.lock().await.base_path.clone();
    let global = state.global_vars.lock().await;

    let pool = match crate::repositories::database_repo::get_db(
        &token,
        &base_path,
        global.opened_db.clone(),
    )
        .await
    {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Error getting database pool");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let google_books_api_key = state.creds.lock().await.google_books_api_key.clone();

    match search_gbapi_comics_by_name(&realname,google_books_api_key).await {
        Ok(cdata) => {
            let total_items = cdata["totalItems"].as_u64().unwrap_or(0);
            if total_items > 0 {
                let book = &cdata["items"][0];
                let thumbnail = book["volumeInfo"]["imageLinks"]
                    .as_object()
                    .and_then(|links| links.get("large").or_else(|| links.get("thumbnail")))
                    .and_then(|link| link.as_str())
                    .unwrap_or_default();
                let description = book["volumeInfo"]["description"]
                    .as_str()
                    .unwrap_or("")
                    .replace("'", "''");
                let print_type = book["volumeInfo"]["printType"].as_str().unwrap_or_default();
                let page_count = book["volumeInfo"]["pageCount"].as_u64().unwrap_or(0);
                let info_link = book["volumeInfo"]["infoLink"].to_string();
                let authors = book["volumeInfo"]["authors"].as_array().unwrap_or(&vec![])
                    .iter()
                    .map(|a| a.as_str().unwrap_or("").to_string())
                    .collect::<Vec<String>>();
                let retail_price = book["saleInfo"]["retailPrice"]
                    .as_object()
                    .and_then(|price| price.get("amount"))
                    .and_then(|amount| amount.as_f64())
                    .unwrap_or(0.0);
                let published_date = book["volumeInfo"]["publishedDate"].to_string();

                let insert_query = format!(
                    "INSERT INTO Books VALUES ('{}_4', '4', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}','{}', NULL, '{}', '{}', {}, '{}', NULL, '{}', NULL, {}, '{}', NULL, NULL, NULL, false);",
                    book["id"].as_str().unwrap_or_default(),
                    realname,
                    path,
                    thumbnail,
                    description,
                    print_type,
                    page_count,
                    info_link,
                    authors.join(", ").replace("'", "''"),
                    retail_price,
                    published_date
                );

                if let Err(err) = sqlx::query(&insert_query).execute(&pool).await {
                    eprintln!("Error inserting book: {}", err);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }

                for author in authors {
                    let author_query = format!(
                        "INSERT INTO Creators VALUES ('{}_4', '{}', NULL, NULL, NULL);",
                        book["id"].as_str().unwrap_or_default(),
                        author.replace("'", "''")
                    );

                    if let Err(err) = sqlx::query(&author_query).execute(&pool).await {
                        eprintln!("Error inserting author: {}", err);
                    }
                }
            } else {
                let random_id = format!("{}_4", rand::random::<u32>());
                let default_query = format!(
                    "INSERT INTO Books VALUES ('{}', '4', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, false);",
                    random_id,
                    realname,
                    path
                );

                if let Err(err) = sqlx::query(&default_query).execute(&pool).await {
                    eprintln!("Error inserting default book: {}", err);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
            let response = serde_json::to_string(&cdata).unwrap_or_default();
            (StatusCode::OK, response).into_response()
        }
        Err(err) => {
            eprintln!("Error fetching Google Books API data: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        _ => {
            eprintln!("Unexpected error occurred");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn insert_olib_book(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    headers: HeaderMap
) -> impl IntoResponse {
    let token = headers.get("token").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let realname = headers.get("name").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();
    let path = headers.get("path").and_then(|h| h.to_str().ok()).unwrap_or_default().to_string();

    let state = state.lock().await;
    let base_path = state.config.lock().await.base_path.clone();
    let global = state.global_vars.lock().await;

    let pool = match crate::repositories::database_repo::get_db(
        &token,
        &base_path,
        global.opened_db.clone(),
    )
        .await
    {
        Ok(pool) => pool,
        Err(_) => {
            eprintln!("Error getting database pool");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match get_olapi_search(&realname).await {
        Ok(cdata) => {
            let num_found = cdata.num_found;
            if num_found > 0 {
                let key = if let Some(doc) = cdata.docs.first() {
                    if let Some(key) = &doc.key {
                        key.as_str()
                            .split('/')
                            .nth(2)
                            .unwrap_or_default()
                    } else {
                        ""
                    }
                } else {
                    ""
                };

                match get_olapi_book(key).await {
                    Ok(book) => {
                        let first_child = book.as_object().unwrap().keys().next().unwrap();
                        let book_details = &book[first_child]["details"];
                        let thumbnail_url = book[first_child]["thumbnail_url"]
                            .as_str()
                            .map(|url| url.replace("-S", "-L"))
                            .unwrap_or_default();
                        let description = book_details["description"]
                            .as_str()
                            .unwrap_or("")
                            .replace("'", "''");
                        let physical_format = book_details["physical_format"]
                            .as_str()
                            .unwrap_or_default();
                        let number_of_pages = book_details["number_of_pages"]
                            .as_u64()
                            .unwrap_or(0);
                        let info_url = book_details["info_url"].to_string();
                        let publish_date = book_details["publish_date"].to_string();
                        let authors = book_details["authors"]
                            .as_array()
                            .unwrap_or(&vec![])
                            .iter()
                            .map(|a| a.to_string())
                            .collect::<Vec<String>>();

                        let insert_query = format!(
                            "INSERT INTO Books VALUES ('{}_3', '3', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}', '{}', NULL, '{}', '{}', {}, '{}', NULL, '{}', NULL, NULL, '{}', NULL, NULL, NULL, false);",
                            book[first_child]["bib_key"].as_str().unwrap_or_default(),
                            realname,
                            path,
                            thumbnail_url,
                            description,
                            physical_format,
                            number_of_pages,
                            info_url,
                            authors.join(", ").replace("'", "''"),
                            publish_date
                        );

                        if let Err(err) = sqlx::query(&insert_query).execute(&pool).await {
                            eprintln!("Error inserting book: {}", err);
                            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                        }

                        for author in authors {
                            let author_query = format!(
                                "INSERT INTO Creators VALUES ('{}_3', '{}', NULL, NULL, NULL);",
                                book[first_child]["bib_key"].as_str().unwrap_or_default(),
                                author.replace("'", "''")
                            );

                            if let Err(err) = sqlx::query(&author_query).execute(&pool).await {
                                eprintln!("Error inserting author: {}", err);
                            }
                        }
                        let response = serde_json::to_string(&book).unwrap_or_default();
                        (StatusCode::OK, response).into_response()
                    }
                    Err(err) => {
                        eprintln!("Error fetching book details: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                    _ => {
                        eprintln!("Unexpected error occurred while fetching book details");
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                }
            } else {
                let random_id = format!("{}_3", rand::random::<u32>());
                let default_query = format!(
                    "INSERT INTO Books VALUES ('{}', '3', '{}', NULL, 0, 0, 1, 0, 0, 0, '{}', NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, false);",
                    random_id,
                    realname,
                    path
                );

                if let Err(err) = sqlx::query(&default_query).execute(&pool).await {
                    eprintln!("Error inserting default book: {}", err);
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
                let response = serde_json::to_string(&cdata).unwrap_or_default();
                (StatusCode::OK, response).into_response()
            }
        }
        Err(err) => {
            eprintln!("Error fetching Open Library API data: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        _ => {
            eprintln!("Unexpected error occurred");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct RefreshMetaPayload {
    pub token: String,
    pub provider: i32,
    #[serde(rename = "type")]
    pub item_type: String,
    pub id: String,
}

pub async fn refresh_meta_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    Json(payload): Json<RefreshMetaPayload>,
) -> impl IntoResponse {
    let state = state.lock().await;
    let config = state.config.lock().await;
    let global = state.global_vars.lock().await;

    let base_path = &config.base_path;
    let resolved_token = match resolve_token(&payload.token, base_path) {
        Some(t) => t,
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    let pool = match get_db(&resolved_token, base_path, global.opened_db.clone()).await {
        Ok(p) => p,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    match payload.provider {
        1 => {
            let marvel_pub_api_key = state.creds.lock().await.marvel_public_key.clone();
            let marvel_priv_api_key = state.creds.lock().await.marvel_private_key.clone();
            if payload.item_type == "book" {
                if let Err(e) = handle_marvel_book(&pool, &payload.id, payload.provider, &payload.token, marvel_priv_api_key.clone(), marvel_pub_api_key.clone()).await {
                    eprintln!("Error handling Marvel book: {}", e);
                }
            } else {
                if let Err(e) = handle_marvel_series(&pool, &payload.id, payload.provider, &payload.token, marvel_priv_api_key.clone(), marvel_pub_api_key.clone()).await{
                    eprintln!("Error handling Marvel series: {}", e);
                }
            }
        }
        2 => {
            if payload.item_type != "book" {
                if let Err(e) = handle_anilist_series(&pool, &payload.id, payload.provider, &payload.token).await {
                    eprintln!("Error handling Anilist series: {}", e);
                }
            }
        }
        3 => {
            if let Err(e) = handle_openlibrary_book(&pool, &payload.id, payload.provider, &payload.token).await {
                eprintln!("Error handling OpenLibrary book: {}", e);
            }
        }
        4 => {
            if let Err(e) = handle_google_book(&pool, &payload.id, payload.provider, &payload.token).await {
                eprintln!("Error handling Google Book: {}", e);
            }
        }
        _ => return StatusCode::BAD_REQUEST.into_response(),
    }

    StatusCode::OK.into_response()
}

pub async fn get_list_of_files_and_folders_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let dir = crate::utils::replace_html_address_path(&path);
    match get_list_of_files_and_folders(dir).await {
        Ok(response) => {
            println!("List of files and folders fetched successfully");
            (StatusCode::OK, response).into_response()
        }
        Err(e) => {
            eprintln!("Error fetching list of files and folders: {}", e);
            let error_response =
                serde_json::json!({"error": "Error fetching list of files and folders"});
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

pub async fn get_list_of_folders_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let dir = crate::utils::replace_html_address_path(&path);
    match get_list_of_folders(dir).await {
        Ok(response) => (StatusCode::OK, response).into_response(),
        Err(e) => {
            eprintln!("Error fetching list of folders: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error fetching list of folders",
            )
                .into_response()
        }
    }
}

pub async fn scrape_images_from_webpage_controller(
    State(state): State<Arc<tokio::sync::Mutex<AppState>>>,
    axum::extract::Json(payload): axum::extract::Json<Value>,
) -> impl IntoResponse {
    let dir = state.lock().await.config.lock().await.base_path.clone();
    let url = payload
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    match crate::services::archive_service::scrape_images_from_webpage(&*url, &*dir).await {
        Ok(response) => (StatusCode::OK, response).into_response(),
        Err(e) => {
            eprintln!("Error scraping images from webpage: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error scraping images from webpage",
            )
                .into_response()
        }
    }
}