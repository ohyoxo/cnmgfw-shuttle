use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use std::process::Command;
use std::net::SocketAddr;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // 执行shell脚本
    let status = Command::new("bash")
        .args(&["start.sh"])
        .status()
        .expect("Failed to execute command");

    if !status.success() {
        eprintln!("Shell command failed");
    }

    // 创建路由
    let router = Router::new()
        .route("/", get(hello))
        .route("/sub", get(get_sub));

    Ok(router.into())
}

async fn hello() -> &'static str {
    "Hello World!"
}

async fn get_sub() -> impl IntoResponse {
    match std::fs::read_to_string("./temp/sub.txt") {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}
