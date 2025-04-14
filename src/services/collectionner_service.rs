use std::{fs, io};
use std::sync::Arc;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::IntoResponse;
use chrono::Date;
use serde::Deserialize;
use serde_json::json;
use sqlx::{Row, SqlitePool};
use crate::repositories::database_repo::update_db;
use crate::routes_manager::AppState;
use crate::services::anilist_service::api_anilist_get_by_id;
use crate::services::googlebooks_service::get_gbapi_comics_by_id;
use crate::services::marvel_service::{get_marvel_api_comics_by_id, get_marvel_api_series_by_id};
use crate::services::openlibrary_service::{get_olapi_book, get_olapi_comics_by_id, get_olapi_search};

pub async fn handle_marvel_book(pool: &SqlitePool, id: &str, provider: i32, token: &str,marvel_priv_key: String, marvel_pub_key: String) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT * FROM Books WHERE ID_book = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    let book_path: String = row.try_get("PATH")?;
    let book_id: String = row.try_get("ID_book")?;
    let api_id = book_id.split('_').next().unwrap_or("");

    let result = get_marvel_api_comics_by_id(api_id, &*marvel_priv_key, &*marvel_pub_key)
        .await.map_err(|e| sqlx::Error::Protocol(format!("Marvel API error: {}", e)))?;

    let mut asso = serde_json::Map::new();
    asso.insert("NOM".to_string(), json!(result.title));
    asso.insert("URLCover".to_string(), json!(format!("{}/detail.{}", result.thumbnail.path, result.thumbnail.extension)));
    asso.insert("issueNumber".to_string(), json!(result.issueNumber));
    asso.insert("description".to_string(), json!(result.description.clone().unwrap_or_default().replace("'", "''")));
    asso.insert("format".to_string(), json!(result.format));
    asso.insert("pageCount".to_string(), json!(result.pageCount));
    asso.insert("URLs".to_string(), json!(result.urls));
    asso.insert("dates".to_string(), json!(result.dates));
    asso.insert("prices".to_string(), json!(result.prices));
    asso.insert("creators".to_string(), json!(result.creators));
    asso.insert("characters".to_string(), json!(result.characters));
    asso.insert("series".to_string(), json!(result.series));
    asso.insert("collectedIssues".to_string(), json!(result.collectedIssues));
    asso.insert("variants".to_string(), json!(result.variants));
    asso.insert("collections".to_string(), json!(result.collections));
    asso.insert("API_ID".to_string(), json!(provider));

    let columns = asso.keys().cloned().collect::<Vec<String>>();
    let values = asso.values().cloned().map(|v| v.to_string()).collect::<Vec<String>>();

    update_db(pool,"edit", columns, values, "Books", "PATH", &book_path).await
}

pub async fn handle_marvel_series(pool: &SqlitePool, id: &str, provider: i32, token: &str,marvel_priv_key: String, marvel_pub_key: String) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT * FROM Series WHERE ID_Series = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    let path: String = row.try_get("PATH")?;
    let res2 = get_marvel_api_series_by_id(id, &*marvel_priv_key, &*marvel_pub_key).await.map_err(|e| sqlx::Error::Protocol(format!("Marvel API error: {}", e)))?;

    let mut asso = serde_json::Map::new();
    asso.insert("title".to_string(), json!(res2.title.replace("'", "''")));
    asso.insert("cover".to_string(), json!(res2.thumbnail));
    asso.insert("description".to_string(), json!(res2.description.clone().unwrap_or_default().replace("'", "''")));
    asso.insert("start_date".to_string(), json!(res2.startYear));
    asso.insert("end_date".to_string(), json!(res2.endYear));
    asso.insert("CHARACTERS".to_string(), json!(res2.characters));
    asso.insert("STAFF".to_string(), json!(res2.creators));
    asso.insert("SOURCE".to_string(), json!(res2.urls.get(0)));
    asso.insert("BG".to_string(), json!(res2.thumbnail));
    asso.insert("volumes".to_string(), json!(res2.comics.items));
    asso.insert("chapters".to_string(), json!(res2.comics.available));
    asso.insert("API_ID".to_string(), json!(provider));

    let columns = asso.keys().cloned().collect::<Vec<String>>();
    let values = asso.values().cloned().map(|v| v.to_string()).collect::<Vec<String>>();

    update_db(pool,"edit", columns,values , "Series", "PATH", &path).await
}

pub async fn handle_anilist_series(pool: &SqlitePool, id: &str, provider: i32, token: &str) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT * FROM Series WHERE ID_Series = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    let path: String = row.try_get("PATH")?;
    let res2 = api_anilist_get_by_id(id).await.map_err(|e| sqlx::Error::Protocol(format!("Anilist API error: {}", e)))?;
    let result = res2.unwrap_or_else(|| {
        panic!("No media found for ID: {}", id);
    });

    let mut asso = serde_json::Map::new();
    asso.insert("title".to_string(), json!(result.title));
    asso.insert("cover".to_string(), json!(result.cover_image.unwrap().large));
    asso.insert("description".to_string(), json!(result.description.unwrap_or_default().replace("'", "''")));
    asso.insert("start_date".to_string(), json!(result.start_date));
    asso.insert("end_date".to_string(), json!(result.end_date));
    asso.insert("CHARACTERS".to_string(), json!(result.characters));
    asso.insert("STAFF".to_string(), json!(result.staff));
    asso.insert("SOURCE".to_string(), json!(result.site_url));
    asso.insert("BG".to_string(), json!(result.banner_image));
    asso.insert("volumes".to_string(), json!(result.volumes));
    asso.insert("chapters".to_string(), json!(result.chapters));
    asso.insert("statut".to_string(), json!(result.status));
    asso.insert("Score".to_string(), json!(result.mean_score));
    asso.insert("genres".to_string(), json!(result.genres));
    asso.insert("TRENDING".to_string(), json!(result.trending));
    asso.insert("API_ID".to_string(), json!(provider));
    
    let columns = asso.keys().cloned().collect::<Vec<String>>();
    let values = asso.values().cloned().map(|v| v.to_string()).collect::<Vec<String>>();

    update_db(pool,"edit", columns, values, "Series", "PATH", &path).await
}

pub async fn handle_openlibrary_book(pool: &SqlitePool, id: &str, provider: i32, token: &str) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT * FROM Books WHERE ID_book = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    let path: String = row.try_get("PATH")?;
    let res = get_olapi_comics_by_id(id).await.map_err(|e| sqlx::Error::Protocol(format!("Open Library API error: {}", e)))?;
    let details = res.details;

    let mut asso = serde_json::Map::new();
    let cover_ol = get_olapi_search(&details.title).await
        .map_err(|e| sqlx::Error::Protocol(format!("Open Library search error: {}", e)))?
        .docs
        .first()
        .and_then(|doc| doc.cover_i)
        .unwrap_or(0);
    let fallback_cover = format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_ol);

    asso.insert("NOM".to_string(), json!(details.title));
    asso.insert("URLCover".to_string(), json!(res.thumbnail_url.clone().unwrap_or(fallback_cover).replace("-S", "-L")));
    asso.insert("API_ID".to_string(), json!(provider));
    asso.insert("issueNumber".to_string(), json!("null"));
    asso.insert("description".to_string(), json!(details.description.clone().unwrap_or("null".to_string()).replace("'", "''")));
    asso.insert("format".to_string(), json!(details.physical_format));
    asso.insert("pageCount".to_string(), json!(details.number_of_pages));
    asso.insert("URLs".to_string(), json!(details.info_url));
    asso.insert("dates".to_string(), json!(details.publish_date));
    asso.insert("prices".to_string(), json!("null"));
    asso.insert("creators".to_string(), json!(details.authors));
    asso.insert("characters".to_string(), json!("null"));
    asso.insert("series".to_string(), json!("null"));
    asso.insert("collectedIssues".to_string(), json!("null"));
    asso.insert("variants".to_string(), json!("null"));
    asso.insert("collections".to_string(), json!("null"));
    
    let columns = asso.keys().cloned().collect::<Vec<String>>();
    let values = asso.values().cloned().map(|v| v.to_string()).collect::<Vec<String>>();

    update_db(pool,"edit", columns, values, "Books", "PATH", &path).await
}

pub async fn handle_google_book(pool: &SqlitePool, id: &str, provider: i32, token: &str) -> Result<(), sqlx::Error> {
    let row = sqlx::query("SELECT * FROM Books WHERE ID_book = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;
    let path: String = row.try_get("PATH")?;
    let res = get_gbapi_comics_by_id(id).await.map_err(|e| sqlx::Error::Protocol(format!("Google Books API error: {}", e)))?;

    let mut asso = serde_json::Map::new();
    let price = res.saleInfo.unwrap().retailPrice.map(|p| p.amount).unwrap_or(0.0);
    let cover = res.volumeInfo.imageLinks.as_ref().and_then(|links| {
        links.large.clone().or_else(|| links.thumbnail.clone())
    });

    asso.insert("NOM".to_string(), json!(res.volumeInfo.title));
    asso.insert("API_ID".to_string(), json!(provider));
    asso.insert("URLCover".to_string(), json!(cover));
    asso.insert("issueNumber".to_string(), json!("null"));
    asso.insert("description".to_string(), json!(res.volumeInfo.description.unwrap_or_default().replace("'", "''")));
    asso.insert("format".to_string(), json!(res.volumeInfo.printType));
    asso.insert("pageCount".to_string(), json!(res.volumeInfo.pageCount));
    asso.insert("URLs".to_string(), json!(res.volumeInfo.infoLink));
    asso.insert("dates".to_string(), json!(res.volumeInfo.publishedDate));
    asso.insert("prices".to_string(), json!(price));
    asso.insert("creators".to_string(), json!(res.volumeInfo.authors));
    asso.insert("characters".to_string(), json!("null"));
    asso.insert("series".to_string(), json!("null"));
    asso.insert("collectedIssues".to_string(), json!("null"));
    asso.insert("variants".to_string(), json!("null"));
    asso.insert("collections".to_string(), json!("null"));
    
    let columns = asso.keys().cloned().collect::<Vec<String>>();
    let values = asso.values().cloned().map(|v| v.to_string()).collect::<Vec<String>>();

    update_db(pool,"edit", columns, values, "Books", "PATH", &path).await
}

pub async fn get_list_of_files_and_folders(dir: String) -> Result<Json<serde_json::Value>, io::Error> {
    let mut result = Vec::new();

    let entries = fs::read_dir(&dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::metadata(&path)?;
        if metadata.is_dir() {
            result.push(path.to_string_lossy().to_string());
        } else {
            result.push(path.to_string_lossy().to_string());
        }
    }

    Ok(Json(json!(result)))
}

pub async fn get_list_of_folders(dir: String) -> Result<Json<serde_json::Value>, io::Error> {
    let mut list_of_folders = Vec::new();

    let entries = fs::read_dir(&dir)?;
    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();
        let metadata = fs::metadata(&file_path)?;
        if metadata.is_dir() {
            list_of_folders.push(file_path.to_string_lossy().to_string());
        }
    }

    Ok(Json(json!(list_of_folders)))
}