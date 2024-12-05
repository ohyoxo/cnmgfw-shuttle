use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use once_cell::sync::Lazy;
use std::process::Command;
use std::convert::Infallible;

static PORT: Lazy<String> =
    Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

#[shuttle_runtime::main]
async fn main() -> shuttle_hyper::ShuttleHyper {
    // 执行shell脚本
    let status = Command::new("bash")
        .args(&["start.sh"])
        .status()
        .expect("Startup command failed");

    if !status.success() {
        eprintln!("Shell command execution failed");
    }

    let service = make_service_fn(|_| async {
        Ok::<_, Infallible>(service_fn(handle_request))
    });

    let router = hyper::Server::bind(&([0, 0, 0, 0], 8000).into()).serve(service);
    Ok(router.into())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let response = match req.uri().path() {
        "/" => Response::new(Body::from("Hello world")),
        "/sub" => {
            let content = std::fs::read_to_string("./temp/sub.txt")
                .unwrap_or_else(|_| String::from("File not found"));
            Response::builder()
                .header("Content-Type", "text/plain; charset=utf-8")
                .body(Body::from(content))
                .unwrap()
        }
        _ => Response::builder()
            .status(404)
            .body(Body::from("Not Found"))
            .unwrap(),
    };
    Ok(response)
}
