#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
    use std::collections::HashMap;

    use crate::services::book_service::{fill_blank_images, get_books_with_blank_covers};

    #[tokio::test]
    async fn test_get_books_with_blank_covers_returns_expected() {
        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE Books (ID_book TEXT, PATH TEXT, NOM TEXT, URLCover TEXT);")
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, PATH, NOM, URLCover) VALUES (?, ?, ?, ?);")
            .bind("1")
            .bind("/some/path.cbz")
            .bind("Test Book")
            .bind("null")
            .execute(&db)
            .await
            .unwrap();

        let result = get_books_with_blank_covers(db.clone()).await.unwrap();

        assert_eq!(result.len(), 1);
        let book = &result[0];
        assert_eq!(book.get("ID_book").unwrap(), "1");
        assert_eq!(book.get("NOM").unwrap(), "Test Book");
    }

    #[tokio::test]
    async fn test_fill_blank_images_does_not_crash() {
        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE Books (ID_book TEXT, PATH TEXT, NOM TEXT, URLCover TEXT);")
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, PATH, NOM, URLCover) VALUES (?, ?, ?, ?);")
            .bind("1")
            .bind("/non/existing/path.cbz")
            .bind("Fail Book")
            .bind("null")
            .execute(&db)
            .await
            .unwrap();

        let valid_exts = ["jpg", "jpeg", "png"];

        let result = fill_blank_images(db.clone(), &valid_exts).await;
        assert!(result.is_ok());
    }
}
