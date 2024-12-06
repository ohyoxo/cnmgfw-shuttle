use axum::{
    routing::get,
    Router,
    response::IntoResponse,
    http::StatusCode,
    body::Body,
};
use once_cell::sync::Lazy;
use std::{
    net::SocketAddr,
    process::Command,
    path::PathBuf,
    fs,
};
use tokio::process::Command as TokioCommand;

static PORT: Lazy<String> = Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    // Start the shell command in a separate thread
    tokio::spawn(async {
        let status = TokioCommand::new("bash")
            .args(&["start.sh"])
            .status()
            .await;

        match status {
            Ok(exit_status) => {
                if !exit_status.success() {
                    eprintln!("Shell command execution failed, please restart server");
                }
            }
            Err(e) => eprintln!("Failed to execute command: {}", e),
        }
    });

    // Create the router with our routes
    let app = Router::new()
        .route("/", get(hello_world))
        .route("/sub", get(get_sub));

    // Get the address to bind to
    let addr: SocketAddr = format!("0.0.0.0:{}", *PORT)
        .parse()
        .expect("Invalid address");

    println!("Server is running on http://{}", addr);
    println!("Thank you for using this script, enjoy!");

    // Return the router
    Ok(app.into())
}

async fn hello_world() -> &'static str {
    "Hello world"
}

async fn get_sub() -> impl IntoResponse {
    match fs::read_to_string("./temp/sub.txt") {
        Ok(content) => (StatusCode::OK, content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}
