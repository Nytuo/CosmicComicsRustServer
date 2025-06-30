#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        AppGlobalVariables,
        services::profile_service::{
            CreateUserPayload, create_user_service, delete_account_service,
            discover_profiles_service, generate_token, login_check_service, login_service,
            logout_service, modify_profile_service, resolve_token,
        },
    };
    use serde_json::Value;

    #[tokio::test]
    async fn test_create_user_service_creates_all_files() {
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::tempdir;
        let dir = tempdir().unwrap();
        let base_path = dir.path();
        let user = "new_user";

        let default_pp_dir = base_path.join("public/Images/account_default");
        fs::create_dir_all(&default_pp_dir).unwrap();
        let default_pp_path = default_pp_dir.join("default_pp.png");
        let mut pp_file = File::create(&default_pp_path).unwrap();
        writeln!(pp_file, "fake image data").unwrap();

        let payload = CreateUserPayload::new(user.to_string(), "secure_pass".to_string(), None);

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
        use serde_json::{Value, json};
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let base_path = dir.path();
        let user_dir = base_path.join("profiles/testuser");
        fs::create_dir_all(&user_dir).unwrap();

        let config_path = base_path.join("serverconfig.json");
        let mut config_file = File::create(&config_path).unwrap();
        write!(config_file, "{}", json!({ "Token": {} }).to_string()).unwrap();

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
        use std::fs::{create_dir_all, read_to_string};
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let base = dir.path();
        let profile_dir = base.join("profiles/testuser");
        create_dir_all(&profile_dir).unwrap();

        std::fs::write(profile_dir.join("passcode.txt"), "oldpass").unwrap();

        modify_profile_service(
            "testuser",
            Some("newpass"),
            None,
            None,
            base.to_str().unwrap(),
        )
        .await
        .unwrap();

        let updated = read_to_string(profile_dir.join("passcode.txt")).unwrap();
        assert_eq!(updated, "newpass");
    }

    #[tokio::test]
    async fn test_modify_profile_update_picture() {
        use std::fs::{create_dir_all, read, write};
        use tempfile::tempdir;

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
        use std::fs::{create_dir_all, metadata};
        use tempfile::tempdir;

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
        use serde_json::json;
        use std::fs::{File, create_dir_all};
        use std::io::Write;
        use tempfile::tempdir;

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
        use std::fs::{File, create_dir_all};
        use std::io::Write;
        use tempfile::tempdir;

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
        use serde_json::json;
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

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
        use serde_json::json;
        use std::fs::File;
        use std::io::Write;
        use tempfile::tempdir;

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
        use std::fs::{File, create_dir_all};
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
            profile["image"]
                .as_str()
                .unwrap()
                .contains("http://localhost/profile/getPPBN/user42")
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

    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_delete_account_service_success() {
        let base_path = "test_data";
        let token = "testuser";
        let user_dir = format!("{}/profiles/{}", base_path, token);

        std::fs::create_dir_all(&user_dir).unwrap();

        let mut global = AppGlobalVariables::default();
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        global.opened_db.insert(token.to_string(), pool);

        let result = delete_account_service(token, base_path, &mut global).await;

        assert!(result.is_ok());
        assert!(!std::path::Path::new(&user_dir).exists());
        assert!(!global.opened_db.contains_key(token));
    }
}
