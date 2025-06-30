#[cfg(test)]
mod tests {
    use headless_chrome::Browser;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::Mutex;
    use zip::write::FileOptions;

    use crate::AppGlobalVariables;
    use crate::services::archive_service::*;
    use pdfium_render::prelude::*;

    fn create_test_cbz(path: &Path) {
        let file = File::create(path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options: zip::write::FileOptions<()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Stored);

        zip.start_file("img.jpg", options).unwrap();
        zip.write_all(b"fakeimage").unwrap();
        zip.finish().unwrap();
    }

    #[tokio::test]
    async fn test_extract_first_image_from_cbz() {
        let temp = tempdir().unwrap();
        let zip_path = temp.path().join("test.cbz");
        let out_dir = temp.path().join("out");
        create_test_cbz(&zip_path);
        fs::create_dir_all(&out_dir).unwrap();

        let result = extract_first_image(
            zip_path.to_str().unwrap().to_string(),
            out_dir.to_str().unwrap().to_string(),
            "cbz",
            "img",
        )
        .await;

        assert!(result.is_ok());
        assert!(out_dir.join("img.jpg").exists());
    }

    #[tokio::test]
    async fn test_extract_first_image_from_cbr_graceful_fail() {
        let temp = tempdir().unwrap();
        let rar_path = temp.path().join("test.cbr");
        File::create(&rar_path).unwrap(); // Not a real RAR
        let out_dir = temp.path().join("out");
        fs::create_dir_all(&out_dir).unwrap();

        let result = extract_first_image(
            rar_path.to_str().unwrap().to_string(),
            out_dir.to_str().unwrap().to_string(),
            "cbr",
            "img",
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_pdf_from_epub_minimal() {
        let temp = tempdir().unwrap();
        let epub_path = temp.path().join("test.epub");
        let mut zip = zip::ZipWriter::new(File::create(&epub_path).unwrap());
        let options: zip::write::FileOptions<()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("content.xhtml", options).unwrap();
        zip.write_all(b"<html><body>test</body></html>").unwrap();
        zip.finish().unwrap();

        let extract_dir = temp.path().join("out");
        fs::create_dir_all(&extract_dir).unwrap();

        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let result = extract_pdf_from_epub(
            epub_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_convert_pdf_to_images() {
        let temp = tempdir().unwrap();
        let pdf_path = temp.path().join("sample.pdf");

        let url = "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";
        let response = reqwest::get(url).await.unwrap();
        assert!(response.status().is_success());
        let bytes = response.bytes().await.unwrap();

        fs::write(&pdf_path, &bytes).unwrap();

        let output_dir = temp.path().join("output_images");
        fs::create_dir_all(&output_dir).unwrap();
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let result = convert_pdf_to_images(
            pdf_path.to_str().unwrap(),
            output_dir.to_str().unwrap(),
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
        let files: Vec<_> = fs::read_dir(&output_dir).unwrap().collect();
        assert!(!files.is_empty(), "No images were created from PDF.");
    }

    #[tokio::test]
    async fn test_scrape_images_from_webpage_basic() {
        let temp = tempdir().unwrap();
        let output_dir = temp.path().join("scraped");

        let url = "https://www.w3schools.com/html/html_images.asp"; // publicly safe test page
        let result = scrape_images_from_webpage(url, output_dir.to_str().unwrap()).await;

        assert!(result.is_ok());
        let files = fs::read_dir(&output_dir).unwrap().collect::<Vec<_>>();
        assert!(!files.is_empty());
    }
}
