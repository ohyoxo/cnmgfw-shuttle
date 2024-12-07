use axum::{
    routing::get,
    Router,
    response::Response,
    http::{StatusCode, header},
    body::Body,
};
use std::process::Command;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use once_cell::sync::Lazy;
use tower_http::services::ServeDir;

static PORT: Lazy<String> = Lazy::new(|| std::env::var("PORT").unwrap_or_else(|_| "3000".to_string()));

async fn hello() -> &'static str {
    "Hello World!"
}

async fn sub() -> Response<Body> {
    match std::fs::read_to_string("./temp/sub.txt") {
        Ok(content) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(content))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("File not found"))
            .unwrap(),
    }
}

fn setup_environment() -> Result<(), Box<dyn std::error::Error>> {
    println!("Current working directory: {:?}", std::env::current_dir()?);
    
    // 列出当前目录内容
    println!("Directory contents:");
    for entry in fs::read_dir(".")? {
        let entry = entry?;
        println!("{:?}", entry.path());
    }

    // 确保 temp 目录存在
    if !std::path::Path::new("temp").exists() {
        println!("Creating temp directory");
        fs::create_dir("temp")?;
    }
    
    // 设置 temp 目录权限为 755
    println!("Setting temp directory permissions");
    fs::set_permissions("temp", fs::Permissions::from_mode(0o755))?;
    
    // 检查并创建 start.sh
    if !std::path::Path::new("start.sh").exists() {
        println!("Creating start.sh");
        // 将脚本内容嵌入到代码中
        let script_content = r#"#!/bin/bash
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

if [ ! -d "${FILE_PATH}" ]; then
    mkdir ${FILE_PATH}
fi

cleanup_oldfiles() {
  rm -rf ${FILE_PATH}/boot.log ${FILE_PATH}/sub.txt ${FILE_PATH}/config.json ${FILE_PATH}/tunnel.json ${FILE_PATH}/tunnel.yml
}
cleanup_oldfiles
sleep 2

argo_configure() {
  if [[ -z $ARGO_AUTH || -z $ARGO_DOMAIN ]]; then
    echo -e "\e[1;32mARGO_DOMAIN or ARGO_AUTH variable is empty, use quick tunnels\e[0m"
    return
  fi

  if [[ $ARGO_AUTH =~ TunnelSecret ]]; then
    echo $ARGO_AUTH > ${FILE_PATH}/tunnel.json
    cat > ${FILE_PATH}/tunnel.yml << EOF
tunnel: $(cut -d\" -f12 <<< "$ARGO_AUTH")
credentials-file: ${FILE_PATH}/tunnel.json
protocol: http2
EOF
  fi
}
"#;
        fs::write("start.sh", script_content)?;
    }
    
    // 设置 start.sh 权限为 755
    println!("Setting start.sh permissions");
    fs::set_permissions("start.sh", fs::Permissions::from_mode(0o755))?;
    
    Ok(())
}

#[shuttle_runtime::main]
async fn axum() -> shuttle_axum::ShuttleAxum {
    // 设置环境
    if let Err(e) = setup_environment() {
        eprintln!("Environment setup failed: {}", e);
        std::process::exit(1);
    }

    println!("Executing start.sh");
    // 执行启动脚本
    let output = Command::new("bash")
        .args(&["start.sh"])
        .output()
        .expect("Failed to execute start.sh");

    println!("start.sh stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("start.sh stderr: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        eprintln!("Shell command execution failed");
    }

    // 创建路由
    let router = Router::new()
        .route("/", get(hello))
        .route("/sub", get(sub))
        .nest_service("/static", ServeDir::new("static"));

    Ok(router.into())
}
