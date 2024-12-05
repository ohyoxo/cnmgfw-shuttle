use axum::{
    routing::get,
    Router,
};
use std::process::Command;
use once_cell::sync::Lazy;

static PORT: Lazy<String> = Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // 执行启动脚本
    let status = Command::new("bash")
        .arg("start.sh")
        .status()
        .expect("Failed to execute start.sh");

    if !status.success() {
        eprintln!("Shell command execution failed");
    }

    // 创建路由
    let router = Router::new()
        .route("/", get(|| async { "Hello world" }))
        .route("/sub", get(handle_sub));

    Ok(router.into())
}

async fn handle_sub() -> String {
    std::fs::read_to_string("./temp/sub.txt").unwrap_or_else(|_| String::from("File not found"))
}
