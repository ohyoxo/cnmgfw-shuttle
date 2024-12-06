use axum::{
    routing::get,
    Router,
};
use std::process::Command;
use once_cell::sync::Lazy;

static PORT: Lazy<String> =
    Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // 启动 shell 命令
    let command = "bash";
    let args = &["start.sh"];
    let mut child = Command::new(command)
        .args(args)
        .spawn()
        .expect("启动命令失败");

    // 等待 shell 命令完成
    let status = child.wait().expect("等待子进程失败");
    if !status.success() {
        eprintln!("Shell 命令执行失败，请重启服务器");
        std::process::exit(1);
    }

    println!("服务器运行在端口 {}", *PORT);
    println!("感谢使用此脚本，祝您使用愉快！");

    // 设置路由
    let router = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .route("/sub", get(handle_sub));

    Ok(router.into())
}

async fn handle_sub() -> String {
    std::fs::read_to_string("./temp/sub.txt")
        .unwrap_or_else(|_| String::from("File not found"))
}
