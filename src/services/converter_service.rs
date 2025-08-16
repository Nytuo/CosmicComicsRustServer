use crate::repositories::database_repo::update_db;
use image::ImageReader;
use sqlx::SqlitePool;
use webp::Encoder;

fn convert_to_webp(input_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Converting {}", input_path);
    let reader = ImageReader::open(&input_path)?.with_guessed_format()?;
    let img = reader.decode()?;
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let encoder = Encoder::from_rgb(&rgb_img, width, height);
    let webp_data = encoder.encode(75.0);
    std::fs::write(output_path, &*webp_data)?;
    Ok(())
}
pub async fn convert_all_images_in_directory(
    dir_path: &str,
    output_dir: &str,
    valid_extensions: &[&str],
    db_pool: SqlitePool,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_dir = std::path::Path::new(dir_path);
    let output_dir = std::path::Path::new(output_dir);

    if !output_dir.exists() {
        std::fs::create_dir_all(output_dir)?;
    }

    for entry in std::fs::read_dir(input_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if valid_extensions.contains(&ext) {
                    let file_name = path.file_name().unwrap().to_str().unwrap();
                    let output_path = output_dir.join(format!("{}.webp", file_name));
                    convert_to_webp(path.to_str().unwrap(), output_path.to_str().unwrap())?;
                    let col_vec = vec!["URLCover".to_string()];
                    let val_vec = vec![output_path.to_str().unwrap().to_string()];
                    update_db(
                        &db_pool, "noedit", col_vec, val_vec, "Books", "ID_book", file_name,
                    )
                    .await?;
                }
            }
        }
    }
    Ok(())
}
