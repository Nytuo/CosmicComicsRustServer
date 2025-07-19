#[cfg(test)]
mod tests {
    use axum::Json;
    use dotenv::dotenv;
    use serde_json::Value;
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_get_list_of_files_and_folders() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");

        File::create(&file_path)
            .unwrap()
            .write_all(b"hello")
            .unwrap();

        let result = get_list_of_files_and_folders(dir.path().to_str().unwrap().to_string())
            .await
            .unwrap();

        let json = result.0;
        assert!(json.is_array());
        assert!(
            json.as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str().unwrap().contains("test.txt"))
        );
    }

    #[tokio::test]
    async fn test_get_list_of_folders() {
        let dir = tempdir().unwrap();
        let sub_dir = dir.path().join("subfolder");
        create_dir_all(&sub_dir).unwrap();

        let result = get_list_of_folders(dir.path().to_str().unwrap().to_string())
            .await
            .unwrap();

        let json = result.0;
        assert!(json.is_array());
        assert!(
            json.as_array()
                .unwrap()
                .iter()
                .any(|v| v.as_str().unwrap().contains("subfolder"))
        );
    }
    use sqlx::{Column, Row, SqlitePool, sqlite::SqlitePoolOptions};
    use std::env;

    use crate::services::collectionner_service::{
        get_list_of_files_and_folders, get_list_of_folders, handle_anilist_series,
        handle_google_book, handle_marvel_book, handle_marvel_series, handle_openlibrary_book,
    };

    /* #[tokio::test]
    async fn test_handle_marvel_book_success() {
        dotenv().ok();
        let public_key = std::env::var("MARVEL_PUBLIC_KEY").expect("Set MARVEL_PUBLIC_KEY");
        let private_key = std::env::var("MARVEL_PRIVATE_KEY").expect("Set MARVEL_PRIVATE_KEY");

        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE Books (ID_book TEXT, PATH TEXT);")
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, PATH) VALUES (?, ?);")
            .bind("82967_test") // Comic ID + suffix
            .bind("82967_test")
            .execute(&db)
            .await
            .unwrap();

        let result = handle_marvel_book(
            &db,
            "82967_test",
            3, // e.g., Marvel provider ID
            "dummy_token",
            private_key,
            public_key,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_handle_marvel_series_success() {
        dotenv().ok();

        let public_key = std::env::var("MARVEL_PUBLIC_KEY").expect("Set MARVEL_PUBLIC_KEY");
        let private_key = std::env::var("MARVEL_PRIVATE_KEY").expect("Set MARVEL_PRIVATE_KEY");

        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE Series (ID_Series TEXT, PATH TEXT);")
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO Series (ID_Series, PATH) VALUES (?, ?);")
            .bind("22551") // Marvel series ID
            .bind("22551")
            .execute(&db)
            .await
            .unwrap();

        let result =
            handle_marvel_series(&db, "22551", 3, "dummy_token", private_key, public_key).await;

        assert!(result.is_ok());
    } */

    #[tokio::test]
    async fn test_handle_anilist_series_writes_to_db() {
        let pool = SqlitePoolOptions::new().connect(":memory:").await.unwrap();

        sqlx::query("CREATE TABLE IF NOT EXISTS Series (ID_Series TEXT PRIMARY KEY NOT NULL UNIQUE,title TEXT NOT NULL,note INTEGER,statut TEXT,start_date TEXT,end_date TEXT,description TEXT,Score INTEGER,genres TEXT,cover TEXT,BG TEXT,CHARACTERS TEXT,TRENDING INTEGER,STAFF TEXT,SOURCE TEXT,volumes INTEGER,chapters INTEGER,favorite BOOLEAN NOT NULL,PATH TEXT NOT NULL,lock BOOLEAN DEFAULT false NOT NULL);").execute(&pool).await.unwrap();

        sqlx::query("INSERT INTO Series (ID_Series, title, note, statut, start_date, end_date, description, Score, genres, cover, BG, CHARACTERS, TRENDING, STAFF, SOURCE, volumes, chapters, favorite, PATH) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
    .bind("30042")
    .bind("Test Title")
    .bind(0)
    .bind("ongoing")
    .bind("2024-01-01")
    .bind("2024-12-31")
    .bind("Test description")
    .bind(0)
    .bind("action,adventure")
    .bind("cover_url")
    .bind("bg_url")
    .bind("characters_json")
    .bind(0)
    .bind("staff_json")
    .bind("original")
    .bind(1)
    .bind(10)
    .bind(false)
    .bind("series/path")
    .execute(&pool)
    .await
    .unwrap();

        let result = handle_anilist_series(&pool, "30042", 2, "dummy_token").await;
        assert!(result.is_ok());

        let row = sqlx::query("SELECT title, description FROM Series WHERE ID_Series = ?")
            .bind("30042")
            .fetch_one(&pool)
            .await
            .unwrap();

        let title: String = row.get("title");
        let description: String = row.get("description");

        assert!(!title.is_empty());
        assert!(!description.is_empty());
    }

    #[tokio::test]
    async fn test_handle_googlebooks_book_writes_to_db() {
        let pool = SqlitePoolOptions::new().connect(":memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS Books (
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
        );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, API_ID, NOM, note, read, reading, unread, favorite, last_page,
            folder, PATH, URLCover, issueNumber, description, format, pageCount, URLs, series, creators, characters, prices, dates,
            collectedIssues, collections, variants, lock) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
    .bind("ltKXEAAAQBAJ")
    .bind("api_ltKXEAAAQBAJ")
    .bind("Test Title")
    .bind(0)
    .bind(false)
    .bind(false)
    .bind(true)
    .bind(false)
    .bind(0)
    .bind(false)
    .bind("series/path")
    .bind("cover_url")
    .bind(1)
    .bind("Test description")
    .bind("original")
    .bind(10)
    .bind("urls_json")
    .bind("series_name")
    .bind("creators_json")
    .bind("characters_json")
    .bind("prices_json")
    .bind("dates_json")
    .bind("collected_issues_json")
    .bind("collections_json")
    .bind("variants_json")
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

        let result = handle_google_book(&pool, "ltKXEAAAQBAJ", 2, "dummy_token").await;
        assert!(result.is_ok());

        let row = sqlx::query("SELECT NOM, description FROM Books WHERE ID_book = ?")
            .bind("ltKXEAAAQBAJ")
            .fetch_one(&pool)
            .await
            .unwrap();

        let title: String = row.get("NOM");
        let description: String = row.get("description");

        assert!(!title.is_empty());
        assert!(!description.is_empty());
    }

    #[tokio::test]
    async fn test_handle_openlibrary_book_writes_to_db() {
        let pool = SqlitePoolOptions::new().connect(":memory:").await.unwrap();

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS Books (
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
        );",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, API_ID, NOM, note, read, reading, unread, favorite, last_page,
            folder, PATH, URLCover, issueNumber, description, format, pageCount, URLs, series, creators, characters, prices, dates,
            collectedIssues, collections, variants, lock) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)")
    .bind("OL43511705M")
    .bind("api_OL43511705M")
    .bind("Test Title")
    .bind(0)
    .bind(false)
    .bind(false)
    .bind(true)
    .bind(false)
    .bind(0)
    .bind(false)
    .bind("series/path")
    .bind("cover_url")
    .bind(1)
    .bind("Test description")
    .bind("original")
    .bind(10)
    .bind("urls_json")
    .bind("series_name")
    .bind("creators_json")
    .bind("characters_json")
    .bind("prices_json")
    .bind("dates_json")
    .bind("collected_issues_json")
    .bind("collections_json")
    .bind("variants_json")
    .bind(false)
    .execute(&pool)
    .await
    .unwrap();

        let result = handle_openlibrary_book(&pool, "OL43511705M", 2, "dummy_token").await;
        assert!(result.is_ok());

        let row = sqlx::query("SELECT NOM, description FROM Books WHERE ID_book = ?")
            .bind("OL43511705M")
            .fetch_one(&pool)
            .await
            .unwrap();

        let title: String = row.get("NOM");
        let description: String = row.get("description");

        assert!(!title.is_empty());
        assert!(!description.is_empty());
    }
}
