use axum::{
    routing::get,
    Router,
    response::Response,
    http::{StatusCode, header},
    body::Body,
};
use std::process::Command;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use once_cell::sync::Lazy;
use tower_http::services::ServeDir;

static PORT: Lazy<String> = Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

async fn hello() -> &'static str {
    "Hello World!"
}

async fn sub() -> Response<Body> {
    match std::fs::read_to_string("./temp/sub.txt") {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(content))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("File not found"))
            .unwrap(),
    }
}

fn setup_environment() -> Result<(), Box<dyn std::error::Error>> {
    // 确保 temp 目录存在
    if !std::path::Path::new("temp").exists() {
        fs::create_dir("temp")?;
    }
    
    // 设置 temp 目录权限为 755
    fs::set_permissions("temp", fs::Permissions::from_mode(0o755))?;
    
    // 确保 start.sh 存在
    if !std::path::Path::new("start.sh").exists() {
        return Err("start.sh not found".into());
    }
    
    // 设置 start.sh 权限为 755
    fs::set_permissions("start.sh", fs::Permissions::from_mode(0o755))?;
    
    Ok(())
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    // 设置环境
    if let Err(e) = setup_environment() {
        eprintln!("Environment setup failed: {}", e);
        std::process::exit(1);
    }

    // 执行启动脚本
    let status = Command::new("bash")
        .args(&["start.sh"])
        .status()
        .expect("Failed to execute start.sh");

    if !status.success() {
        eprintln!("Shell command execution failed");
    }

    // 创建路由
    let router = Router::new()
        .route("/", get(hello))
        .route("/sub", get(sub))
        .nest_service("/static", ServeDir::new("static"));

    Ok(router.into())
}
