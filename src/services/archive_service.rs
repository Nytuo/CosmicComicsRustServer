use crate::{AppGlobalVariables, utils::is_image_file};
use futures::executor;
use headless_chrome::{Browser, LaunchOptionsBuilder};
use pdfium_render::prelude::*;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::sync::Mutex;
use unrar::Archive;
use zip::ZipArchive;

pub async fn unzip_and_process(
    zip_path: &str,
    extract_dir: &str,
    ext: &str,
    token: String,
    progress_status: &Arc<Mutex<AppGlobalVariables>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if Path::new(&extract_dir).exists() {
        fs::remove_dir_all(&extract_dir)?;
    }
    fs::create_dir_all(&extract_dir)?;

    let path_file = Path::new(&extract_dir).join("path.txt");
    let mut file = File::create(path_file)?;
    writeln!(file, "{}", zip_path)?;

    match ext {
        "zip" | "cbz" | "7z" | "cb7" | "tar" | "cbt" => {
            println!("Processing zip-based archive: {}", zip_path);
            extract_all_images_from_zip(zip_path, extract_dir, token, progress_status).await?;
        }

        "rar" | "cbr" => {
            println!("Processing rar-based archive: {}", zip_path);
            extract_all_images_from_rar(zip_path, extract_dir, token, progress_status).await?;
        }

        "pdf" => {
            println!("Processing PDF: {}", zip_path);
            convert_pdf_to_images(zip_path, extract_dir, token.clone(), progress_status).await?;
        }

        "epub" | "ebook" => {
            println!("Processing EPUB: {}", zip_path);
            extract_pdf_from_epub(zip_path, extract_dir, token.clone(), progress_status).await?;
        }

        _ => {
            println!("Unsupported extension: {}", ext);
            return Err(format!("Extension {} is not supported.", ext).into());
        }
    }

    Ok(())
}
pub async fn extract_first_image(
    zip_path: String,
    extract_dir: String,
    extension: &str,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match extension {
        "zip" | "cbz" | "7z" | "cb7" | "tar" | "cbt" => {
            extract_first_image_from_zip(zip_path, extract_dir, file_name)
        }
        "rar" | "cbr" => extract_first_image_from_rar(zip_path, extract_dir, file_name),
        _ => Err(format!("Unsupported extension: {}", extension).into()),
    }
}
fn extract_first_image_from_zip<P: AsRef<Path>>(
    zip_path: P,
    extract_dir: P,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut image_file_index = None;

    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if is_image_file(file.name()) {
            image_file_index = Some(i);
            break;
        }
    }

    if let Some(index) = image_file_index {
        let mut img_file = archive.by_index(index)?;
        let out_path = extract_dir.as_ref().join(format!("{}.jpg", file_name));
        let mut out_file = File::create(out_path)?;
        io::copy(&mut img_file, &mut out_file)?;
        println!("Image extracted from ZIP.");
        Ok(())
    } else {
        println!("No image file found in ZIP.");
        Ok(())
    }
}

fn extract_first_image_from_rar<P: AsRef<Path>>(
    rar_path: P,
    extract_dir: P,
    file_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut archive = Archive::new(rar_path.as_ref().to_str().unwrap()).open_for_processing()?;

    while let Some(header) = archive.read_header()? {
        let file_path = header.entry().filename.to_string_lossy().to_string();

        if header.entry().is_file() && is_image_file(&file_path) {
            println!("Found image: {}", file_path);

            archive = header.extract_to(&extract_dir)?;

            let extracted_file_path = extract_dir.as_ref().join(&*file_path);
            let renamed_path = extract_dir.as_ref().join(format!("{}.jpg", file_name));

            if extracted_file_path.exists() {
                fs::rename(&extracted_file_path, &renamed_path)?;
                println!("Extracted and renamed to: {:?}", renamed_path);
                return Ok(());
            } else {
                println!(
                    "Image file not found after extraction: {:?}",
                    extracted_file_path
                );
            }
        } else {
            archive = header.skip()?;
        }
    }

    println!("No image found in the archive.");
    Ok(())
}

pub async fn extract_all_images_from_zip<P: AsRef<Path>>(
    zip_path: P,
    extract_dir: P,
    token: String,
    progress_status: &Arc<Mutex<AppGlobalVariables>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut progress_status = progress_status.lock().await;
    let file = File::open(&zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    fs::create_dir_all(&extract_dir)?;
    let mut image_count = 0;
    let total_files = archive.len() as u32;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_name = file.name().to_string();

        if is_image_file(&file_name) {
            let out_path = extract_dir.as_ref().join(format!("{:05}.jpg", image_count));
            let mut out_file = File::create(&out_path)?;
            io::copy(&mut file, &mut out_file)?;
            fs::set_permissions(&out_path, fs::Permissions::from_mode(0o777))?;
            image_count += 1;
            progress_status.set_progress_status(
                token.clone(),
                "unzip".to_string(),
                "loading".to_string(),
                ((image_count * 100) / total_files as u32).to_string(),
                file_name,
            )
        }
    }

    progress_status.set_progress_status(
        token,
        "unzip".to_string(),
        "done".to_string(),
        "100".to_string(),
        "All images extracted.".to_string(),
    );

    if image_count == 0 {
        println!("No images found in ZIP archive.");
    } else {
        println!("Extracted {} images from ZIP archive.", image_count);
    }

    Ok(())
}

async fn extract_all_images_from_rar<P: AsRef<Path>>(
    rar_path: P,
    extract_dir: P,
    token: String,
    progress_status: &Arc<Mutex<AppGlobalVariables>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut progress_status = progress_status.lock().await;
    let mut archive = Archive::new(rar_path.as_ref().to_str().unwrap()).open_for_processing()?;
    fs::create_dir_all(&extract_dir)?;
    let mut image_count = 0;
    let mut total_files = 0;
    let archive_for_count = Archive::new(rar_path.as_ref().to_str().unwrap())
        .open_for_listing()
        .unwrap();
    for e in archive_for_count {
        let entry = e.unwrap();
        if entry.is_file() && is_image_file(&entry.filename.to_string_lossy()) {
            total_files += 1;
        }
    }

    while let Some(header) = archive.read_header()? {
        let file_path = header.entry().filename.to_string_lossy().to_string();

        if header.entry().is_file() && is_image_file(&file_path) {
            let extracted_file_path = extract_dir.as_ref().join(&file_path);
            archive = header.extract_to(&extract_dir)?;

            if extracted_file_path.exists() {
                let renamed_path = extract_dir.as_ref().join(format!("{:05}.jpg", image_count));
                fs::rename(&extracted_file_path, &renamed_path)?;
                fs::set_permissions(&renamed_path, fs::Permissions::from_mode(0o777))?;
                image_count += 1;
                progress_status.set_progress_status(
                    token.clone(),
                    "unzip".to_string(),
                    "loading".to_string(),
                    ((image_count * 100) / total_files as u32).to_string(),
                    file_path,
                );
            }
        } else {
            archive = header.skip()?;
        }
    }
    progress_status.set_progress_status(
        token,
        "unzip".to_string(),
        "done".to_string(),
        "100".to_string(),
        "All images extracted.".to_string(),
    );

    if image_count == 0 {
        println!("No images found in RAR archive.");
    } else {
        println!("Extracted {} images from RAR archive.", image_count);
    }

    Ok(())
}

pub async fn extract_pdf_from_epub(
    epub_path: &str,
    extract_dir: &str,
    token: String,
    progress_status: &Arc<Mutex<AppGlobalVariables>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::open(epub_path)?;
    let mut archive = ZipArchive::new(file)?;
    fs::create_dir_all(extract_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(extract_dir).join(file.name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(true)
            .sandbox(false)
            .build()
            .unwrap(),
    )?;
    let total_files = fs::read_dir(extract_dir)?
        .filter(|entry| {
            if let Ok(entry) = entry {
                if let Some(ext) = entry.path().extension() {
                    return ext == "xhtml";
                }
            }
            false
        })
        .count();
    let entries = fs::read_dir(extract_dir)?;
    let mut count = 0;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "xhtml") {
            let url = format!("file://{}", path.display());
            let tab = browser.new_tab()?;
            tab.navigate_to(&url)?;
            tab.wait_until_navigated()?;
            std::thread::sleep(Duration::from_millis(300));

            let pdf_data = tab.print_to_pdf(Default::default())?;
            let output_path = format!("{}/page_{}.pdf", extract_dir, count);
            fs::write(output_path, pdf_data)?;
            count += 1;
            progress_status.lock().await.set_progress_status(
                token.clone(),
                "unzip".to_string(),
                "Converting".to_string(),
                ((count * 100) / total_files as u32).to_string(),
                path.display().to_string(),
            );
        }
    }
    progress_status.lock().await.set_progress_status(
        token.clone(),
        "unzip".to_string(),
        "Merging".to_string(),
        ((count * 100) / total_files as u32).to_string(),
        "Merging PDF files".to_string(),
    );
    let output_pdf_path = format!("{}/output.pdf", extract_dir);

    merge_pdfs(
        (0..count)
            .map(|i| format!("{}/page_{}.pdf", extract_dir, i))
            .collect::<Vec<String>>()
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>(),
        &output_pdf_path,
    );

    //clean up the individual PDF files
    let entries = fs::read_dir(extract_dir)?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |e| e == "pdf") && !path.ends_with("output.pdf") {
            fs::remove_file(path)?;
        }
    }

    // Convert the merged PDF to images
    convert_pdf_to_images(
        &output_pdf_path,
        extract_dir,
        token.clone(),
        progress_status,
    )
    .await;
    fs::remove_file(output_pdf_path)?;
    Ok(())
}

pub async fn convert_pdf_to_images(
    pdf_path: &str,
    output_dir: &str,
    token: String,
    progress_status: &Arc<Mutex<AppGlobalVariables>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pdf_path = pdf_path.to_string();
    let output_dir = output_dir.to_string();
    let progress_status = Arc::clone(progress_status);

    tokio::task::spawn_blocking(move || {
        let pdfium = Pdfium::default();
        let pdf_path = Path::new(&pdf_path);
        if !pdf_path.exists() {
            return Err(format!("PDF file does not exist: {}", pdf_path.display()).into());
        }
        let doc = pdfium.load_pdf_from_file(pdf_path, None)?;

        std::fs::create_dir_all(&output_dir)?;
        let total_pages = doc.pages().len();

        for (i, page) in doc.pages().iter().enumerate() {
            let image = page
                .render_with_config(
                    &PdfRenderConfig::new()
                        .set_target_width(1200)
                        .render_form_data(true),
                )?
                .as_image()
                .into_rgb8();

            let file_path = format!("{}/page_{}.webp", output_dir, i);
            image.save_with_format(file_path, image::ImageFormat::WebP)?;

            let mut progress_status = executor::block_on(progress_status.lock());
            progress_status.set_progress_status(
                token.clone(),
                "unzip".to_string(),
                "loading".to_string(),
                ((i * 100) / total_pages as usize).to_string(),
                format!("page_{}", i),
            );
        }

        let mut progress_status = executor::block_on(progress_status.lock());
        progress_status.set_progress_status(
            token,
            "unzip".to_string(),
            "done".to_string(),
            "100".to_string(),
            "All pages rendered.".to_string(),
        );

        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
    .await?
}

fn merge_pdfs(
    input_paths: Vec<&str>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pdfium = Pdfium::default();
    let mut merged_doc = pdfium.create_new_pdf()?;

    for input_path in input_paths {
        let source_doc = pdfium.load_pdf_from_file(input_path, None)?;
        merged_doc.pages_mut().append(&source_doc)?;
    }

    let save_path = Path::new(output_path);
    merged_doc.save_to_file(save_path)?;

    println!("Merged PDF saved to: {}", output_path);
    Ok(())
}

pub async fn scrape_images_from_webpage(
    url: &str,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let browser = Browser::new(
        LaunchOptionsBuilder::default()
            .headless(true)
            .sandbox(false)
            .build()
            .unwrap(),
    )?;
    let tab = browser.new_tab()?;
    tab.navigate_to(url)?;
    tab.wait_until_navigated()?;
    std::thread::sleep(std::time::Duration::from_secs(2));

    let elements = tab.find_elements("img")?;
    let mut images = Vec::new();

    for element in elements {
        if let Ok(src) = element.call_js_fn("function() { return this.src; }", vec![], false) {
            if let Some(Value::String(src_str)) = src.value {
                images.push(src_str.to_string());
            }
        }
    }

    fs::create_dir_all(output_dir)?;
    for (i, img_url) in images.iter().enumerate() {
        let response = reqwest::get(img_url).await?;
        if response.status().is_success() {
            let mut file = fs::File::create(format!("{}/image_{}.jpg", output_dir, i))?;
            let bytes = response.bytes().await?;
            file.write_all(&bytes)?;
        } else {
            println!("Failed to download image: {}", img_url);
        }
    }

    println!("Scraped {} images from {}", images.len(), url);
    tab.close(true)?;
    Ok(())
}
