use sqlx::sqlite::{SqlitePool, SqliteValueRef};
use sqlx::{Column, Executor, Row, TypeInfo, ValueRef, query};
use std::collections::HashMap;
use std::fs;
use std::iter::repeat;
use std::path::Path;

use crate::utils::strip_outer_quotes;

pub async fn make_db(profile_owner: &str, base_path: &str) -> Result<(), sqlx::Error> {
    let db_path = format!("{}/profiles/{}/CosmicComics.db", base_path, profile_owner);
    let db_dir = Path::new(&db_path).parent().unwrap();
    fs::create_dir_all(db_dir).expect("Failed to create database directory");
    fs::write(
        format!("{}/profiles/{}/CosmicComics.db", base_path, profile_owner),
        "",
    )
    .expect("Failed to create database file");
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;
    let mut conn = pool.acquire().await?;
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Books (
            ID_book TEXT PRIMARY KEY NOT NULL,
            API_ID TEXT,
            NOM TEXT NOT NULL,
            note INTEGER,
            read BOOLEAN NOT NULL,
            reading BOOLEAN NOT NULL,
            unread BOOLEAN NOT NULL,
            favorite BOOLEAN NOT NULL,
            last_page INTEGER NOT NULL,
            folder BOOLEAN NOT NULL,
            PATH TEXT NOT NULL,
            URLCover TEXT,
            issueNumber INTEGER,
            description TEXT,
            format TEXT,
            pageCount INTEGER,
            URLs TEXT,
            series TEXT,
            creators TEXT,
            characters TEXT,
            prices TEXT,
            dates TEXT,
            collectedIssues TEXT,
            collections TEXT,
            variants TEXT,
            lock BOOLEAN DEFAULT false NOT NULL
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Bookmarks (
            ID_BOOKMARK INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            BOOK_ID TEXT NOT NULL,
            PATH TEXT NOT NULL,
            page INTEGER NOT NULL,
            FOREIGN KEY (BOOK_ID) REFERENCES Books (ID_book)
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS API (
            ID_API TEXT PRIMARY KEY NOT NULL,
            NOM TEXT NOT NULL
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        INSERT OR REPLACE INTO API (ID_API, NOM)
        VALUES
            ('1', 'Marvel'),
            ('2', 'Anilist'),
            ('4', 'Google Books API'),
            ('3', 'OpenLibrary'),
            ('0', 'MANUAL');
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Series (
            ID_Series TEXT PRIMARY KEY NOT NULL UNIQUE,
            title TEXT NOT NULL,
            note INTEGER,
            statut TEXT,
            start_date TEXT,
            end_date TEXT,
            description TEXT,
            Score INTEGER,
            genres TEXT,
            cover TEXT,
            BG TEXT,
            CHARACTERS TEXT,
            TRENDING INTEGER,
            STAFF TEXT,
            SOURCE TEXT,
            volumes INTEGER,
            chapters INTEGER,
            favorite BOOLEAN NOT NULL,
            PATH TEXT NOT NULL,
            lock BOOLEAN DEFAULT false NOT NULL
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Creators (
            ID_CREATOR TEXT PRIMARY KEY NOT NULL UNIQUE,
            name TEXT,
            image TEXT,
            description TEXT,
            url TEXT
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Characters (
            ID_CHAR TEXT PRIMARY KEY NOT NULL UNIQUE,
            name TEXT,
            image TEXT,
            description TEXT,
            url TEXT
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS variants (
            ID_variant TEXT PRIMARY KEY NOT NULL UNIQUE,
            name TEXT,
            image TEXT,
            url TEXT,
            series TEXT,
            FOREIGN KEY (series) REFERENCES Series (ID_Series)
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS relations (
            ID_variant TEXT PRIMARY KEY NOT NULL UNIQUE,
            name TEXT,
            image TEXT,
            description TEXT,
            url TEXT,
            series TEXT,
            FOREIGN KEY (series) REFERENCES Series (ID_Series)
        );
        "#,
    )
    .await?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS Libraries (
            ID_LIBRARY INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
            NAME TEXT NOT NULL,
            PATH TEXT NOT NULL,
            API_ID TEXT NOT NULL,
            FOREIGN KEY (API_ID) REFERENCES API (ID_API)
        );
        "#,
    )
    .await?;

    // Set the PRAGMA user_version
    let version = env!("CARGO_PKG_VERSION").replace('.', "");
    conn.execute(format!("PRAGMA user_version = {};", version).as_str())
        .await?;

    Ok(())
}

pub async fn get_db(
    forwho: &str,
    base_path: &str,
    mut opened_db: HashMap<String, SqlitePool>,
) -> Result<SqlitePool, sqlx::Error> {
    if let Some(pool) = opened_db.get(forwho) {
        return Ok(pool.clone());
    }

    let db_path = format!("{}/profiles/{}/CosmicComics.db", base_path, forwho);
    let pool = SqlitePool::connect(&format!("sqlite://{}", db_path)).await?;
    opened_db.insert(forwho.to_string(), pool.clone());
    Ok(pool)
}

pub async fn update_db(
    db_pool: &SqlitePool,
    update_type: &str,
    columns: Vec<String>,
    values: Vec<String>,
    table: &str,
    condition_column: &str,
    condition_value: &str,
) -> Result<(), sqlx::Error> {
    if update_type == "edit" {
        let mut updates = Vec::new();
        for (i, column) in columns.iter().enumerate() {
            let value = values[i].replace("'", "''").replace("\"", "\\\"");
            updates.push(format!("{} = '{}'", column, value));
        }
        let update_query = format!(
            "UPDATE {} SET {} WHERE {} = '{}';",
            table,
            updates.join(", "),
            condition_column,
            condition_value
        );
        println!("Executing query: {}", update_query);
        if let Err(e) = query(&update_query).execute(db_pool).await {
            eprintln!("Update query failed: {}", e);
            return Err(e);
        }
    } else {
        let update_query = format!(
            "UPDATE {} SET {} = '{}' WHERE {} = '{}';",
            table, columns[0], values[0], condition_column, condition_value
        );
        println!("Executing query: {}", update_query);
        query(&update_query).execute(db_pool).await?;
        println!("Updated {} in {} table", condition_value, table);
    }
    Ok(())
}

pub async fn insert_into_db(
    db_pool: &SqlitePool,
    table: &str,
    columns: Option<Vec<String>>,
    values: Vec<String>,
) -> Result<(), sqlx::Error> {
    let column_names = if let Some(cols) = columns {
        format!("({})", cols.join(", "))
    } else {
        String::new()
    };
    let placeholders = repeat("?")
        .take(values.len())
        .collect::<Vec<_>>()
        .join(", ");
    let insert_query = format!(
        "INSERT OR IGNORE INTO {} {} VALUES ({});",
        table, column_names, placeholders
    );
    println!("Executing query: {}", insert_query);
    let mut query_builder = query(&insert_query);
    for value in values {
        println!("{}", value);
        query_builder = query_builder.bind(value);
    }
    query_builder.execute(db_pool).await?;
    Ok(())
}

pub async fn select_from_db(
    db_pool: &SqlitePool,
    table: &str,
    columns: Vec<String>,
    condition_column: Option<Vec<&str>>,
    condition_value: Option<Vec<&str>>,
    condition_separator: Option<&str>,
) -> Result<Vec<HashMap<String, String>>, sqlx::Error> {
    let column_names = if columns.is_empty() {
        "*"
    } else {
        &*columns.join(", ")
    };
    let mut query_str = format!("SELECT {} FROM {}", column_names, table);
    if let (Some(cols), Some(vals), Some(separator)) =
        (condition_column, condition_value, condition_separator)
    {
        let conditions: Vec<String> = cols
            .iter()
            .zip(vals.iter())
            .map(|(col, val)| format!("{} = '{}'", col, val))
            .collect();
        query_str.push_str(&format!(
            " WHERE {}",
            conditions.join(&format!(" {} ", separator))
        ));
    }
    println!("Executing query: {}", query_str);
    let rows = query(&query_str).fetch_all(db_pool).await?;
    let mut results = Vec::new();
    for row in rows {
        let mut row_map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let raw_value = row.try_get_raw(i)?;
            let value_str = if raw_value.is_null() {
                "NULL".to_string()
            } else {
                match raw_value.type_info().name() {
                    "INTEGER" => row.try_get::<i64, _>(i).map(|v| v.to_string()),
                    "TEXT" => row.try_get::<String, _>(i),
                    "BOOLEAN" => row.try_get::<bool, _>(i).map(|v| v.to_string()),
                    "REAL" => row.try_get::<f64, _>(i).map(|v| v.to_string()),
                    _ => Ok("<unsupported>".to_string()),
                }
                .unwrap_or_else(|_| "<error>".to_string())
            };

            row_map.insert(column.name().to_string(), value_str);
        }
        results.push(row_map);
    }
    Ok(results)
}

pub async fn select_from_db_with_options(
    db_pool: &SqlitePool,
    option: &str,
) -> Result<Vec<HashMap<String, String>>, sqlx::Error> {
    let mut query_str = format!("SELECT {};", option);
    println!("Executing query: {}", query_str);
    let rows = query(&query_str).fetch_all(db_pool).await?;
    let mut results = Vec::new();
    for row in rows {
        let mut row_map = HashMap::new();
        for (i, column) in row.columns().iter().enumerate() {
            let raw_value = row.try_get_raw(i)?;
            let value_str = if raw_value.is_null() {
                "NULL".to_string()
            } else {
                match raw_value.type_info().name() {
                    "INTEGER" => row.try_get::<i64, _>(i).map(|v| v.to_string()),
                    "TEXT" => row.try_get::<String, _>(i),
                    "BOOLEAN" => row.try_get::<bool, _>(i).map(|v| v.to_string()),
                    "REAL" => row.try_get::<f64, _>(i).map(|v| v.to_string()),
                    _ => Ok("<unsupported>".to_string()),
                }
                .unwrap_or_else(|_| "<error>".to_string())
            };

            row_map.insert(
                column.name().to_string(),
                strip_outer_quotes(&value_str[..]).to_string(),
            );
        }
        results.push(row_map);
    }
    Ok(results)
}

pub async fn delete_from_db(
    db_pool: &SqlitePool,
    table: &str,
    condition_column: &str,
    condition_value: &str,
    option: Option<&str>,
) -> Result<(), sqlx::Error> {
    let delete_query = format!(
        "DELETE FROM {} WHERE {} = '{}' {};",
        table,
        condition_column,
        condition_value,
        option.unwrap_or("?")
    );
    println!("Executing query: {}", delete_query);
    query(&delete_query).execute(db_pool).await?;
    Ok(())
}
