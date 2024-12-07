use axum::{
    routing::{get},
    Router,
};
use hyper::StatusCode;
use once_cell::sync::OnceCell;
use serde_json::json;
use std::{process::Command, env, path::Path, fs};
use tokio::process::Command as TokioCommand;
use uuid::Uuid;

static TEMP_DIR: OnceCell<String> = OnceCell::new();
static UUID: OnceCell<String> = OnceCell::new();
static ARGO_PORT: OnceCell<u16> = OnceCell::new();

#[shuttle_runtime::main]
async fn main() -> shuttle_runtime::Result<Router> {
    // Initialize global variables
    TEMP_DIR.set("./temp".to_string()).unwrap();
    UUID.set(env::var("UUID").unwrap_or_else(|_| Uuid::new_v4().to_string())).unwrap();
    ARGO_PORT.set(env::var("ARGO_PORT").unwrap_or_else(|_| "8001".to_string()).parse().unwrap()).unwrap();

    // Create temp directory if it doesn't exist
    let temp_path = Path::new(TEMP_DIR.get().unwrap());
    if !temp_path.exists() {
        fs::create_dir_all(temp_path)?;
    }

    // Initialize the service
    initialize_service().await?;

    // Create router
    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/sub", get(handle_sub));

    Ok(app)
}

async fn initialize_service() -> shuttle_runtime::Result<()> {
    cleanup_old_files()?;
    generate_config()?;
    download_required_files().await?;
    run_services().await?;
    generate_links().await?;
    Ok(())
}

fn cleanup_old_files() -> shuttle_runtime::Result<()> {
    let files_to_remove = vec![
        "boot.log", "sub.txt", "config.json", "tunnel.json", "tunnel.yml"
    ];
    
    for file in files_to_remove {
        let file_path = format!("{}/{}", TEMP_DIR.get().unwrap(), file);
        if Path::new(&file_path).exists() {
            fs::remove_file(file_path)?;
        }
    }
    Ok(())
}

fn generate_config() -> shuttle_runtime::Result<()> {
    let config = json!({
        "log": { "access": "/dev/null", "error": "/dev/null", "loglevel": "none" },
        "inbounds": [
            {
                "port": *ARGO_PORT.get().unwrap(),
                "protocol": "vless",
                "settings": {
                    "clients": [{ "id": UUID.get().unwrap(), "flow": "xtls-rprx-vision" }],
                    "decryption": "none",
                    "fallbacks": [
                        { "dest": 3001 },
                        { "path": "/vless", "dest": 3002 },
                        { "path": "/vmess", "dest": 3003 },
                        { "path": "/trojan", "dest": 3004 }
                    ]
                },
                "streamSettings": { "network": "tcp" }
            },
            // ... rest of the config
        ]
    });

    let config_path = format!("{}/config.json", TEMP_DIR.get().unwrap());
    fs::write(config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

async fn download_required_files() -> shuttle_runtime::Result<()> {
    let arch = std::env::consts::ARCH;
    let files = match arch {
        "aarch64" | "arm" => vec![
            ("https://github.com/eooce/test/releases/download/arm64/bot13", "bot"),
            ("https://github.com/eooce/test/releases/download/ARM/web", "web"),
            ("https://github.com/eooce/test/releases/download/ARM/swith", "npm"),
        ],
        "x86_64" | "x86" => vec![
            ("https://github.com/eooce/test/releases/download/amd64/bot13", "bot"),
            ("https://github.com/eooce/test/releases/download/123/web", "web"),
            ("https://github.com/eooce/test/releases/download/bulid/swith", "npm"),
        ],
        _ => return Err("Unsupported architecture".into()),
    };

    for (url, filename) in files {
        let path = format!("{}/{}", TEMP_DIR.get().unwrap(), filename);
        if !Path::new(&path).exists() {
            let response = reqwest::get(url).await?;
            let bytes = response.bytes().await?;
            fs::write(&path, bytes)?;
            
            // Make file executable
            Command::new("chmod")
                .arg("777")
                .arg(&path)
                .output()?;
        }
    }
    Ok(())
}

async fn run_services() -> shuttle_runtime::Result<()> {
    // Run nezha service if configured
    if let (Ok(server), Ok(port), Ok(key)) = (
        env::var("NEZHA_SERVER"),
        env::var("NEZHA_PORT"),
        env::var("NEZHA_KEY")
    ) {
        let npm_path = format!("{}/npm", TEMP_DIR.get().unwrap());
        if Path::new(&npm_path).exists() {
            let tls = matches!(port.as_str(), "443"|"8443"|"2096"|"2087"|"2083"|"2053");
            TokioCommand::new(&npm_path)
                .args(&["-s", &format!("{}:{}", server, port), "-p", &key])
                .arg(if tls { "--tls" } else { "" })
                .spawn()?;
        }
    }

    // Run web service
    let web_path = format!("{}/web", TEMP_DIR.get().unwrap());
    if Path::new(&web_path).exists() {
        TokioCommand::new(&web_path)
            .args(&["-c", &format!("{}/config.json", TEMP_DIR.get().unwrap())])
            .spawn()?;
    }

    // Run bot service
    let bot_path = format!("{}/bot", TEMP_DIR.get().unwrap());
    if Path::new(&bot_path).exists() {
        let args = match env::var("ARGO_AUTH") {
            Ok(auth) if auth.len() >= 120 && auth.len() <= 250 => {
                vec!["tunnel", "--edge-ip-version", "auto", "--no-autoupdate", 
                     "--protocol", "http2", "run", "--token", &auth]
            },
            Ok(auth) if auth.contains("TunnelSecret") => {
                vec!["tunnel", "--edge-ip-version", "auto", "--config",
                     &format!("{}/tunnel.yml", TEMP_DIR.get().unwrap()), "run"]
            },
            _ => {
                vec!["tunnel", "--edge-ip-version", "auto", "--no-autoupdate",
                     "--protocol", "http2", "--logfile",
                     &format!("{}/boot.log", TEMP_DIR.get().unwrap()),
                     "--loglevel", "info",
                     "--url", &format!("http://localhost:{}", ARGO_PORT.get().unwrap())]
            }
        };
        TokioCommand::new(&bot_path).args(&args).spawn()?;
    }

    Ok(())
}

async fn generate_links() -> shuttle_runtime::Result<()> {
    // Implementation of link generation logic
    // This would include reading the boot.log file, generating the subscription links
    // and saving them to sub.txt
    Ok(())
}

async fn handle_sub() -> Result<String, StatusCode> {
    let sub_path = format!("{}/sub.txt", TEMP_DIR.get().unwrap());
    fs::read_to_string(sub_path).map_err(|_| StatusCode::NOT_FOUND)
}
