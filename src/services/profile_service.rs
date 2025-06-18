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

//#######################################################
#[tokio::test]
async fn test_create_user_service_creates_all_files() {
    use tempfile::tempdir;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    let dir = tempdir().unwrap();
    let base_path = dir.path();
    let user = "new_user";

    let default_pp_dir = base_path.join("public/Images/account_default");
    fs::create_dir_all(&default_pp_dir).unwrap();
    let default_pp_path = default_pp_dir.join("default_pp.png");
    let mut pp_file = File::create(&default_pp_path).unwrap();
    writeln!(pp_file, "fake image data").unwrap();

    let payload = CreateUserPayload::new(
        user.to_string(),
        "secure_pass".to_string(),
        None,
    );

    let result = create_user_service(&payload, base_path.to_str().unwrap()).await;
    assert!(result.is_ok(), "User creation should succeed");

    let user_dir = base_path.join("profiles").join(user);

    assert!(user_dir.exists());
    assert!(user_dir.join("passcode.txt").exists());
    assert!(user_dir.join("config.json").exists());
    assert!(user_dir.join("pp.png").exists());

    let pass = fs::read_to_string(user_dir.join("passcode.txt")).unwrap();
    assert_eq!(pass, "secure_pass");

    let config = fs::read_to_string(user_dir.join("config.json")).unwrap();
    let json: serde_json::Value = serde_json::from_str(&config).unwrap();
    assert_eq!(json["language"], "us");
}

#[test]
fn test_generate_token_creates_token_and_updates_filesystem() {
    use tempfile::tempdir;
    use std::fs::{self, File};
    use std::io::Write;
    use serde_json::{json, Value};

    let dir = tempdir().unwrap();
    let base_path = dir.path();
    let user_dir = base_path.join("profiles/testuser");
    fs::create_dir_all(&user_dir).unwrap();

    let config_path = base_path.join("serverconfig.json");
    let mut config_file = File::create(&config_path).unwrap();
    write!(
        config_file,
        "{}",
        json!({ "Token": {} }).to_string()
    )
        .unwrap();

    let token = generate_token(
        "testuser",
        base_path.to_str().unwrap(),
        user_dir.to_str().unwrap(),
    );

    assert_eq!(token.len(), 32);
    assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));

    let updated_config: Value =
        serde_json::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    assert_eq!(updated_config["Token"]["testuser"], token);

    assert!(user_dir.join("current_book").exists());
}

#[tokio::test]
async fn test_modify_profile_update_passcode() {
    use tempfile::tempdir;
    use std::fs::{create_dir_all, read_to_string};

    let dir = tempdir().unwrap();
    let base = dir.path();
    let profile_dir = base.join("profiles/testuser");
    create_dir_all(&profile_dir).unwrap();

    std::fs::write(profile_dir.join("passcode.txt"), "oldpass").unwrap();

    modify_profile_service("testuser", Some("newpass"), None, None, base.to_str().unwrap())
        .await
        .unwrap();

    let updated = read_to_string(profile_dir.join("passcode.txt")).unwrap();
    assert_eq!(updated, "newpass");
}

#[tokio::test]
async fn test_modify_profile_update_picture() {
    use tempfile::tempdir;
    use std::fs::{create_dir_all, write, read};

    let dir = tempdir().unwrap();
    let base = dir.path();
    let user = "testuser";
    let public_path = base.join("public/path/to/img.png");
    let profile_dir = base.join("profiles").join(user);
    create_dir_all(public_path.parent().unwrap()).unwrap();
    create_dir_all(&profile_dir).unwrap();

    write(&public_path, b"fake image content").unwrap();

    let url_like = format!("http://localhost/path/to/img.png");
    modify_profile_service(user, None, Some(&url_like), None, base.to_str().unwrap())
        .await
        .unwrap();

    let copied = read(profile_dir.join("pp.png")).unwrap();
    assert_eq!(copied, b"fake image content");
}

#[tokio::test]
async fn test_modify_profile_rename_user() {
    use tempfile::tempdir;
    use std::fs::{create_dir_all, metadata};

    let dir = tempdir().unwrap();
    let base = dir.path();
    let old_user = "oldname";
    let new_user = "newname";

    let old_path = base.join("profiles").join(old_user);
    create_dir_all(&old_path).unwrap();

    modify_profile_service(old_user, None, None, Some(new_user), base.to_str().unwrap())
        .await
        .unwrap();

    assert!(metadata(base.join("profiles").join(new_user)).is_ok());
    assert!(metadata(base.join("profiles").join(old_user)).is_err());
}


#[tokio::test]
async fn test_login_service_success() {
    use tempfile::tempdir;
    use std::fs::{create_dir_all, File};
    use std::io::Write;
    use serde_json::json;

    let dir = tempdir().unwrap();
    let base_path = dir.path();
    let user_dir = base_path.join("profiles/jane");
    create_dir_all(&user_dir).unwrap();

    let config_path = base_path.join("serverconfig.json");
    let mut config_file = File::create(&config_path).unwrap();
    write!(config_file, "{}", json!({ "Token": {} }).to_string()).unwrap();

    let mut file = File::create(user_dir.join("passcode.txt")).unwrap();
    writeln!(file, "mypassword").unwrap();

    let token = login_service("jane", "mypassword", base_path.to_str().unwrap())
        .await
        .expect("Login should succeed");

    assert!(!token.is_empty());
}

#[tokio::test]
async fn test_login_service_user_not_found() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let base_path = dir.path();

    let result = login_service("ghost", "secret", base_path.to_str().unwrap()).await;
    assert_eq!(result.unwrap_err(), "User does not exist");
}

#[tokio::test]
async fn test_login_service_wrong_passcode() {
    use tempfile::tempdir;
    use std::fs::{create_dir_all, File};
    use std::io::Write;

    let dir = tempdir().unwrap();
    let base_path = dir.path();
    let user_dir = base_path.join("profiles/user1");
    create_dir_all(&user_dir).unwrap();

    let mut file = File::create(user_dir.join("passcode.txt")).unwrap();
    writeln!(file, "correctpass").unwrap();

    let result = login_service("user1", "wrongpass", base_path.to_str().unwrap()).await;
    assert_eq!(result.unwrap_err(), "Invalid passcode");
}



#[tokio::test]
async fn test_login_check_service_valid_token() {
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use serde_json::json;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("serverconfig.json");

    let config_json = json!({
        "Token": {
            "john_doe": "securetoken123"
        }
    });
    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    let result = login_check_service("securetoken123", dir.path().to_str().unwrap()).await;
    assert_eq!(result.unwrap(), "john_doe");
}

#[tokio::test]
async fn test_login_check_service_invalid_token() {
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use serde_json::json;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("serverconfig.json");

    let config_json = json!({
        "Token": {
            "john_doe": "securetoken123"
        }
    });
    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_json.to_string().as_bytes()).unwrap();

    let result = login_check_service("wrongtoken", dir.path().to_str().unwrap()).await;
    assert_eq!(result.unwrap_err(), "Token not found");
}


#[tokio::test]
async fn test_logout_service_removes_token() {
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("serverconfig.json");

    let initial_config = json!({
        "Token": {
            "user1": "abc123"
        }
    });
    let mut file = File::create(&config_path).unwrap();
    file.write_all(initial_config.to_string().as_bytes())
        .unwrap();

    logout_service("user1", dir.path().to_str().unwrap())
        .await
        .unwrap();

    let updated_content = std::fs::read_to_string(&config_path).unwrap();
    let updated_json: Value = serde_json::from_str(&updated_content).unwrap();
    assert!(
        !updated_json["Token"]
            .as_object()
            .unwrap()
            .contains_key("user1")
    );
}

#[tokio::test]
async fn test_discover_profiles_service() {
    use std::fs::{create_dir_all, File};
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let base_path = dir.path();

    let user_dir = base_path.join("profiles/user42");
    create_dir_all(&user_dir).unwrap();

    File::create(user_dir.join("passcode.txt")).unwrap();
    File::create(user_dir.join("pp.png")).unwrap();

    let result = discover_profiles_service(base_path.to_str().unwrap(), "http", "localhost")
        .await
        .expect("Service failed");

    assert_eq!(result.len(), 1);
    let profile = &result[0];
    assert_eq!(profile["name"], "user42");
    assert_eq!(profile["passcode"], true);
    assert!(
        profile["image"].as_str().unwrap().contains("http://localhost/profile/getPPBN/user42")
    );
}

#[test]
fn test_create_user_payload() {
    let payload = CreateUserPayload::new(
        "alice".to_string(),
        "password123".to_string(),
        Some("profile_pic.png".to_string()),
    );
    assert_eq!(payload.name, "alice");
    assert_eq!(payload.password, "password123");
    assert_eq!(payload.pp, Some("profile_pic.png".to_string()));
}

#[test]
fn test_resolve_token_success() {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("serverconfig.json");

    let config_json = r#"
    {
        "Token": {
            "user123": "valid_token"
        }
    }"#;

    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_json.as_bytes()).unwrap();

    let result = resolve_token("valid_token", dir.path().to_str().unwrap());
    assert_eq!(result, Some("user123".to_string()));
}
#[test]
fn test_resolve_token_invalid_token() {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("serverconfig.json");

    let config_json = r#"
    {
        "Token": {
            "user123": "valid_token"
        }
    }"#;

    let mut file = File::create(&config_path).unwrap();
    file.write_all(config_json.as_bytes()).unwrap();

    let result = resolve_token("wrong_token", dir.path().to_str().unwrap());
    assert_eq!(result, None);
}
#[test]
fn test_resolve_token_file_missing() {
    use tempfile::tempdir;

    let dir = tempdir().unwrap(); // no file created
    let result = resolve_token("any_token", dir.path().to_str().unwrap());
    assert_eq!(result, None);
}
#[test]
fn test_resolve_token_malformed_json() {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("serverconfig.json");

    let mut file = File::create(&config_path).unwrap();
    file.write_all(b"{ invalid json").unwrap();

    let result = resolve_token("any_token", dir.path().to_str().unwrap());
    assert_eq!(result, None);
}
