#[cfg(test)]
mod tests {
    use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use tempfile::tempdir;

    use crate::services::converter_service::convert_all_images_in_directory;

    fn create_test_image(path: &Path) {
        let img = image::RgbImage::new(100, 100);
        img.save(path).unwrap();
    }

    #[tokio::test]
    async fn test_convert_all_images_in_directory() {
        let input_dir = tempdir().unwrap();
        let output_dir = tempdir().unwrap();
        let img_path = input_dir.path().join("test_image.jpg");

        create_test_image(&img_path);

        let db_url = "sqlite::memory:";
        let db_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(db_url)
            .await
            .expect("Failed to create DB");

        sqlx::query("CREATE TABLE Books (ID_book TEXT, URLCover TEXT);")
            .execute(&db_pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO Books (ID_book, URLCover) VALUES ('test_image.jpg', '');")
            .execute(&db_pool)
            .await
            .unwrap();

        let result = convert_all_images_in_directory(
            input_dir.path().to_str().unwrap(),
            output_dir.path().to_str().unwrap(),
            &["jpg"],
            db_pool.clone(),
        )
        .await;

        assert!(result.is_ok());

        let webp_path = output_dir.path().join("test_image.jpg.webp");
        assert!(webp_path.exists());
    }
}
