use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
};
use std::process::Command;
use std::net::SocketAddr;
use once_cell::sync::Lazy;
use shuttle_service::ShuttleAxum;

static PORT: Lazy<String> = 
    Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));  // 定义 http 服务端口

#[shuttle_runtime::main]
async fn axum() -> ShuttleAxum {
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

    let addr: SocketAddr = format!("0.0.0.0:{}", *PORT)
        .parse()
        .expect("无效地址");

    println!("服务器运行在 http://{}", addr);
    println!("感谢使用此脚本，祝您使用愉快！");

    let router = Router::new()
        .route("/", get(hello_world))
        .route("/sub", get(get_sub));

    Ok(router.into())
}

async fn hello_world() -> &'static str {
    "Hello World!"
}

async fn get_sub() -> impl IntoResponse {
    match std::fs::read_to_string("./temp/sub.txt") {
        Ok(content) => {
            axum::response::Response::builder()
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(content)
                .unwrap()
                .into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_hello_world() {
        let app = Router::new().route("/", get(hello_world));
        let response = app
            .oneshot(Request::builder().uri("/").body(()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_sub() {
        let app = Router::new().route("/sub", get(get_sub));
        let response = app
            .oneshot(Request::builder().uri("/sub").body(()).unwrap())
            .await
            .unwrap();
        
        // 如果 sub.txt 不存在，应该返回 404
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
