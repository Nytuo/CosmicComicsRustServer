#[cfg(test)]
mod tests {
    use crate::services::book_service::{fill_blank_images, get_books_with_blank_covers};
    use sqlx::sqlite::SqlitePoolOptions;
    use std::fs;
    use std::io::Write;
    use tempfile::tempdir;
    use zip::write::FileOptions;

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

        let result = fill_blank_images(db.clone(), &valid_exts, Option::None).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fill_blank_images_with_valid_extensions() {
        let temp = tempdir().unwrap();
        let extract_dir = temp.path().join("out");

        let zip_path = temp.path().join("test.cbz");
        {
            let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
            let options: zip::write::FileOptions<()> =
                FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("img.jpg", options).unwrap();
            zip.write_all(b"fakeimage").unwrap();
            zip.finish().unwrap();
        }

        let db = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE Books (ID_book TEXT, PATH TEXT, NOM TEXT, URLCover TEXT);")
            .execute(&db)
            .await
            .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, PATH, NOM, URLCover) VALUES (?, ?, ?, ?);")
            .bind("2")
            .bind(zip_path.to_str().unwrap())
            .bind("Valid Book")
            .bind("null")
            .execute(&db)
            .await
            .unwrap();

        let valid_exts = ["jpg", "jpeg", "png"];

        let result = fill_blank_images(
            db.clone(),
            &valid_exts,
            Option::from(extract_dir.to_string_lossy().to_string()),
        )
        .await;

        assert!(result.is_ok());
    }
}
