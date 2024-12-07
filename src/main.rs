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
use std::time::Duration;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use reqwest::Client;

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
    argo_configure()?;
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
        "log": {
            "access": "/dev/null",
            "error": "/dev/null",
            "loglevel": "none"
        },
        "inbounds": [
            {
                "port": *ARGO_PORT.get().unwrap(),
                "protocol": "vless",
                "settings": {
                    "clients": [
                        {
                            "id": UUID.get().unwrap(),
                            "flow": "xtls-rprx-vision"
                        }
                    ],
                    "decryption": "none",
                    "fallbacks": [
                        { "dest": 3001 },
                        { "path": "/vless", "dest": 3002 },
                        { "path": "/vmess", "dest": 3003 },
                        { "path": "/trojan", "dest": 3004 }
                    ]
                },
                "streamSettings": {
                    "network": "tcp"
                }
            },
            {
                "port": 3001,
                "listen": "127.0.0.1",
                "protocol": "vless",
                "settings": {
                    "clients": [
                        {
                            "id": UUID.get().unwrap()
                        }
                    ],
                    "decryption": "none"
                },
                "streamSettings": {
                    "network": "ws",
                    "security": "none"
                }
            },
            {
                "port": 3002,
                "listen": "127.0.0.1",
                "protocol": "vless",
                "settings": {
                    "clients": [
                        {
                            "id": UUID.get().unwrap(),
                            "level": 0
                        }
                    ],
                    "decryption": "none"
                },
                "streamSettings": {
                    "network": "ws",
                    "security": "none",
                    "wsSettings": {
                        "path": "/vless"
                    }
                },
                "sniffing": {
                    "enabled": true,
                    "destOverride": ["http", "tls", "quic"],
                    "metadataOnly": false
                }
            },
            {
                "port": 3003,
                "listen": "127.0.0.1",
                "protocol": "vmess",
                "settings": {
                    "clients": [
                        {
                            "id": UUID.get().unwrap(),
                            "alterId": 0
                        }
                    ]
                },
                "streamSettings": {
                    "network": "ws",
                    "wsSettings": {
                        "path": "/vmess"
                    }
                },
                "sniffing": {
                    "enabled": true,
                    "destOverride": ["http", "tls", "quic"],
                    "metadataOnly": false
                }
            },
            {
                "port": 3004,
                "listen": "127.0.0.1",
                "protocol": "trojan",
                "settings": {
                    "clients": [
                        {
                            "password": UUID.get().unwrap()
                        }
                    ]
                },
                "streamSettings": {
                    "network": "ws",
                    "security": "none",
                    "wsSettings": {
                        "path": "/trojan"
                    }
                },
                "sniffing": {
                    "enabled": true,
                    "destOverride": ["http", "tls", "quic"],
                    "metadataOnly": false
                }
            }
        ],
        "dns": {
            "servers": ["https+local://8.8.8.8/dns-query"]
        },
        "outbounds": [
            {
                "protocol": "freedom"
            },
            {
                "tag": "WARP",
                "protocol": "wireguard",
                "settings": {
                    "secretKey": "YFYOAdbw1bKTHlNNi+aEjBM3BO7unuFC5rOkMRAz9XY=",
                    "address": [
                        "172.16.0.2/32",
                        "2606:4700:110:8a36:df92:102a:9602:fa18/128"
                    ],
                    "peers": [
                        {
                            "publicKey": "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=",
                            "allowedIPs": ["0.0.0.0/0", "::/0"],
                            "endpoint": "162.159.193.10:2408"
                        }
                    ],
                    "reserved": [78, 135, 76],
                    "mtu": 1280
                }
            }
        ],
        "routing": {
            "domainStrategy": "AsIs",
            "rules": [
                {
                    "type": "field",
                    "domain": ["domain:openai.com", "domain:ai.com"],
                    "outboundTag": "WARP"
                }
            ]
        }
    });

    let config_path = format!("{}/config.json", TEMP_DIR.get().unwrap());
    fs::write(config_path, serde_json::to_string_pretty(&config)?)?;
    Ok(())
}

fn argo_configure() -> shuttle_runtime::Result<()> {
    if let (Ok(auth), Ok(domain)) = (env::var("ARGO_AUTH"), env::var("ARGO_DOMAIN")) {
        if !auth.is_empty() && !domain.is_empty() {
            if auth.contains("TunnelSecret") {
                let tunnel_json_path = format!("{}/tunnel.json", TEMP_DIR.get().unwrap());
                fs::write(&tunnel_json_path, auth)?;

                let tunnel_id = {
                    let content = fs::read_to_string(&tunnel_json_path)?;
                    let v: serde_json::Value = serde_json::from_str(&content)?;
                    v["TunnelSecret"].as_str().unwrap_or("").to_string()
                };

                let tunnel_config = format!(
                    "tunnel: {}\n\
                     credentials-file: {}/tunnel.json\n\
                     protocol: http2\n\n\
                     ingress:\n\
                     - hostname: {}\n\
                     service: http://localhost:{}\n\
                     originRequest:\n\
                     noTLSVerify: true\n\
                     - service: http_status:404",
                    tunnel_id,
                    TEMP_DIR.get().unwrap(),
                    domain,
                    *ARGO_PORT.get().unwrap()
                );

                let tunnel_yml_path = format!("{}/tunnel.yml", TEMP_DIR.get().unwrap());
                fs::write(tunnel_yml_path, tunnel_config)?;
            }
        }
    }
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

    let client = Client::new();
    for (url, filename) in files {
        let path = format!("{}/{}", TEMP_DIR.get().unwrap(), filename);
        if !Path::new(&path).exists() {
            let response = client.get(url)
                .timeout(Duration::from_secs(30))
                .send()
                .await?;
            let bytes = response.bytes().await?;
            fs::write(&path, bytes)?;
            
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

async fn get_argo_domain() -> shuttle_runtime::Result<String> {
    if let Ok(domain) = env::var("ARGO_DOMAIN") {
        if !domain.is_empty() {
            return Ok(domain);
        }
    }

    let boot_log_path = format!("{}/boot.log", TEMP_DIR.get().unwrap());
    let content = fs::read_to_string(boot_log_path)?;
    
    let domain = content
        .lines()
        .find(|line| line.contains("https://") && line.contains("trycloudflare.com"))
        .and_then(|line| line.split("https://").nth(1))
        .and_then(|domain| domain.split('/').next())
        .ok_or("Failed to extract domain from boot.log")?;

    Ok(domain.to_string())
}

async fn get_isp_info() -> shuttle_runtime::Result<String> {
    let client = Client::new();
    let response = client.get("https://speed.cloudflare.com/meta")
        .timeout(Duration::from_secs(10))
        .send()
        .await?
        .text()
        .await?;

    let v: serde_json::Value = serde_json::from_str(&response)?;
    let isp = format!("{}-{}", 
        v["organization"].as_str().unwrap_or(""),
        v["asOrganization"].as_str().unwrap_or("")
    );
    
    Ok(isp.replace(' ', "_"))
}

async fn generate_links() -> shuttle_runtime::Result<()> {
    let domain = get_argo_domain().await?;
    let isp = get_isp_info().await?;
    
    let links = vec![
        format!("vless://{uuid}@{domain}:443?encryption=none&security=tls&sni={domain}&fp=random&type=ws&host={domain}&path=%2F#{isp}_VLESS"),
        format!("vless://{uuid}@{domain}:443?encryption=none&security=tls&sni={domain}&fp=random&type=ws&host={domain}&path=%2Fvless#{isp}_VLESS"),
        format!("vmess://{}", BASE64.encode(format!(r#"{{"v": "2","ps": "{isp}_Vmess","add": "{domain}","port": "443","id": "{uuid}","aid": "0","scy": "none","net": "ws","type": "none","host": "{domain}","path": "/vmess","tls": "tls","sni": "{domain}","fp": "random"}}"#))),
        format!("trojan://{uuid}@{domain}:443?security=tls&sni={domain}&fp=random&type=ws&host={domain}&path=%2Ftrojan#{isp}_Trojan"),
    ];

    let sub_content = BASE64.encode(links.join("\n"));
    fs::write(format!("{}/sub.txt", TEMP_DIR.get().unwrap()), sub_content)?;
    
    Ok(())
}

async fn handle_sub() -> Result<String, StatusCode> {
    let sub_path = format!("{}/sub.txt", TEMP_DIR.get().unwrap());
    fs::read_to_string(sub_path)
        .map_err(|_| StatusCode::NOT_FOUND)
}
