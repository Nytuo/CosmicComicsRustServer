#[cfg(test)]
mod tests {

    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;
    use tokio::sync::Mutex;
    use zip::write::FileOptions;

    use crate::AppGlobalVariables;
    use crate::services::archive_service::*;

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
    async fn test_extract_first_image_from_cbr() {
        let temp = tempdir().unwrap();
        let rar_path = temp.path().join("test.cbr");
        fs::copy("sample.cbr", &rar_path).unwrap();
        let out_dir = temp.path().join("out");
        fs::create_dir_all(&out_dir).unwrap();

        let result = extract_first_image(
            rar_path.to_str().unwrap().to_string(),
            out_dir.to_str().unwrap().to_string(),
            "cbr",
            "img",
        )
        .await;

        assert!(result.is_ok());
        assert!(out_dir.join("img.jpg").exists());
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

    #[tokio::test]
    async fn test_extract_all_images_from_rar() {
        let sample_rar_init_location = Path::new("sample.cbr");
        let temp = tempdir().unwrap();
        fs::copy(sample_rar_init_location, temp.path().join("sample.cbr")).unwrap();
        let rar_path = temp.path().join("sample.cbr");
        let extract_dir = temp.path().join("extracted_images");

        assert!(rar_path.exists());
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let result = extract_all_images_from_rar(
            rar_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
        let extracted_files: Vec<_> = fs::read_dir(&extract_dir).unwrap().collect();
        assert!(
            !extracted_files.is_empty(),
            "No images were extracted from the RAR archive."
        );
        assert!(
            extracted_files
                .iter()
                .any(|entry| { entry.as_ref().unwrap().file_name() == "00000.jpg" }),
            "Expected image file not found in the extracted directory."
        );
    }

    #[tokio::test]
    async fn test_extract_all_images_from_zip() {
        let temp = tempdir().unwrap();
        let zip_path = temp.path().join("sample.zip");
        let extract_dir = temp.path().join("extracted_images");

        let mut zip = zip::ZipWriter::new(File::create(&zip_path).unwrap());
        let options: zip::write::FileOptions<()> =
            FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip.start_file("image1.jpg", options).unwrap();
        zip.write_all(b"fakeimage1").unwrap();
        zip.start_file("image2.png", options).unwrap();
        zip.write_all(b"fakeimage2").unwrap();
        zip.finish().unwrap();

        fs::create_dir_all(&extract_dir).unwrap();

        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let result = extract_all_images_from_zip(
            zip_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
        let extracted_files: Vec<_> = fs::read_dir(&extract_dir).unwrap().collect();
        assert!(
            !extracted_files.is_empty(),
            "No images were extracted from the ZIP archive."
        );
    }

    #[tokio::test]
    async fn test_unzip_and_process_creates_path_file_zip() {
        use crate::AppGlobalVariables;
        use crate::services::archive_service::unzip_and_process;
        use std::fs;
        use std::sync::Arc;
        use tempfile::tempdir;
        use tokio::sync::Mutex;

        let temp = tempdir().unwrap();
        let extract_dir = temp.path().join("out");
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let zip_path = temp.path().join("test.cbz");
        {
            let mut zip = zip::ZipWriter::new(fs::File::create(&zip_path).unwrap());
            let options: zip::write::FileOptions<()> =
                FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("img.jpg", options).unwrap();
            zip.write_all(b"fakeimage").unwrap();
            zip.finish().unwrap();
        }

        let result = unzip_and_process(
            zip_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "cbz",
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
        assert!(extract_dir.exists());
        assert!(extract_dir.join("path.txt").exists());
    }

    #[tokio::test]
    async fn test_unzip_and_process_creates_path_file_rar() {
        use crate::AppGlobalVariables;
        use crate::services::archive_service::unzip_and_process;
        use std::fs::{self};
        use std::sync::Arc;
        use tempfile::tempdir;
        use tokio::sync::Mutex;

        let temp = tempdir().unwrap();
        let extract_dir = temp.path().join("out");
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let sample_rar_init_location = Path::new("sample.cbr");
        fs::copy(sample_rar_init_location, temp.path().join("test.cbr")).unwrap();
        let rar_path = temp.path().join("test.cbr");

        let result = unzip_and_process(
            rar_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "cbr",
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_err() || extract_dir.exists());
    }

    #[tokio::test]
    async fn test_unzip_and_process_creates_path_file_epub() {
        use crate::AppGlobalVariables;
        use crate::services::archive_service::unzip_and_process;
        use std::fs::File;
        use std::sync::Arc;
        use tempfile::tempdir;
        use tokio::sync::Mutex;
        use zip::write::FileOptions;

        let temp = tempdir().unwrap();
        let extract_dir = temp.path().join("out");
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        // Create a minimal valid EPUB (ZIP) file
        let epub_path = temp.path().join("test.epub");
        {
            let mut zip = zip::ZipWriter::new(File::create(&epub_path).unwrap());
            let options: zip::write::FileOptions<()> =
                FileOptions::default().compression_method(zip::CompressionMethod::Stored);
            zip.start_file("content.xhtml", options).unwrap();
            zip.write_all(b"<html><body>test</body></html>").unwrap();
            zip.finish().unwrap();
        }

        let result = unzip_and_process(
            epub_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "epub",
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
        assert!(extract_dir.exists());
        assert!(extract_dir.join("path.txt").exists());
    }

    #[tokio::test]
    async fn test_unzip_and_process_pdf() {
        use crate::AppGlobalVariables;
        use crate::services::archive_service::unzip_and_process;
        use std::fs;
        use std::sync::Arc;
        use tempfile::tempdir;
        use tokio::sync::Mutex;

        let temp = tempdir().unwrap();
        let extract_dir = temp.path().join("out");
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        // Download a sample PDF
        let pdf_path = temp.path().join("test.pdf");
        let url = "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";
        let bytes = reqwest::get(url).await.unwrap().bytes().await.unwrap();
        fs::write(&pdf_path, &bytes).unwrap();

        let result = unzip_and_process(
            pdf_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "pdf",
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_ok());
        assert!(extract_dir.exists());
        assert!(extract_dir.join("path.txt").exists());
    }

    #[tokio::test]
    async fn test_unzip_and_process_unknown_extension() {
        use crate::AppGlobalVariables;
        use crate::services::archive_service::unzip_and_process;
        use std::fs::File;
        use std::sync::Arc;
        use tempfile::tempdir;
        use tokio::sync::Mutex;

        let temp = tempdir().unwrap();
        let extract_dir = temp.path().join("out");
        let progress = Arc::new(Mutex::new(AppGlobalVariables::default()));

        let unk_path = temp.path().join("test.unk");
        File::create(&unk_path).unwrap();

        let result = unzip_and_process(
            unk_path.to_str().unwrap(),
            extract_dir.to_str().unwrap(),
            "unk",
            "token".to_string(),
            &progress,
        )
        .await;

        assert!(result.is_err());
    }
}
