use std::collections::HashMap;
use std::{fs, path};
use std::os::unix::fs::PermissionsExt;
use futures_util::future::ok;
use sqlx::Row;
use sqlx::SqlitePool;
use tower::util::Optional;
use crate::repositories::database_repo::update_db;

pub async fn get_books_with_blank_covers(
    db_pool: SqlitePool,
) -> Result<Vec<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let query =
        "select * from Books where URLCover IS NULL OR URLCover = 'null' OR URLCover='undefined';";
    let rows = sqlx::query(query).fetch_all(&db_pool).await?;

    let mut books = Vec::new();
    for row in rows {
        let mut book = HashMap::new();
        book.insert("ID_book".to_string(), row.get("ID_book"));
        book.insert("PATH".to_string(), row.get("PATH"));
        book.insert("NOM".to_string(), row.get("NOM"));
        books.push(book);
    }

    Ok(books)
}

pub async fn fill_blank_images(
    db_pool: SqlitePool,
    valid_image_extensions: &[&str],
    output_dir: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result: Vec<String> = Vec::new();

    let output_dir = if output_dir.is_some() {
        output_dir.unwrap()
    } else {
        format!("{}/public/FirstImagesOfAll", env!("CARGO_MANIFEST_DIR"))
    };
    let books = crate::services::book_service::get_books_with_blank_covers(db_pool.clone()).await?;
    for book in books {
        println!("Beginning fill_blank_images for: {}", book["NOM"]);

        let filename = book["ID_book"].clone();
        let path = book["PATH"].clone();

        if let Some(ext) = path::Path::new(path.clone().as_str())
            .extension()
            .and_then(|e| e.to_str())
        {
            let output_path = format!("{}/{}.jpg", output_dir, filename);

            fs::create_dir_all(&output_dir.clone())?;

            if let Err(e) = crate::services::archive_service::extract_first_image(
                path.clone(),
                output_dir.clone(),
                ext,
                &filename,
            )
                .await
            {
                println!("NOT SUPPORTED: {}", e);
                continue;
            }

            for entry in fs::read_dir(&output_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let permissions = fs::Permissions::from_mode(0o777);
                    fs::set_permissions(path, permissions)?;
                }
            }
            
            let col_vec = vec!["URLCover".to_string()];
            let val_vec = vec![output_path.clone()];

            update_db(
                &db_pool.clone(),
                "noedit",
                col_vec,
                val_vec,
                "Books",
                "ID_book",
                &filename,
            )
                .await?;
        }
    }

    println!("Converting images in directory: {}", output_dir);
    crate::services::converter_service::convert_all_images_in_directory(
        output_dir.clone().as_str(),
        &output_dir,
        &valid_image_extensions,
        db_pool,
    )
        .await?;
    println!("Finished fill_blank_images");
    Ok(())
}