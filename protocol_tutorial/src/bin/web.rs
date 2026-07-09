//! 协议教程 Web 服务（Axum + 静态前端）
//!
//! 启动：cargo run --bin web
//! 指定端口：PORT=3001 cargo run --bin web

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum::routing::get_service;
use protocol_tutorial::{
    BitFinalizeResponse, BuildResponse, ProtocolForm, SimulateFullResponse, StepPreviewResponse,
    build_response, finalize_bits, prepare_bit_mode, random_form, simulate_full_transfer,
    step_preview,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, services::ServeDir};

const STATIC_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/static");

#[derive(Clone)]
struct AppState {
    index_html: Arc<String>,
    guide_html: Arc<String>,
}

const GUIDE_HTML: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/tet.html");

#[derive(Deserialize)]
struct StepPreviewRequest {
    packet_hex: String,
    bit_stream: String,
}

#[derive(Deserialize)]
struct BitFinalizeRequest {
    packet_hex: String,
    bit_stream: String,
}

#[derive(Serialize)]
struct ApiError {
    error: String,
}

#[derive(Serialize)]
struct PrepareBitsResponse {
    build: BuildResponse,
    tx_flow: String,
    total_bits: usize,
    bit_chars: String,
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    axum::response::Html(state.index_html.as_str().to_string())
}

async fn guide(State(state): State<AppState>) -> impl IntoResponse {
    axum::response::Html(state.guide_html.as_str().to_string())
}

async fn api_random() -> Json<ProtocolForm> {
    Json(random_form())
}

async fn api_build(Json(form): Json<ProtocolForm>) -> Result<Json<BuildResponse>, Response> {
    match build_response(&form) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err(api_err(e)),
    }
}

async fn api_simulate_full(
    Json(form): Json<ProtocolForm>,
) -> Result<Json<SimulateFullResponse>, Response> {
    match simulate_full_transfer(&form) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err(api_err(e)),
    }
}

async fn api_prepare_bits(Json(form): Json<ProtocolForm>) -> Result<Json<PrepareBitsResponse>, Response> {
    match prepare_bit_mode(&form) {
        Ok((build, tx_flow, total, bit_chars)) => Ok(Json(PrepareBitsResponse {
            build,
            tx_flow,
            total_bits: total,
            bit_chars,
        })),
        Err(e) => Err(api_err(e)),
    }
}

async fn api_step_preview(
    Json(req): Json<StepPreviewRequest>,
) -> Result<Json<StepPreviewResponse>, Response> {
    match step_preview(&req.packet_hex, &req.bit_stream) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err(api_err(e)),
    }
}

async fn api_finalize_bits(
    Json(req): Json<BitFinalizeRequest>,
) -> Result<Json<BitFinalizeResponse>, Response> {
    match finalize_bits(&req.packet_hex, &req.bit_stream) {
        Ok(resp) => Ok(Json(resp)),
        Err(e) => Err(api_err(e)),
    }
}

fn api_err(msg: String) -> Response {
    (StatusCode::BAD_REQUEST, Json(ApiError { error: msg })).into_response()
}

async fn bind_listener(port: u16) -> Result<TcpListener, std::io::Error> {
    TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await
}

async fn bind_with_fallback(preferred: u16, explicit: bool) -> Result<(TcpListener, u16), std::io::Error> {
    if explicit {
        return bind_listener(preferred).await.map(|l| (l, preferred));
    }
    for port in preferred..=preferred.saturating_add(10) {
        match bind_listener(port).await {
            Ok(listener) => {
                if port != preferred {
                    eprintln!("端口 {preferred} 已被占用，已自动改用 {port}");
                }
                return Ok((listener, port));
            }
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrInUse,
        format!("端口 {preferred}–{} 均已被占用", preferred.saturating_add(10)),
    ))
}

#[tokio::main]
async fn main() {
    let explicit_port = std::env::var("PORT").ok();
    let preferred: u16 = explicit_port
        .as_deref()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);

    let (listener, port) = match bind_with_fallback(preferred, explicit_port.is_some()).await {
        Ok(bound) => bound,
        Err(e) => {
            eprintln!("无法启动 Web 服务：{e}");
            eprintln!("请关闭占用端口的进程，或指定端口：PORT=3011 cargo run --bin web");
            std::process::exit(1);
        }
    };

    let index_html = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html"))
        .expect("static/index.html 缺失");
    let guide_html = std::fs::read_to_string(GUIDE_HTML)
        .expect("docs/tet.html 缺失");

    let state = AppState {
        index_html: Arc::new(index_html),
        guide_html: Arc::new(guide_html),
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/lab", get(index))
        .route("/guide", get(guide))
        .route("/api/random", post(api_random))
        .route("/api/build", post(api_build))
        .route("/api/simulate-full", post(api_simulate_full))
        .route("/api/prepare-bits", post(api_prepare_bits))
        .route("/api/step-preview", post(api_step_preview))
        .route("/api/finalize-bits", post(api_finalize_bits))
        .nest_service("/static", get_service(ServeDir::new(STATIC_DIR)))
        .layer(CorsLayer::permissive())
        .with_state(state);

    println!("协议教程 Web 版 → http://127.0.0.1:{port}/");
    println!("位域图解教程   → http://127.0.0.1:{port}/guide");
    println!("（别名 /lab 同实验台）");
    axum::serve(listener, app).await.unwrap();
}
