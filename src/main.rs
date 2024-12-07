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
use uuid::Uuid;

// 全局常量
static UUID: Lazy<String> = Lazy::new(|| {
    env::var("UUID").unwrap_or_else(|_| Uuid::new_v4().to_string())
});

static ARGO_PORT: Lazy<String> = Lazy::new(|| {
    env::var("ARGO_PORT").unwrap_or_else(|_| "8001".to_string())
});

static FILE_PATH: Lazy<String> = Lazy::new(|| {
    env::var("FILE_PATH").unwrap_or_else(|_| "./temp".to_string())
});

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // 创建必要的目录
    if !Path::new(&*FILE_PATH).exists() {
        fs::create_dir_all(&*FILE_PATH).expect("Failed to create directory");
    }

    // 生成配置文件
    generate_config();

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

fn generate_config() {
    let config = format!(r#"{{
        "log": {{ "access": "/dev/null", "error": "/dev/null", "loglevel": "none" }},
        "inbounds": [
            {{
                "port": {},
                "protocol": "vless",
                "settings": {{
                    "clients": [{{ "id": "{}", "flow": "xtls-rprx-vision" }}],
                    "decryption": "none",
                    "fallbacks": [
                        {{ "dest": 3001 }},
                        {{ "path": "/vless", "dest": 3002 }},
                        {{ "path": "/vmess", "dest": 3003 }},
                        {{ "path": "/trojan", "dest": 3004 }}
                    ]
                }},
                "streamSettings": {{ "network": "tcp" }}
            }}
        ],
        "outbounds": [
            {{ "protocol": "freedom" }}
        ]
    }}"#, *ARGO_PORT, *UUID);

    let config_path = format!("{}/config.json", *FILE_PATH);
    let mut file = File::create(&config_path).expect("Failed to create config file");
    file.write_all(config.as_bytes()).expect("Failed to write config");
}
