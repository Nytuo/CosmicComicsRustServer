use std::{
    fs::{self},
    os::unix::fs::PermissionsExt,
    path::{self},
};

use pdfium_render::prelude::*;
use sqlx::{Row, SqlitePool};

use crate::repositories::database_repo::update_db;

const VALID_BOOK_EXTENSION: &[&str] = &[
    "cbr", "cbz", "pdf", "zip", "7z", "cb7", "rar", "tar", "cbt", "epub", "ebook",
];
const VALID_IMAGE_EXTENSION: &[&str] = &[
    "png", "jpg", "jpeg", "bmp", "apng", "svg", "ico", "webp", "gif", "tiff",
];

pub fn replace_html_address_path(path: &str) -> String {
    path.replace("%20", " ")
        .replace("ù", "/")
        .replace("%C3%B9", "/")
        .replace("%23", "#")
}
pub fn get_element_from_info_path(
    search: &str,
    info: &std::collections::HashMap<String, String>,
) -> Option<String> {
    info.get(search).cloned()
}
pub fn get_list_of_images(dir_path: &std::path::Path, valid_extensions: &[&str]) -> Vec<String> {
    if let Ok(entries) = std::fs::read_dir(dir_path) {
        let mut list_of_images = Vec::new();
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if valid_extensions.contains(&ext) {
                    if let Some(file_name) = entry.file_name().to_str() {
                        list_of_images.push(file_name.to_string());
                    }
                } else {
                    println!(
                        "{} has a non-compatible viewer extension: {}",
                        entry.path().display(),
                        ext
                    );
                }
            }
        }
        list_of_images
    } else {
        Vec::new()
    }
}

async fn fill_blank_images(
    db_pool: SqlitePool,
    valid_image_extensions: Vec<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let result: Vec<String> = Vec::new();

    let output_dir = format!("{}/public/FirstImagesOfAll", env!("CARGO_MANIFEST_DIR"));
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

            update_db(
                &db_pool.clone(),
                "noedit",
                Vec::from(["URLCover"]),
                Vec::from([output_path.as_str()]),
                "Books",
                "ID_book",
                &filename,
            )
            .await?;
        }
    }

    crate::services::converter_service::convert_all_images_in_directory(
        output_dir.clone().as_str(),
        &output_dir,
        &valid_image_extensions,
        db_pool,
    )
    .await?;

    Ok(())
}

pub fn is_image_file(name: &str) -> bool {
    VALID_IMAGE_EXTENSION
        .iter()
        .any(|ext| name.to_lowercase().ends_with(ext))
}

pub fn is_light_color(r: u8, g: u8, b: u8) -> bool {
    let brightness = (r as f32 * 0.299) + (g as f32 * 0.587) + (b as f32 * 0.114);
    brightness > 186.0
}
pub fn darken_color(r: u8, g: u8, b: u8) -> String {
    let factor = 0.7;
    let darken = |c: u8| (c as f32 * factor).max(0.0) as u8;
    format!("rgb({},{},{})", darken(r), darken(g), darken(b))
}
