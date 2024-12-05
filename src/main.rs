use std::fs;
use std::path::Path;
use std::process::Command;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use once_cell::sync::Lazy;
use std::net::SocketAddr;

static PORT: Lazy<String> =
    Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

const START_SCRIPT: &str = r#"#!/bin/bash
export UUID=${UUID:-'2447700e-0d8e-44c2-b9b2-6a5a73777981'}
export NEZHA_SERVER=${NEZHA_SERVER:-''}
export NEZHA_PORT=${NEZHA_PORT:-'5555'}
export NEZHA_KEY=${NEZHA_KEY:-''}
export ARGO_DOMAIN=${ARGO_DOMAIN:-''}
export ARGO_AUTH=${ARGO_AUTH:-''}
export CFIP=${CFIP:-'ma.ma'}
export NAME=${NAME:-'Vls'}
export FILE_PATH=${FILE_PATH:-'./temp'} 
export ARGO_PORT=${ARGO_PORT:-'8001'}

if [ ! -d "${FILE_PATH}" ]; then
    mkdir -p ${FILE_PATH}
fi

cleanup_oldfiles() {
    rm -rf ${FILE_PATH}/boot.log ${FILE_PATH}/sub.txt ${FILE_PATH}/config.json ${FILE_PATH}/tunnel.json ${FILE_PATH}/tunnel.yml
}
cleanup_oldfiles
"#;

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
    println!("Starting application...");
    
    // 创建临时目录
    if !Path::new("temp").exists() {
        fs::create_dir("temp").expect("Failed to create temp directory");
    }

    // 打印当前目录内容
    println!("Current directory contents:");
    if let Ok(entries) = fs::read_dir(".") {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("- {:?}", entry.path());
            }
        }
    }

    // 创建并写入 start.sh
    println!("Creating start.sh...");
    fs::write("start.sh", START_SCRIPT).expect("Failed to write start.sh");
    
    // 设置执行权限
    println!("Setting execute permissions...");
    Command::new("chmod")
        .args(&["+x", "start.sh"])
        .status()
        .expect("Failed to chmod start.sh");

    // 执行脚本
    println!("Executing start.sh...");
    let output = Command::new("bash")
        .arg("start.sh")
        .output()
        .expect("Failed to execute start.sh");

    println!("Script stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Script stderr: {}", String::from_utf8_lossy(&output.stderr));

    // 设置服务器地址
    let addr: SocketAddr = format!("0.0.0.0:{}", *PORT)
        .parse()
        .expect("Invalid address");

    // 创建服务
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Server is running on http://{}", addr);
    println!("Thank you for using this script, enjoy!");

    tokio::select! {
        _ = tokio::spawn(server) => {},
        _ = tokio::signal::ctrl_c() => {}
    }
}
