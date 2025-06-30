use std::path::Path;
use std::{fs, path::PathBuf};

use serde::Deserialize;
use serde_json::Value;
use serde_json::json;

use crate::repositories::database_repo;

#[derive(Deserialize)]
pub struct CreateUserPayload {
    pub(crate) name: String,
    pub(crate) password: String,
    pub(crate) pp: Option<String>,
}

impl CreateUserPayload {
    pub fn new(name: String, password: String, pp: Option<String>) -> Self {
        CreateUserPayload { name, password, pp }
    }
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

pub fn generate_token(name: &str, cosmic_comics_temp: &str, user_dir: &str) -> String {
    use rand::Rng;
    use rand::distr::Alphanumeric;

    let mut rng = rand::thread_rng();
    let token: String = (0..32).map(|_| rng.sample(Alphanumeric) as char).collect();
    let config_path = format!("{}/serverconfig.json", cosmic_comics_temp);
    let mut config: Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    let token_object = config
        .get_mut("Token")
        .and_then(|t| t.as_object_mut())
        .unwrap();
    token_object.insert(name.to_string(), json!(token.clone()));
    fs::write(config_path, serde_json::to_string_pretty(&config).unwrap())
        .expect("Failed to write serverconfig.json");
    let book_path = format!("{}/current_book", user_dir);
    fs::create_dir_all(&book_path).expect("Failed to create book path");
    token
}

pub async fn create_user_service(
    payload: &CreateUserPayload,
    base_path: &str,
) -> Result<(), String> {
    let user_dir = format!("{}/profiles/{}", base_path, payload.name);
    fs::create_dir_all(&user_dir).map_err(|e| format!("Failed to create user directory: {}", e))?;
    let passcode_path = format!("{}/passcode.txt", user_dir);
    fs::write(&passcode_path, payload.password.trim())
        .map_err(|e| format!("Failed to write passcode: {}", e))?;
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

    let pp_path = format!("{}/pp.png", user_dir);
    if let Some(pp) = &payload.pp {
        if fs::copy(pp, &pp_path).is_err() {
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

    database_repo::make_db(&payload.name, base_path)
        .await
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    Ok(())
}
pub async fn delete_account_service(
    token: &str,
    base_path: &str,
    global: &mut crate::AppGlobalVariables,
) -> Result<(), String> {
    let pool = match crate::repositories::database_repo::get_db(
        &token,
        &base_path,
        global.opened_db.clone(),
    )
    .await
    {
        Ok(pool) => pool,
        Err(_) => return Err("Failed to get database connection".to_string()),
    };

    pool.close().await;

    if global.opened_db.remove(token).is_none() {
        eprintln!("Failed to remove DB from openedDB");
    }

    let user_dir = format!("{}/profiles/{}", base_path, token);
    if fs::remove_dir_all(&user_dir).is_err() {
        eprintln!("Failed to delete user directory: {}", user_dir);
        return Err("Failed to delete user directory".to_string());
    }

    Ok(())
}

pub async fn modify_profile_service(
    token: &str,
    new_pass: Option<&str>,
    new_pp: Option<&str>,
    new_user: Option<&str>,
    base_path: &str,
) -> Result<(), String> {
    if let Some(new_pass) = new_pass {
        fs::write(
            format!("{}/profiles/{}/passcode.txt", base_path, token),
            new_pass.trim(),
        )
        .map_err(|e| {
            eprintln!("Failed to write passcode: {}", e);
            "Failed to write passcode".to_string()
        })?;
    }

    if let Some(new_pp) = new_pp {
        let regex =
            regex::Regex::new(r"^(https?://)?([a-zA-Z0-9.-]+|localhost)(:[0-9]+)?").unwrap();
        let new_pp_path = regex
            .replace(new_pp, format!("{}/public", base_path).as_str())
            .to_string();
        if let Err(e) = fs::copy(
            new_pp_path,
            format!("{}/profiles/{}/pp.png", base_path, token),
        ) {
            eprintln!("Failed to copy profile picture: {}", e);
            return Err("Failed to copy profile picture".to_string());
        }
    }

    if let Some(new_user) = new_user {
        if let Err(e) = fs::rename(
            format!("{}/profiles/{}", base_path, token),
            format!("{}/profiles/{}", base_path, new_user),
        ) {
            eprintln!("Failed to rename user directory: {}", e);
            return Err("Failed to rename user directory".to_string());
        }
    }

    Ok(())
}

pub async fn login_service(name: &str, passcode: &str, base_path: &str) -> Result<String, String> {
    let user_dir = format!("{}/profiles/{}", base_path, name);
    let passcode_path = format!("{}/passcode.txt", user_dir);

    if !Path::new(&user_dir).exists() {
        return Err("User does not exist".to_string());
    }

    let stored_passcode = fs::read_to_string(&passcode_path)
        .map_err(|e| format!("Failed to read passcode: {}", e))?;

    if stored_passcode.trim() != passcode {
        return Err("Invalid passcode".to_string());
    }

    let token = generate_token(name, base_path, user_dir.as_str());

    Ok(token)
}

pub async fn login_check_service(token: &str, base_path: &str) -> Result<String, String> {
    let config_path = format!("{}/serverconfig.json", base_path);
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read serverconfig.json: {}", e))?;
    let config: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse serverconfig.json: {}", e))?;

    if let Some(tokens) = config.get("Token").and_then(|t| t.as_object()) {
        for (key, value) in tokens {
            if value.as_str() == Some(token) {
                return Ok(key.clone());
            }
        }
    }
    Err("Token not found".to_string())
}

pub async fn discover_profiles_service(
    base_path: &str,
    protocol: &str,
    host: &str,
) -> Result<Vec<Value>, String> {
    let profiles_dir = format!("{}/profiles", base_path);
    let mut profiles = Vec::new();

    for entry in fs::read_dir(profiles_dir)
        .map_err(|e| format!("Failed to read profiles directory: {}", e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        if entry
            .file_type()
            .map_err(|e| format!("Failed to get file type: {}", e))?
            .is_dir()
        {
            let name = entry
                .file_name()
                .into_string()
                .map_err(|e| format!("Failed to convert file name to string: {:?}", e))?;
            let passcode_path = format!("{}/profiles/{}/passcode.txt", base_path, name);
            let passcode_exists = Path::new(&passcode_path).exists();
            let pp_path = format!("{}/profiles/{}/pp.png", base_path, name);
            let pp_server_url = format!("{}://{}/profile/getPPBN/{}", protocol, host, name);
            let profile = json!({
                "name": name,
                "image": pp_server_url,
                "passcode": passcode_exists
            });
            profiles.push(profile);
        }
    }

    Ok(profiles)
}

pub async fn logout_service(token: &str, base_path: &str) -> Result<(), String> {
    let config_path = format!("{}/serverconfig.json", base_path);
    let config_content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read serverconfig.json: {}", e))?;
    let mut config: Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse serverconfig.json: {}", e))?;

    if let Some(tokens) = config.get_mut("Token").and_then(|t| t.as_object_mut()) {
        tokens.remove(token);
        fs::write(config_path, serde_json::to_string_pretty(&config).unwrap())
            .map_err(|e| format!("Failed to write serverconfig.json: {}", e))?;
    }

    Ok(())
}