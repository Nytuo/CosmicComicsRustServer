use std::path::Path;
use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

use crate::repositories::database_repo;

#[derive(Deserialize)]
pub struct CreateUserPayload {
    name: String,
    password: String,
    pp: Option<String>,
}

pub fn resolve_token(token: &str, cosmic_comics_temp: &str) -> Option<String> {
    let server_config_path = PathBuf::from(cosmic_comics_temp).join("serverconfig.json");

    let config_content = fs::read_to_string(&server_config_path).ok()?;
    let config: Value = serde_json::from_str(&config_content).ok()?;

    if let Some(tokens) = config.get("Token").and_then(|t| t.as_object()) {
        for (key, value) in tokens {
            if value.as_str() == Some(token) {
                return Some(key.clone());
            }
        }
    }

    None
}

pub async fn create_user_service(
    payload: &CreateUserPayload,
    base_path: &str,
) -> Result<(), String> {
    let user_dir = format!("{}/profiles/{}", base_path, payload.name);

    // Create user directory
    fs::create_dir_all(&user_dir).map_err(|e| format!("Failed to create user directory: {}", e))?;

    // Write passcode to file
    let passcode_path = format!("{}/passcode.txt", user_dir);
    fs::write(&passcode_path, payload.password.trim())
        .map_err(|e| format!("Failed to write passcode: {}", e))?;

    // Write default config if it doesn't exist
    let config_path = format!("{}/config.json", user_dir);
    if !Path::new(&config_path).exists() {
        let default_config = json!({
            "path": "",
            "last_opened": "",
            "language": "us",
            "update_provider": "",
            "ZoomLVL": 10,
            "Scroll_bar_visible": true,
            "Background_color": "rgb(33,33,33)",
            "WebToonMode": false,
            "Vertical_Reader_Mode": false,
            "Page_Counter": true,
            "SideBar": false,
            "NoBar": false,
            "SlideShow": false,
            "SlideShow_Time": 1,
            "Rotate_All": 0,
            "Margin": 0,
            "Manga_Mode": false,
            "No_Double_Page_For_Horizontal": false,
            "Blank_page_At_Begginning": false,
            "Double_Page_Mode": false,
            "Automatic_Background_Color": false,
            "magnifier_zoom": 1,
            "magnifier_Width": 100,
            "magnifier_Height": 100,
            "magnifier_Radius": 0,
            "reset_zoom": false,
            "force_update": false,
            "skip": false,
            "display_style": 0,
            "theme": "default.css",
            "theme_date": true
        });

        fs::write(
            &config_path,
            serde_json::to_string_pretty(&default_config).unwrap(),
        )
        .map_err(|e| format!("Failed to write config: {}", e))?;
    }

    // Handle profile picture
    let pp_path = format!("{}/pp.png", user_dir);
    if let Some(pp) = &payload.pp {
        fs::copy(pp, &pp_path).map_err(|e| format!("Failed to copy profile picture: {}", e))?;
    } else {
        let default_pp_dir = format!("{}/public/Images/account_default", base_path);
        let default_pp_files = fs::read_dir(&default_pp_dir)
            .map_err(|e| format!("Failed to read default profile pictures: {}", e))?;

        if let Some(random_pp) = default_pp_files
            .filter_map(|entry| entry.ok())
            .next()
            .map(|entry| entry.path())
        {
            fs::copy(random_pp, &pp_path)
                .map_err(|e| format!("Failed to copy default profile picture: {}", e))?;
        }
    }

    // Call make_db function
    database_repo::make_db(&payload.name, base_path)
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    Ok(())
}
