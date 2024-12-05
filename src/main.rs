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
    
    // 打印所有环境变量
    println!("Environment variables:");
    for (key, value) in env::vars() {
        println!("{}: {}", key, value);
    }
    
    // 创建临时目录
    if !Path::new("temp").exists() {
        fs::create_dir("temp").expect("Failed to create temp directory");
    }

    // 添加更多调试信息到脚本
    let debug_script = r#"#!/bin/bash
set -x  # 启用调试模式
echo "Debug: Script started" > ./temp/debug.log
env >> ./temp/debug.log
echo "Debug: Current directory: $(pwd)" >> ./temp/debug.log
echo "Debug: Directory listing:" >> ./temp/debug.log
ls -la >> ./temp/debug.log

export UUID=${UUID:-'2447700e-0d8e-44c2-b9b2-6a5a73777981'}
export NEZHA_SERVER=${NEZHA_SERVER:-'nz.abcd.cn'}
export NEZHA_PORT=${NEZHA_PORT:-'5555'}
export NEZHA_KEY=${NEZHA_KEY:-''}
export ARGO_DOMAIN=${ARGO_DOMAIN:-''}
export ARGO_AUTH=${ARGO_AUTH:-''}
export CFIP=${CFIP:-'ma.ma'}
export NAME=${NAME:-'Vls'}
export FILE_PATH=${FILE_PATH:-'./temp'} 
export ARGO_PORT=${ARGO_PORT:-'8001'}

echo "Debug: Environment variables set" >> ./temp/debug.log

if [ ! -d "${FILE_PATH}" ]; then
    mkdir -p ${FILE_PATH}
    echo "Debug: Created FILE_PATH directory" >> ./temp/debug.log
fi

cleanup_oldfiles() {
    echo "Debug: Cleaning up old files" >> ./temp/debug.log
    rm -rf ${FILE_PATH}/boot.log ${FILE_PATH}/sub.txt ${FILE_PATH}/config.json ${FILE_PATH}/tunnel.json ${FILE_PATH}/tunnel.yml
    echo "Debug: Cleanup complete" >> ./temp/debug.log
}
cleanup_oldfiles

echo "Debug: Script completed" >> ./temp/debug.log
"#;

    // 创建并写入 start.sh
    println!("Creating start.sh...");
    fs::write("start.sh", debug_script).expect("Failed to write start.sh");
    
    // 打印脚本内容
    println!("start.sh contents:");
    println!("{}", debug_script);
    
    // 设置执行权限
    println!("Setting execute permissions...");
    let chmod_output = Command::new("chmod")
        .args(&["+x", "start.sh"])
        .output()
        .expect("Failed to chmod start.sh");
    println!("chmod stdout: {}", String::from_utf8_lossy(&chmod_output.stdout));
    println!("chmod stderr: {}", String::from_utf8_lossy(&chmod_output.stderr));

    // 执行脚本
    println!("Executing start.sh...");
    let output = Command::new("bash")
        .arg("start.sh")
        .output()
        .expect("Failed to execute start.sh");

    println!("Script stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Script stderr: {}", String::from_utf8_lossy(&output.stderr));

    // 尝试读取调试日志
    if let Ok(debug_log) = fs::read_to_string("./temp/debug.log") {
        println!("Debug log contents:");
        println!("{}", debug_log);
    } else {
        println!("Failed to read debug log");
    }

    // 列出 temp 目录内容
    println!("temp directory contents:");
    if let Ok(entries) = fs::read_dir("temp") {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("- {:?}", entry.path());
            }
        }
    }

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

// 修改 handle_request 函数以显示调试信息
async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/" => {
            // 尝试读取调试日志
            let debug_info = if let Ok(debug_log) = fs::read_to_string("./temp/debug.log") {
                debug_log
            } else {
                "Debug log not found".to_string()
            };
            
            let response_text = format!("Hello world\n\nDebug Information:\n{}", debug_info);
            Ok(Response::new(Body::from(response_text)))
        },
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
