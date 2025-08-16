use rand::Rng;
use tracing::error;

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
                    error!(
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

pub(crate) fn generate_random_id() -> u32 {
    let mut rng = rand::rng();
    let id: u32 = rng.random_range(0..=u32::MAX);
    id
}

pub fn strip_outer_quotes(s: &str) -> &str {
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        &s[1..s.len() - 1]
    } else {
        s
    }
}
