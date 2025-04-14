use std::{
    fs::{self},
    os::unix::fs::PermissionsExt,
    path::{self},
};

use pdfium_render::prelude::*;
use rand::Rng;
use sqlx::{Row, SqlitePool};

pub const VALID_BOOK_EXTENSION: &[&str] = &[
    "cbr", "cbz", "pdf", "zip", "7z", "cb7", "rar", "tar", "cbt", "epub", "ebook",
];
pub const VALID_IMAGE_EXTENSION: &[&str] = &[
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

pub(crate) fn generate_random_id() -> u32 {
    let mut rng = rand::thread_rng();
    let id: u32 = rng.gen_range(0..=u32::MAX);
    id
}