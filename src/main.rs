use axum::{
    routing::get,
    Router,
};
use std::process::Command;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // 执行 start.sh
    let status = Command::new("bash")
        .arg("start.sh")
        .status()
        .expect("Failed to execute start.sh");

    if !status.success() {
        panic!("start.sh execution failed");
    }

    // 创建路由
    let router = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/sub", get(handle_sub));

    Ok(router.into())
}

async fn handle_sub() -> String {
    std::fs::read_to_string("./temp/sub.txt")
        .unwrap_or_else(|_| String::from("File not found"))
}
