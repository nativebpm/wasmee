use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, RwLock};
use axum::{routing::post, response::IntoResponse, Router};
use wasmtime::{Engine, Module};
use tower_http::cors::{CorsLayer, Any};

pub mod engine;
pub mod git_resolver;

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/wasmee.rs"));
}

struct AppState {
    engine: Engine,
    default_module: Option<Module>,
    modules: RwLock<HashMap<String, Module>>,
    modules_order: Mutex<VecDeque<String>>,
    compiling: Mutex<HashMap<String, Arc<tokio::sync::Notify>>>,
    api_token: String,
}

impl AppState {
    async fn get_or_compile_module(&self, wasm_bytes: Vec<u8>) -> Result<(Module, String), String> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&wasm_bytes);
        let hash_hex = format!("{:x}", hasher.finalize());

        let cached_module = {
            let modules = self.modules.read().unwrap();
            modules.get(&hash_hex).cloned()
        };

        match cached_module {
            Some(m) => Ok((m, hash_hex)),
            None => {
                let notify = {
                    let mut compiling = self.compiling.lock().unwrap();
                    if let Some(n) = compiling.get(&hash_hex) {
                        Some(n.clone())
                    } else {
                        let n = Arc::new(tokio::sync::Notify::new());
                        compiling.insert(hash_hex.clone(), n);
                        None
                    }
                };

                if let Some(n) = notify {
                    n.notified().await;
                    let modules = self.modules.read().unwrap();
                    match modules.get(&hash_hex).cloned() {
                        Some(m) => Ok((m, hash_hex)),
                        None => Err("Concurrent compilation failed".to_string()),
                    }
                } else {
                    let engine_clone = self.engine.clone();
                    let wasm_bytes_clone = wasm_bytes.clone();
                    let compile_res = tokio::task::spawn_blocking(move || {
                        Module::new(&engine_clone, &wasm_bytes_clone)
                    }).await.unwrap();

                    let notify_waiters = {
                        let mut compiling = self.compiling.lock().unwrap();
                        compiling.remove(&hash_hex)
                    };

                    let res = match compile_res {
                        Ok(m) => {
                            let mut modules = self.modules.write().unwrap();
                            let mut order = self.modules_order.lock().unwrap();

                            if !modules.contains_key(&hash_hex) {
                                modules.insert(hash_hex.clone(), m.clone());
                                order.push_back(hash_hex.clone());

                                if order.len() > 100 {
                                    if let Some(oldest) = order.pop_front() {
                                        modules.remove(&oldest);
                                    }
                                }
                            }
                            Ok((m, hash_hex))
                        }
                        Err(e) => Err(format!("Failed to compile dynamic WASM module: {}", e)),
                    };

                    if let Some(nw) = notify_waiters {
                        nw.notify_waiters();
                    }
                    res
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = "target/wasm32-wasip1/release/wasmee_guest.wasm";
    let wasm_bytes_opt = std::fs::read(wasm_path)
        .or_else(|_| std::fs::read("../wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .or_else(|_| std::fs::read("../../wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .or_else(|_| std::fs::read("wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .ok();

    let mut config = wasmtime::Config::new();
    config.consume_fuel(true);
    config.cranelift_opt_level(wasmtime::OptLevel::Speed);
    config.parallel_compilation(true);
    let engine = Engine::new(&config).expect("failed to initialize Wasmtime engine");
    let default_module = match wasm_bytes_opt {
        Some(bytes) => match Module::new(&engine, &bytes) {
            Ok(m) => Some(m),
            Err(e) => {
                eprintln!("Warning: failed to compile default guest WASM module: {}", e);
                None
            }
        },
        None => {
            println!("Warning: wasmee_guest.wasm not found. Running without default module.");
            None
        }
    };
    let api_token = std::env::var("API_TOKEN").unwrap_or_else(|_| "test-bearer-token".to_string());

    let state = Arc::new(AppState {
        engine,
        default_module,
        modules: RwLock::new(HashMap::new()),
        modules_order: Mutex::new(VecDeque::new()),
        compiling: Mutex::new(HashMap::new()),
        api_token,
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/execute", post(execute_handler))
        .route("/warmup", post(warmup_handler))
        .layer(cors)
        .layer(axum::extract::DefaultBodyLimit::max(20 * 1024 * 1024))
        .with_state(state);

    let http_addr = "0.0.0.0:8081";
    let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
    println!("Wasmee Rust HTTP execution engine listening on http://{}", http_addr);

    axum::serve(http_listener, app).await?;

    Ok(())
}

async fn execute_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body_bytes: axum::body::Bytes,
) -> impl axum::response::IntoResponse {
    use prost::Message;

    // 1. Enforce Token Authentication
    let auth_header = headers.get(axum::http::header::AUTHORIZATION);
    let is_auth = if let Some(val) = auth_header {
        if let Ok(val_str) = val.to_str() {
            val_str == format!("Bearer {}", state.api_token)
        } else {
            false
        }
    } else {
        false
    };

    if !is_auth {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            [("content-type", "text/plain")],
            b"Unauthorized: invalid or missing bearer token".to_vec(),
        ).into_response();
    }

    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_json = content_type.contains("application/json");

    let payload = if is_json {
        match serde_json::from_slice::<pb::ExecuteRequest>(&body_bytes) {
            Ok(req) => req,
            Err(e) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    [("content-type", "text/plain")],
                    format!("Failed to decode JSON request: {}", e).into_bytes(),
                ).into_response();
            }
        }
    } else {
        match pb::ExecuteRequest::decode(body_bytes) {
            Ok(req) => req,
            Err(e) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    [("content-type", "text/plain")],
                    format!("Failed to decode protobuf request: {}", e).into_bytes(),
                ).into_response();
            }
        }
    };

    // 2. Enforce Max Payload Limit (15MB)
    if payload.wasm_bytes.len() > 15 * 1024 * 1024 {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            [("content-type", "text/plain")],
            b"Payload size exceeds 15MB limit".to_vec(),
        ).into_response();
    }

    let engine = state.engine.clone();
    let initial_call_index = 0;

    // 3. Thread Separation & JIT Cache Eviction
    enum ModuleResolveResult {
        Ok(Module),
        NotFoundInCache,
        Err(String),
    }

    let module_res = if !payload.wasm_bytes.is_empty() {
        match state.get_or_compile_module(payload.wasm_bytes).await {
            Ok((m, _hash)) => ModuleResolveResult::Ok(m),
            Err(e) => ModuleResolveResult::Err(e),
        }
    } else if let Some(git_src) = payload.git_source {
        match git_resolver::resolve_git_source(&git_src).await {
            Ok(bytes) => {
                match state.get_or_compile_module(bytes).await {
                    Ok((m, _hash)) => ModuleResolveResult::Ok(m),
                    Err(e) => ModuleResolveResult::Err(e),
                }
            }
            Err(e) => ModuleResolveResult::Err(format!("Git resolution failed: {}", e)),
        }
    } else if !payload.wasm_hash.is_empty() {
        let cached_module = {
            let modules = state.modules.read().unwrap();
            modules.get(&payload.wasm_hash).cloned()
        };
        match cached_module {
            Some(m) => ModuleResolveResult::Ok(m),
            None => ModuleResolveResult::NotFoundInCache,
        }
    } else {
        match &state.default_module {
            Some(m) => ModuleResolveResult::Ok(m.clone()),
            None => ModuleResolveResult::Err("No WASM module, Git source, or hash provided in request and no default guest module loaded on host".to_string()),
        }
    };

    let send_response = |resp: pb::ExecuteResponse| {
        if is_json {
            match serde_json::to_vec(&resp) {
                Ok(buf) => (
                    axum::http::StatusCode::OK,
                    [("content-type", "application/json")],
                    buf,
                ).into_response(),
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    [("content-type", "text/plain")],
                    format!("Failed to encode JSON response: {}", e).into_bytes(),
                ).into_response(),
            }
        } else {
            let mut buf = Vec::new();
            if let Err(e) = resp.encode(&mut buf) {
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    [("content-type", "text/plain")],
                    format!("Failed to encode protobuf response: {}", e).into_bytes(),
                ).into_response();
            }
            (
                axum::http::StatusCode::OK,
                [("content-type", "application/x-protobuf")],
                buf,
            ).into_response()
        }
    };

    let module = match module_res {
        ModuleResolveResult::Ok(m) => m,
        ModuleResolveResult::NotFoundInCache => {
            return send_response(pb::ExecuteResponse {
                crashed: true,
                error: "Module not found in cache".to_string(),
                final_deltas: HashMap::new(),
                final_oplog: vec![],
                checkpoints: vec![],
                response_bytes: vec![],
                module_not_found: true,
            });
        }
        ModuleResolveResult::Err(err_msg) => {
            return send_response(pb::ExecuteResponse {
                crashed: true,
                error: err_msg,
                final_deltas: HashMap::new(),
                final_oplog: vec![],
                checkpoints: vec![],
                response_bytes: vec![],
                module_not_found: false,
            });
        }
    };

    let res = tokio::task::spawn_blocking(move || {
        engine::run_wasm_precompiled(
            &engine,
            &module,
            &payload.entrypoint,
            &payload.params,
            payload.instance_id,
            &payload.base_snapshot,
            &payload.memory_deltas,
            payload.oplog,
            initial_call_index,
            &payload.exchange_buffer,
            payload.sandbox_config.as_ref(),
        )
    }).await.unwrap();

    let resp = pb::ExecuteResponse {
        crashed: res.crashed,
        error: res.error,
        final_deltas: res.final_deltas,
        final_oplog: res.final_oplog,
        checkpoints: res.checkpoints,
        response_bytes: res.response_bytes,
        module_not_found: false,
    };

    send_response(resp)
}

async fn warmup_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    body_bytes: axum::body::Bytes,
) -> impl axum::response::IntoResponse {
    use prost::Message;

    // 1. Enforce Token Authentication
    let auth_header = headers.get(axum::http::header::AUTHORIZATION);
    let is_auth = if let Some(val) = auth_header {
        if let Ok(val_str) = val.to_str() {
            val_str == format!("Bearer {}", state.api_token)
        } else {
            false
        }
    } else {
        false
    };

    if !is_auth {
        return (
            axum::http::StatusCode::UNAUTHORIZED,
            [("content-type", "text/plain")],
            b"Unauthorized: invalid or missing bearer token".to_vec(),
        ).into_response();
    }

    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let is_json = content_type.contains("application/json");

    let payload = if is_json {
        match serde_json::from_slice::<pb::WarmupRequest>(&body_bytes) {
            Ok(req) => req,
            Err(e) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    [("content-type", "text/plain")],
                    format!("Failed to decode JSON request: {}", e).into_bytes(),
                ).into_response();
            }
        }
    } else {
        match pb::WarmupRequest::decode(body_bytes) {
            Ok(req) => req,
            Err(e) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    [("content-type", "text/plain")],
                    format!("Failed to decode protobuf request: {}", e).into_bytes(),
                ).into_response();
            }
        }
    };

    let send_response = |resp: pb::WarmupResponse| {
        if is_json {
            match serde_json::to_vec(&resp) {
                Ok(buf) => (
                    axum::http::StatusCode::OK,
                    [("content-type", "application/json")],
                    buf,
                ).into_response(),
                Err(e) => (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    [("content-type", "text/plain")],
                    format!("Failed to encode JSON response: {}", e).into_bytes(),
                ).into_response(),
            }
        } else {
            let mut buf = Vec::new();
            let _ = resp.encode(&mut buf);
            (
                axum::http::StatusCode::OK,
                [("content-type", "application/x-protobuf")],
                buf,
            ).into_response()
        }
    };

    let git_src = match payload.git_source {
        Some(src) => src,
        None => {
            return send_response(pb::WarmupResponse {
                success: false,
                wasm_hash: "".to_string(),
                error: "Missing git_source in request".to_string(),
            });
        }
    };

    match git_resolver::resolve_git_source(&git_src).await {
        Ok(bytes) => {
            match state.get_or_compile_module(bytes).await {
                Ok((_module, hash)) => {
                    send_response(pb::WarmupResponse {
                        success: true,
                        wasm_hash: hash,
                        error: "".to_string(),
                    })
                }
                Err(e) => {
                    send_response(pb::WarmupResponse {
                        success: false,
                        wasm_hash: "".to_string(),
                        error: format!("Failed to compile module: {}", e),
                    })
                }
            }
        }
        Err(e) => {
            send_response(pb::WarmupResponse {
                success: false,
                wasm_hash: "".to_string(),
                error: format!("Git resolution failed: {}", e),
            })
        }
    }
}
