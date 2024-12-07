use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use once_cell::sync::Lazy;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use uuid::Uuid;
use base64::{Engine as _, engine::general_purpose};
use serde_json::{json, Value};
use tokio::time::{sleep, Duration};

// 全局常量
static UUID: Lazy<String> = Lazy::new(|| {
    env::var("UUID").unwrap_or_else(|_| "2447700e-0d8e-44c2-b9b2-6a5a73777981".to_string())
});

static NEZHA_SERVER: Lazy<String> = Lazy::new(|| {
    env::var("NEZHA_SERVER").unwrap_or_else(|_| "nz.abcd.cn".to_string())
});

static NEZHA_PORT: Lazy<String> = Lazy::new(|| {
    env::var("NEZHA_PORT").unwrap_or_else(|_| "5555".to_string())
});

static NEZHA_KEY: Lazy<String> = Lazy::new(|| {
    env::var("NEZHA_KEY").unwrap_or_else(|_| String::new())
});

static ARGO_DOMAIN: Lazy<String> = Lazy::new(|| {
    env::var("ARGO_DOMAIN").unwrap_or_else(|_| String::new())
});

static ARGO_AUTH: Lazy<String> = Lazy::new(|| {
    env::var("ARGO_AUTH").unwrap_or_else(|_| String::new())
});

static CFIP: Lazy<String> = Lazy::new(|| {
    env::var("CFIP").unwrap_or_else(|_| "ma.ma".to_string())
});

static NAME: Lazy<String> = Lazy::new(|| {
    env::var("NAME").unwrap_or_else(|_| "Vls".to_string())
});

static FILE_PATH: Lazy<String> = Lazy::new(|| {
    env::var("FILE_PATH").unwrap_or_else(|_| "./temp".to_string())
});

static ARGO_PORT: Lazy<String> = Lazy::new(|| {
    env::var("ARGO_PORT").unwrap_or_else(|_| "8001".to_string())
});

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // 创建必要的目录
    if !Path::new(&*FILE_PATH).exists() {
        fs::create_dir_all(&*FILE_PATH).expect("Failed to create directory");
    }

    // 清理旧文件
    cleanup_oldfiles();

    // 配置 Argo
    argo_configure();

    // 生成配置文件
    generate_config();

    // 下载并运行必要的文件
    download_and_run().await;

    // 生成链接
    generate_links().await;

    // 设置路由
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/sub", get(get_sub));

    Ok(router.into())
}

async fn hello_world() -> impl IntoResponse {
    "Hello, World!"
}

async fn get_sub() -> impl IntoResponse {
    match fs::read_to_string(format!("{}/sub.txt", *FILE_PATH)) {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

fn cleanup_oldfiles() {
    let files = vec![
        "boot.log", "sub.txt", "config.json", "tunnel.json", "tunnel.yml"
    ];
    for file in files {
        let path = format!("{}/{}", *FILE_PATH, file);
        let _ = fs::remove_file(path);
    }
}

fn argo_configure() {
    if ARGO_AUTH.is_empty() || ARGO_DOMAIN.is_empty() {
        println!("ARGO_DOMAIN or ARGO_AUTH variable is empty, use quick tunnels");
        return;
    }

    if ARGO_AUTH.contains("TunnelSecret") {
        let tunnel_json_path = format!("{}/tunnel.json", *FILE_PATH);
        let tunnel_yml_path = format!("{}/tunnel.yml", *FILE_PATH);
        
        // 写入 tunnel.json
        fs::write(&tunnel_json_path, &*ARGO_AUTH).expect("Failed to write tunnel.json");

        // 提取 tunnel ID
        let tunnel_id = ARGO_AUTH.split('"').nth(11).unwrap_or("");

        // 创建 tunnel.yml
        let tunnel_yml_content = format!(
            "tunnel: {}\ncredentials-file: {}/tunnel.json\nprotocol: http2\n\ningress:\n  - hostname: {}\n    service: http://localhost:{}\n    originRequest:\n      noTLSVerify: true\n  - service: http_status:404\n",
            tunnel_id, *FILE_PATH, *ARGO_DOMAIN, *ARGO_PORT
        );
        fs::write(&tunnel_yml_path, tunnel_yml_content).expect("Failed to write tunnel.yml");
    } else {
        println!("ARGO_AUTH mismatch TunnelSecret, use token connect to tunnel");
    }
}

fn generate_config() {
    let config = json!({
        "log": {
            "access": "/dev/null",
            "error": "/dev/null",
            "loglevel": "none"
        },
        "inbounds": [
            {
                "port": ARGO_PORT.parse::<i32>().unwrap(),
                "protocol": "vless",
                "settings": {
                    "clients": [
                        {
                            "id": *UUID,
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
            }
        ],
        "outbounds": [
            {
                "protocol": "freedom"
            }
        ]
    });

    let config_path = format!("{}/config.json", *FILE_PATH);
    fs::write(config_path, serde_json::to_string_pretty(&config).unwrap()).expect("Failed to write config");
}

async fn download_and_run() {
    // 这里需要实现文件下载和运行逻辑
    // 由于 Shuttle.dev 的限制，这部分可能需要修改或移除
}

async fn generate_links() {
    let argo_domain = get_argodomain().await;
    println!("Argodomain: {}", argo_domain);

    // 获取 ISP 信息
    let isp = get_isp_info().await;
    
    // 生成 VMESS 配置
    let vmess_config = json!({
        "v": "2",
        "ps": format!("{}-{}", *NAME, isp),
        "add": *CFIP,
        "port": "443",
        "id": *UUID,
        "aid": "0",
        "scy": "none",
        "net": "ws",
        "type": "none",
        "host": argo_domain,
        "path": "/vmess?ed=2048",
        "tls": "tls",
        "sni": argo_domain,
        "alpn": ""
    });

    // 生成链接列表
    let links = format!(
        "vless://{}@{}:443?encryption=none&security=tls&sni={}&type=ws&host={}&path=%2Fvless?ed=2048#{}-{}\n\nvmess://{}\n\ntrojan://{}@{}:443?security=tls&sni={}&type=ws&host={}&path=%2Ftrojan?ed=2048#{}-{}",
        *UUID, *CFIP, argo_domain, argo_domain, *NAME, isp,
        general_purpose::STANDARD.encode(vmess_config.to_string()),
        *UUID, *CFIP, argo_domain, argo_domain, *NAME, isp
    );

    // 保存链接
    let sub_path = format!("{}/sub.txt", *FILE_PATH);
    fs::write(&sub_path, general_purpose::STANDARD.encode(links)).expect("Failed to write sub.txt");
}

async fn get_argodomain() -> String {
    if !ARGO_AUTH.is_empty() {
        ARGO_DOMAIN.to_string()
    } else {
        // 这里需要实现从 boot.log 读取域名的逻辑
        String::new()
    }
}

async fn get_isp_info() -> String {
    match reqwest::get("https://speed.cloudflare.com/meta").await {
        Ok(response) => {
            if let Ok(text) = response.text().await {
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    return format!("{}-{}", 
                        json["colo"].as_str().unwrap_or(""),
                        json["asOrganization"].as_str().unwrap_or("")
                    ).replace(" ", "_");
                }
            }
            String::from("unknown")
        }
        Err(_) => String::from("unknown")
    }
}
