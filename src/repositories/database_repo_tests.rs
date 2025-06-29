use sqlx::{Column, Executor, Row, query};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::tempdir;
    use sqlx::{SqlitePool, Row};
    use crate::repositories::database_repo::{delete_from_db, get_db, insert_into_db, make_db, select_from_db, select_from_db_with_options, update_db};

    fn get_test_paths() -> (String, String) {
        let dir = tempdir().unwrap();
        let base_path = dir.path().to_str().unwrap().to_string();
        let profile = "test_user".to_string();
        (base_path, profile)
    }

    async fn setup_db(base_path: &str, profile: &str) -> SqlitePool {
        make_db(profile, base_path).await.unwrap();
        let db_path = format!("{}/profiles/{}/CosmicComics.db", base_path, profile);
        SqlitePool::connect(&format!("sqlite://{}", db_path)).await.unwrap()
    }

    #[tokio::test]
    async fn test_make_db_creates_tables() {
        let (base_path, profile) = get_test_paths();
        let result = make_db(&profile, &base_path).await;
        assert!(result.is_ok());
        let db_path = format!("{}/profiles/{}/CosmicComics.db", base_path, profile);
        assert!(Path::new(&db_path).exists());
    }

    #[tokio::test]
    async fn test_get_db_opens_connection() {
        let (base_path, profile) = get_test_paths();
        make_db(&profile, &base_path).await.unwrap();
        let mut opened = HashMap::new();
        let pool = get_db(&profile, &base_path, opened.clone()).await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master")
            .fetch_one(&pool).await.unwrap();
        assert!(row.0 > 0);
    }

    #[tokio::test]
    async fn test_insert_and_select_from_db() {
        let (base_path, profile) = get_test_paths();
        let pool = setup_db(&base_path, &profile).await;

        insert_into_db(
            &pool,
            "API",
            Some(vec!["ID_API".into(), "NOM".into()]),
            vec!["999".into(), "TestAPI".into()]
        ).await.unwrap();

        let results = select_from_db(
            &pool,
            "API",
            vec!["ID_API".into(), "NOM".into()],
            Some(vec!["ID_API"]),
            Some(vec!["999"]),
            Some("AND")
        ).await.unwrap();

        assert_eq!(results.len(), 1);
        let id_api: i64 = results[0]["ID_API"].parse().unwrap();
        let nom: &str = &results[0]["NOM"];

        assert_eq!(id_api, 999);
        assert_eq!(nom, "TestAPI");
    }

    #[tokio::test]
    async fn test_update_db_modifies_data() {
        let (base_path, profile) = get_test_paths();
        let pool = setup_db(&base_path, &profile).await;

        insert_into_db(
            &pool,
            "API",
            Some(vec!["ID_API".into(), "NOM".into()]),
            vec!["998".into(), "OldName".into()]
        ).await.unwrap();

        update_db(
            &pool,
            "edit",
            vec!["NOM".into()],
            vec!["NewName".into()],
            "API",
            "ID_API",
            "998"
        ).await.unwrap();

        let results = select_from_db(
            &pool,
            "API",
            vec!["NOM".into()],
            Some(vec!["ID_API"]),
            Some(vec!["998"]),
            Some("AND")
        ).await.unwrap();

        assert_eq!(results[0]["NOM"], "NewName");
    }

    #[tokio::test]
    async fn test_delete_from_db_removes_data() {
        let (base_path, profile) = get_test_paths();
        let pool = setup_db(&base_path, &profile).await;

        insert_into_db(
            &pool,
            "API",
            Some(vec!["ID_API".into(), "NOM".into()]),
            vec!["997".into(), "ToDelete".into()]
        ).await.unwrap();

        delete_from_db(
            &pool,
            "API",
            "ID_API",
            "997",
            Some("")
        ).await.unwrap();

        let results = select_from_db(
            &pool,
            "API",
            vec!["ID_API".into()],
            Some(vec!["ID_API"]),
            Some(vec!["997"]),
            Some("AND")
        ).await.unwrap();

        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_select_from_db_with_options_executes_custom_query() {
        let (base_path, profile) = get_test_paths();
        let pool = setup_db(&base_path, &profile).await;

        let results = select_from_db_with_options(
            &pool,
            "name FROM sqlite_master WHERE type='table' AND name='API'"
        ).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0]["name"], "API");
    }
}