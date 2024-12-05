use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::process::Command;

static PORT: Lazy<String> =
    Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));  // Define the http service port

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/" => Ok(Response::new(Body::from("Hello world"))),
        "/sub" => {
            let content = std::fs::read_to_string("./temp/sub.txt")
                .unwrap_or_else(|_| String::from("File not found"));
            Ok(Response::builder()
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Body::from(content))
                .unwrap())
        }
        _ => Ok(Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() {
    // Start the shell command
    let command = "bash";
    let args = &["start.sh"];
    let mut child = Command::new(command)
        .args(args)
        .spawn()
        .expect("Startup command failed");

    // Wait for the shell command to finish
    let status = child.wait().expect("Wait for child process failure");
    if !status.success() {
        eprintln!("Shell command execution failed, please restart server");
        std::process::exit(1);
    }

    let addr: SocketAddr = format!("0.0.0.0:{}", *PORT)
        .parse()
        .expect("Invalid address");

    // Create a closure to handle HTTP requests
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server is running on http://{}", addr);
    println!("Thank you for using this script, enjoy!");

    tokio::select! {
        _ = tokio::spawn(server) => {},
        _ = tokio::signal::ctrl_c() => {}
    }
}
