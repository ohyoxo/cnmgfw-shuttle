use std::fs;
use std::path::Path;
use std::process::Command;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use once_cell::sync::Lazy;
use std::net::SocketAddr;
use std::env;

static PORT: Lazy<String> =
    Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

// 添加更多调试输出的脚本
const START_SCRIPT: &str = r#"#!/bin/bash
set -x  # 启用调试模式，显示执行的每个命令

# 输出环境信息
echo "=== Environment Information ==="
pwd
ls -la
echo "=== Environment Variables ==="
env
echo "=========================="

# 设置变量
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

echo "Creating directory: ${FILE_PATH}"
mkdir -p "${FILE_PATH}"

echo "Cleaning old files..."
rm -rf "${FILE_PATH}"/boot.log "${FILE_PATH}"/sub.txt "${FILE_PATH}"/config.json "${FILE_PATH}"/tunnel.json "${FILE_PATH}"/tunnel.yml

# 创建一些测试文件
echo "Creating test files..."
echo "This is a test" > "${FILE_PATH}/sub.txt"
echo "Script executed at $(date)" > "${FILE_PATH}/boot.log"

echo "Listing ${FILE_PATH} contents:"
ls -la "${FILE_PATH}"

echo "Script completed successfully"
"#;

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match req.uri().path() {
        "/" => {
            // 读取并显示更多信息
            let mut response = String::from("Server Status:\n\n");
            
            // 添加目录内容
            response.push_str("Directory contents:\n");
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        response.push_str(&format!("- {:?}\n", entry.path()));
                    }
                }
            }

            // 添加 temp 目录内容
            response.push_str("\nTemp directory contents:\n");
            if let Ok(entries) = fs::read_dir("./temp") {
                for entry in entries {
                    if let Ok(entry) = entry {
                        response.push_str(&format!("- {:?}\n", entry.path()));
                    }
                }
            }

            // 添加环境变量信息
            response.push_str("\nEnvironment Variables:\n");
            for (key, value) in env::vars() {
                if !key.contains("SECRET") && !key.contains("KEY") {
                    response.push_str(&format!("{}: {}\n", key, value));
                }
            }

            Ok(Response::new(Body::from(response)))
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

    // 执行脚本并捕获输出
    println!("Executing start.sh...");
    let output = Command::new("bash")
        .arg("start.sh")
        .output()
        .expect("Failed to execute start.sh");

    println!("Script stdout:\n{}", String::from_utf8_lossy(&output.stdout));
    println!("Script stderr:\n{}", String::from_utf8_lossy(&output.stderr));

    // 检查脚本执行状态
    if output.status.success() {
        println!("Script executed successfully");
    } else {
        println!("Script failed with exit code: {:?}", output.status.code());
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
