use crate::routes_manager::create_router;
use serde_json::{Value, json};
use sqlx::sqlite::SqlitePool;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};
use std::fs::File;
use std::io::Write;
use rust_embed::RustEmbed;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{self};

mod controllers;
mod endpoints;
mod repositories;
mod routes_manager;
mod services;
mod utils;

#[derive(RustEmbed)]
#[folder = "public/Images"]
struct Asset;

pub struct AppConfig {
    pub base_path: String,
    pub version: String,
}

pub struct ApiTokens {
    pub marvel_public_key: String,
    pub marvel_private_key: String,
    pub google_books_api_key: String,
    pub open_library_api_key: String,
}

pub struct AppGlobalVariables {
    pub progress_status: HashMap<String, HashMap<String, HashMap<String, String>>>,
    pub opened_db: HashMap<String, SqlitePool>,
}

impl AppGlobalVariables {
    pub fn new() -> Self {
        AppGlobalVariables {
            progress_status: HashMap::new(),
            opened_db: HashMap::new(),
        }
    }
    pub fn default() -> Self {
        AppGlobalVariables {
            progress_status: HashMap::new(),
            opened_db: HashMap::new(),
        }
    }
    pub fn get_progress_status(
        &self,
        key: &str,
    ) -> Option<&HashMap<String, HashMap<String, String>>> {
        self.progress_status.get(key)
    }
    pub fn set_progress_status(
        &mut self,
        user_token: String,
        key: String,
        status: String,
        progress: String,
        current_task: String,
    ) {
        self.progress_status.entry(user_token).or_default().insert(
            key,
            HashMap::from([
                ("status".to_string(), status),
                ("percentage".to_string(), progress),
                ("current_file".to_string(), current_task),
            ]),
        );
    }
}

fn get_data_path() -> PathBuf {
    let is_portable = PathBuf::from("portable.txt").exists();
    if is_portable {
        PathBuf::from("./..").join("CosmicData")
    } else {
        match env::consts::OS {
            "windows" => {
                let appdata = env::var("APPDATA").unwrap_or_else(|_| "".to_string());
                PathBuf::from(appdata)
                    .join("CosmicComics")
                    .join("CosmicData")
            }
            "macos" => {
                let home = env::var("HOME").unwrap_or_else(|_| "".to_string());
                PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("CosmicComics")
                    .join("CosmicData")
            }
            "linux" => {
                let home = env::var("HOME").unwrap_or_else(|_| "".to_string());
                PathBuf::from(home)
                    .join(".config")
                    .join("CosmicComics")
                    .join("CosmicData")
            }
            _ => PathBuf::from("."),
        }
    }
}

fn setup_cosmic_comics_temp(base_path: &str) {
    let cosmic_comics_temp = PathBuf::from(base_path.clone());

    fs::create_dir_all(&cosmic_comics_temp).unwrap_or_else(|err| {
        eprintln!("Failed to create directory: {:?}", err);
    });

    let env_path = cosmic_comics_temp.join(".env");
    if !env_path.exists() {
        let env_sample_path = PathBuf::from(".env.sample");
        if env_sample_path.exists() {
            fs::copy(&env_sample_path, &env_path).unwrap_or_else(|err| {
                eprintln!("Failed to copy .env.sample: {:?}", err);
                0
            });
        } else {
            eprintln!(".env.sample file not found!");
        }
    }
}

fn setup_server_config(cosmic_comics_temp: &str, dev_mode: bool) {
    let server_config_path = PathBuf::from(cosmic_comics_temp.clone()).join("serverconfig.json");

    if !server_config_path.exists() {
        let default_config = json!({
            "Token": {},
            "port": 4696
        });

        if let Err(err) = fs::write(
            &server_config_path,
            serde_json::to_string_pretty(&default_config).unwrap(),
        ) {
            eprintln!("Failed to create serverconfig.json: {:?}", err);
        }
    } else if !dev_mode {
        if let Ok(config_content) = fs::read_to_string(&server_config_path) {
            if let Ok(mut config_json) = serde_json::from_str::<Value>(&config_content) {
                if let Some(token_field) = config_json.get_mut("Token") {
                    *token_field = json!({});
                }

                if let Err(err) = fs::write(
                    &server_config_path,
                    serde_json::to_string_pretty(&config_json).unwrap(),
                ) {
                    eprintln!(
                        "Failed to reset Token field in serverconfig.json: {:?}",
                        err
                    );
                }
            } else {
                eprintln!("Failed to parse serverconfig.json");
            }
        } else {
            eprintln!("Failed to read serverconfig.json");
        }
    }
}

async fn handle_sigint() {
    println!("Removing ZIPs to DL");

    let todl_path = "./public/TODL";
    let uploads_path = format!(
        "{}/uploads",
        std::env::var("CosmicComicsTemp").unwrap_or_else(|_| ".".to_string())
    );

    if Path::new(todl_path).exists() {
        if let Err(err) = fs::remove_dir_all(todl_path) {
            eprintln!("Failed to remove directory {}: {:?}", todl_path, err);
        }
    }

    if Path::new(&uploads_path).exists() {
        if let Err(err) = fs::remove_dir_all(&uploads_path) {
            eprintln!("Failed to remove directory {}: {:?}", uploads_path, err);
        }
    }
}

async fn sigint() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("Received Ctrl+C, shutting down...");
            handle_sigint().await;
            },
        _ = terminate => {
            println!("Received SIGTERM, shutting down...");
            handle_sigint().await;
            },
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let dev_mode = env::var("DEV_MODE").unwrap_or_else(|_| "false".to_string());
    let base_path: String = if dev_mode == "true" {
        env::current_dir().unwrap().to_str().unwrap().to_string()
    } else {
        get_data_path().to_str().unwrap().to_string()
    };

    setup_cosmic_comics_temp(&base_path);
    setup_server_config(&base_path, dev_mode == "true");
    fs::create_dir_all(
        PathBuf::from(base_path.clone())
            .join("public")
            .join("FirstImagesOfAll"),
    )
    .unwrap_or_else(|err| {
        eprintln!("Failed to create directory: {:?}", err);
    });
    let mut permissions = fs::metadata(
        PathBuf::from(base_path.clone())
            .join("public")
            .join("FirstImagesOfAll"),
    )
    .unwrap()
    .permissions();
    permissions.set_mode(0o777);
    fs::set_permissions(
        PathBuf::from(base_path.clone())
            .join("public")
            .join("FirstImagesOfAll"),
        permissions,
    )
    .unwrap_or_else(|err| {
        eprintln!("Failed to set permissions: {:?}", err);
    });
    fs::create_dir_all(PathBuf::from(base_path.clone()).join("public").join("TODL"))
        .unwrap_or_else(|err| {
            eprintln!("Failed to create directory: {:?}", err);
        });
    fs::create_dir_all(
        PathBuf::from(base_path.clone())
            .join("public")
            .join("uploads"),
    )
    .unwrap_or_else(|err| {
        eprintln!("Failed to create directory: {:?}", err);
    });
    fs::create_dir_all(PathBuf::from(base_path.clone()).join("profiles")).unwrap_or_else(|err| {
        eprintln!("Failed to create directory: {:?}", err);
    });

    fs::create_dir_all(PathBuf::from(base_path.clone()) .join("public").join("Images")).unwrap_or_else(|err| {
        eprintln!("Failed to create directory: {:?}", err);
    });

    for file in Asset::iter() {
        println!("Extracting: {}", file);

        if let Some(content) = Asset::get(&file) {
            let out_path = Path::new(&base_path)
                .join("public")
                .join("Images")
                .join(&*file);
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create output directories");
            }

            let mut out_file = File::create(&out_path).expect("Failed to create output file");
            out_file
                .write_all(&content.data)
                .expect("Failed to write file");

            println!("Written to: {:?}", out_path);
        }
    }

    let scheduler = JobScheduler::new().await.unwrap();
    let base_path_clone = base_path.clone();
    let token_reset = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
        let base_path = base_path_clone.clone();
        Box::pin(async move {
            let server_config_path = PathBuf::from(base_path.clone()).join("serverconfig.json");
            if let Ok(config_content) = fs::read_to_string(&server_config_path) {
                if let Ok(mut config_json) = serde_json::from_str::<Value>(&config_content) {
                    if let Some(token_field) = config_json.get_mut("Token") {
                        *token_field = json!({});
                    }

                    if let Err(err) = fs::write(
                        &server_config_path,
                        serde_json::to_string_pretty(&config_json).unwrap(),
                    ) {
                        eprintln!(
                            "Failed to reset Token field in serverconfig.json: {:?}",
                            err
                        );
                    }
                } else {
                    eprintln!("Failed to parse serverconfig.json");
                }
            } else {
                eprintln!("Failed to read serverconfig.json");
            }
        })
    })
    .unwrap();

    let base_path_clone1 = base_path.clone();
    let zip_remover = Job::new_async("0 0 */2 * * *", move |_uuid, _l| {
        let base_path = base_path_clone1.clone();
        Box::pin(async move {
            let public_path = PathBuf::from(base_path.clone()).join("public").join("TODL");
            if public_path.exists() {
                fs::remove_dir_all(&public_path).unwrap_or_else(|err| {
                    eprintln!("Failed to remove directory: {:?}", err);
                });
            }
        })
    })
    .unwrap();

    scheduler.add(token_reset).await.unwrap();
    scheduler.add(zip_remover).await.unwrap();
    scheduler.start().await.unwrap();

    let marvel_public_key = std::env::var("MARVEL_PUBLIC_KEY").unwrap_or_else(|_| "".to_string());
    let marvel_private_key = std::env::var("MARVEL_PRIVATE_KEY").unwrap_or_else(|_| "".to_string());
    let google_books_api_key =
        std::env::var("GOOGLE_BOOKS_API_KEY").unwrap_or_else(|_| "".to_string());
    let open_library_api_key =
        std::env::var("OPEN_LIBRARY_API_KEY").unwrap_or_else(|_| "".to_string());
    let version = env!("CARGO_PKG_VERSION").to_string();
    let app_state = Arc::new(tokio::sync::Mutex::new(AppConfig {
        base_path: base_path.clone(),
        version: version.clone(),
    }));

    let api_tokens = Arc::new(tokio::sync::Mutex::new(ApiTokens {
        marvel_public_key,
        marvel_private_key,
        google_books_api_key,
        open_library_api_key,
    }));

    let app_global_variables = Arc::new(tokio::sync::Mutex::new(AppGlobalVariables::new()));

    let app = create_router(app_state, api_tokens, app_global_variables).layer(
        ServiceBuilder::new().layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        ),
    );

    let port = if dev_mode == "true" {
        3000
    } else {
        let server_config_path = PathBuf::from(base_path.clone()).join("serverconfig.json");
        let config_content = fs::read_to_string(&server_config_path).unwrap();
        let config: Value = serde_json::from_str(&config_content).unwrap();
        config["port"].as_u64().unwrap_or(4696) as u16
    };

    let bind_url = format!("0.0.0.0:{}", port);
    println!("Server running at {}", bind_url);

    let listener = tokio::net::TcpListener::bind(bind_url)
        .await
        .expect("Failed to bind TCP listener");
    axum::serve(listener, app)
        .with_graceful_shutdown(sigint())
        .await
        .unwrap();
}
