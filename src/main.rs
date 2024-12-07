use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use once_cell::sync::Lazy;
use std::process::Command;
use std::fs;
use std::path::Path;
use std::io::Write;
use base64;
use serde_json::json;

static PORT: Lazy<String> = Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));
static UUID: Lazy<String> = Lazy::new(|| std::env::var("UUID").unwrap_or_else(|_| "2447700e-0d8e-44c2-b9b2-6a5a73777981".to_string()));
static NEZHA_SERVER: Lazy<String> = Lazy::new(|| std::env::var("NEZHA_SERVER").unwrap_or_else(|_| "nz.abcd.cn".to_string()));
static NEZHA_PORT: Lazy<String> = Lazy::new(|| std::env::var("NEZHA_PORT").unwrap_or_else(|_| "5555".to_string()));
static NEZHA_KEY: Lazy<String> = Lazy::new(|| std::env::var("NEZHA_KEY").unwrap_or_else(|_| "".to_string()));
static ARGO_DOMAIN: Lazy<String> = Lazy::new(|| std::env::var("ARGO_DOMAIN").unwrap_or_else(|_| "".to_string()));
static ARGO_AUTH: Lazy<String> = Lazy::new(|| std::env::var("ARGO_AUTH").unwrap_or_else(|_| "".to_string()));
static CFIP: Lazy<String> = Lazy::new(|| std::env::var("CFIP").unwrap_or_else(|_| "ma.ma".to_string()));
static NAME: Lazy<String> = Lazy::new(|| std::env::var("NAME").unwrap_or_else(|_| "Vls".to_string()));
static FILE_PATH: Lazy<String> = Lazy::new(|| std::env::var("FILE_PATH").unwrap_or_else(|_| "./temp".to_string()));
static ARGO_PORT: Lazy<String> = Lazy::new(|| std::env::var("ARGO_PORT").unwrap_or_else(|_| "8001".to_string()));

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    // Create FILE_PATH directory if it doesn't exist
    let file_path = FILE_PATH.as_str();
    fs::create_dir_all(file_path).expect("Failed to create directory");

    // Cleanup old files
    cleanup_oldfiles(file_path);

    // Configure Argo
    argo_configure(file_path);

    // Generate config
    generate_config(file_path);

    // Download and prepare files
    prepare_files(file_path);

    // Run services
    run_services(file_path);

    // Generate links
    generate_links(file_path);

    let router = Router::new()
        .route("/", get(|| async { "Hello world" }))
        .route("/sub", get(handle_sub));

    Ok(router.into())
}

async fn handle_sub() -> impl IntoResponse {
    match fs::read_to_string(format!("{}/sub.txt", *FILE_PATH)) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

fn cleanup_oldfiles(file_path: &str) {
    let files = ["boot.log", "sub.txt", "config.json", "tunnel.json", "tunnel.yml"];
    for file in files {
        let _ = fs::remove_file(format!("{}/{}", file_path, file));
    }
}

fn argo_configure(file_path: &str) {
    if ARGO_AUTH.is_empty() || ARGO_DOMAIN.is_empty() {
        println!("ARGO_DOMAIN or ARGO_AUTH variable is empty, use quick tunnels");
        return;
    }

    if ARGO_AUTH.contains("TunnelSecret") {
        fs::write(format!("{}/tunnel.json", file_path), ARGO_AUTH.as_bytes()).expect("Failed to write tunnel.json");
        
        let tunnel_yml = format!(
            r#"tunnel: {}
credentials-file: {}/tunnel.json
protocol: http2

ingress:
  - hostname: {}
    service: http://localhost:{}
    originRequest:
      noTLSVerify: true
  - service: http_status:404
"#,
            ARGO_AUTH.split('"').nth(11).unwrap_or(""),
            file_path,
            *ARGO_DOMAIN,
            *ARGO_PORT
        );
        
        fs::write(format!("{}/tunnel.yml", file_path), tunnel_yml).expect("Failed to write tunnel.yml");
    } else {
        println!("ARGO_AUTH mismatch TunnelSecret, use token connect to tunnel");
    }
}

fn generate_config(file_path: &str) {
    let config = json!({
        "log": { "access": "/dev/null", "error": "/dev/null", "loglevel": "none" },
        "inbounds": [
            {
                "port": ARGO_PORT.parse::<i32>().unwrap_or(8001),
                "protocol": "vless",
                "settings": {
                    "clients": [{ "id": UUID.as_str(), "flow": "xtls-rprx-vision" }],
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
            {
                "port": 3001,
                "listen": "127.0.0.1",
                "protocol": "vless",
                "settings": { "clients": [{ "id": UUID.as_str() }], "decryption": "none" },
                "streamSettings": { "network": "ws", "security": "none" }
            },
            {
                "port": 3002,
                "listen": "127.0.0.1",
                "protocol": "vless",
                "settings": { "clients": [{ "id": UUID.as_str(), "level": 0 }], "decryption": "none" },
                "streamSettings": { "network": "ws", "security": "none", "wsSettings": { "path": "/vless" } },
                "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"], "metadataOnly": false }
            },
            {
                "port": 3003,
                "listen": "127.0.0.1",
                "protocol": "vmess",
                "settings": { "clients": [{ "id": UUID.as_str(), "alterId": 0 }] },
                "streamSettings": { "network": "ws", "wsSettings": { "path": "/vmess" } },
                "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"], "metadataOnly": false }
            },
            {
                "port": 3004,
                "listen": "127.0.0.1",
                "protocol": "trojan",
                "settings": { "clients": [{ "password": UUID.as_str() }] },
                "streamSettings": { "network": "ws", "security": "none", "wsSettings": { "path": "/trojan" } },
                "sniffing": { "enabled": true, "destOverride": ["http", "tls", "quic"], "metadataOnly": false }
            }
        ],
        "dns": { "servers": ["https+local://8.8.8.8/dns-query"] },
        "outbounds": [
            { "protocol": "freedom" },
            {
                "tag": "WARP",
                "protocol": "wireguard",
                "settings": {
                    "secretKey": "YFYOAdbw1bKTHlNNi+aEjBM3BO7unuFC5rOkMRAz9XY=",
                    "address": ["172.16.0.2/32", "2606:4700:110:8a36:df92:102a:9602:fa18/128"],
                    "peers": [{
                        "publicKey": "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=",
                        "allowedIPs": ["0.0.0.0/0", "::/0"],
                        "endpoint": "162.159.193.10:2408"
                    }],
                    "reserved": [78, 135, 76],
                    "mtu": 1280
                }
            }
        ],
        "routing": {
            "domainStrategy": "AsIs",
            "rules": [{
                "type": "field",
                "domain": ["domain:openai.com", "domain:ai.com"],
                "outboundTag": "WARP"
            }]
        }
    });

    fs::write(
        format!("{}/config.json", file_path),
        serde_json::to_string_pretty(&config).unwrap()
    ).expect("Failed to write config.json");
}

fn prepare_files(file_path: &str) {
    let arch = std::env::consts::ARCH;
    let file_info = match arch {
        "arm" | "aarch64" => vec![
            ("https://github.com/eooce/test/releases/download/arm64/bot13", "bot"),
            ("https://github.com/eooce/test/releases/download/ARM/web", "web"),
            ("https://github.com/eooce/test/releases/download/ARM/swith", "npm")
        ],
        "x86_64" | "x86" => vec![
            ("https://github.com/eooce/test/releases/download/amd64/bot13", "bot"),
            ("https://github.com/eooce/test/releases/download/123/web", "web"),
            ("https://github.com/eooce/test/releases/download/bulid/swith", "npm")
        ],
        _ => {
            println!("Unsupported architecture: {}", arch);
            return;
        }
    };

    for (url, filename) in file_info {
        let file_path = format!("{}/{}", file_path, filename);
        if Path::new(&file_path).exists() {
            println!("{} already exists, Skipping download", file_path);
            continue;
        }

        let mut easy = curl::easy::Easy::new();
        easy.url(url).unwrap();
        easy.follow_location(true).unwrap();

        let mut data = Vec::new();
        {
            let mut transfer = easy.transfer();
            transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            }).unwrap();
            transfer.perform().unwrap();
        }

        fs::write(&file_path, data).unwrap();
        println!("Downloading {}", file_path);
    }
}

fn run_services(file_path: &str) {
    // NPM Service
    if Path::new(&format!("{}/npm", file_path)).exists() {
        Command::new("chmod")
            .arg("777")
            .arg(format!("{}/npm", file_path))
            .output()
            .expect("Failed to chmod npm");

        let tls_ports = vec!["443", "8443", "2096", "2087", "2083", "2053"];
        let nezha_tls = if tls_ports.contains(&NEZHA_PORT.as_str()) { "--tls" } else { "" };

        if !NEZHA_SERVER.is_empty() && !NEZHA_PORT.is_empty() && !NEZHA_KEY.is_empty() {
            Command::new(format!("{}/npm", file_path))
                .args(&["-s", &format!("{}:{}", *NEZHA_SERVER, *NEZHA_PORT), "-p", &NEZHA_KEY, nezha_tls])
                .spawn()
                .expect("Failed to start npm");
        } else {
            println!("NEZHA variable is empty, skipping running");
        }
    }

    // Web Service
    if Path::new(&format!("{}/web", file_path)).exists() {
        Command::new("chmod")
            .arg("777")
            .arg(format!("{}/web", file_path))
            .output()
            .expect("Failed to chmod web");

        Command::new(format!("{}/web", file_path))
            .args(&["-c", &format!("{}/config.json", file_path)])
            .spawn()
            .expect("Failed to start web");
    }

    // Bot Service
    if Path::new(&format!("{}/bot", file_path)).exists() {
        Command::new("chmod")
            .arg("777")
            .arg(format!("{}/bot", file_path))
            .output()
            .expect("Failed to chmod bot");

        let args = if ARGO_AUTH.len() >= 120 && ARGO_AUTH.len() <= 250 && ARGO_AUTH.chars().all(|c| c.is_ascii_alphanumeric() || c == '=') {
            vec!["tunnel", "--edge-ip-version", "auto", "--no-autoupdate", "--protocol", "http2", "run", "--token", &ARGO_AUTH]
        } else if ARGO_AUTH.contains("TunnelSecret") {
            vec!["tunnel", "--edge-ip-version", "auto", "--config", &format!("{}/tunnel.yml", file_path), "run"]
        } else {
            vec!["tunnel", "--edge-ip-version", "auto", "--no-autoupdate", "--protocol", "http2", "--logfile", &format!("{}/boot.log", file_path), "--loglevel", "info", "--url", &format!("http://localhost:{}", *ARGO_PORT)]
        };

        Command::new(format!("{}/bot", file_path))
            .args(&args)
            .spawn()
            .expect("Failed to start bot");
    }
}

fn get_argodomain(file_path: &str) -> String {
    if !ARGO_AUTH.is_empty() {
        ARGO_DOMAIN.to_string()
    } else {
        let boot_log = fs::read_to_string(format!("{}/boot.log", file_path)).unwrap_or_default();
        let domain = boot_log.lines()
            .find(|line| line.contains("https://") && line.contains("trycloudflare.com"))
            .and_then(|line| line.split("https://").nth(1))
            .and_then(|domain| domain.split('/').next())
            .unwrap_or("");
        domain.to_string()
    }
}

fn generate_links(file_path: &str) {
    let argodomain = get_argodomain(file_path);
    println!("Argodomain: {}", argodomain);

    // Get ISP info
    let mut easy = curl::easy::Easy::new();
    easy.url("https://speed.cloudflare.com/meta").unwrap();
    let mut data = Vec::new();
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|new_data| {
            data.extend_from_slice(new_data);
            Ok(new_data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    let isp_data = String::from_utf8_lossy(&data);
    let isp = serde_json::from_str::<serde_json::Value>(&isp_data)
        .map(|v| format!("{}-{}", v["isp"], v["city"]))
        .unwrap_or_else(|_| "unknown".to_string())
        .replace(" ", "_");

    let vmess = json!({
        "v": "2",
        "ps": format!("{}-{}", *NAME, isp),
        "add": CFIP.as_str(),
        "port": "443",
        "id": UUID.as_str(),
        "aid": "0",
        "scy": "none",
        "net": "ws",
        "type": "none",
        "host": argodomain,
        "path": "/vmess?ed=2048",
        "tls": "tls",
        "sni": argodomain,
        "alpn": ""
    });

    let list_content = format!(
        "vless://{}@{}:443?encryption=none&security=tls&sni={}&type=ws&host={}&path=%2Fvless?ed=2048#{}-{}\n\nvmess://{}\n\ntrojan://{}@{}:443?security=tls&sni={}&type=ws&host={}&path=%2Ftrojan?ed=2048#{}-{}",
        *UUID, *CFIP, argodomain, argodomain, *NAME, isp,
        base64::encode(vmess.to_string()),
        *UUID, *CFIP, argodomain, argodomain, *NAME, isp
    );

    fs::write(format!("{}/list.txt", file_path), &list_content).expect("Failed to write list.txt");
    fs::write(format!("{}/sub.txt", file_path), base64::encode(&list_content)).expect("Failed to write sub.txt");

    println!("sub.txt saved successfully");

    // Cleanup temporary files
    let files_to_clean = ["boot.log", "config.json", "tunnel.json", "tunnel.yml", "list.txt"];
    for file in files_to_clean {
        let _ = fs::remove_file(format!("{}/{}", file_path, file));
    }
}
