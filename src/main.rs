use std::sync::Arc;
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, RwLock};
use axum::{routing::post, response::IntoResponse, Router};
use wasmtime::{Engine, Module};

pub mod engine;

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/wasmee.rs"));
}

struct AppState {
    engine: Engine,
    default_module: Option<Module>,
    modules: RwLock<HashMap<String, Module>>,
    modules_order: Mutex<VecDeque<String>>,
    store: engine::RustStore,
    api_token: String,
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
    let store = engine::RustStore::default();
    let api_token = std::env::var("API_TOKEN").unwrap_or_else(|_| "test-bearer-token".to_string());

    let state = Arc::new(AppState {
        engine,
        default_module,
        modules: RwLock::new(HashMap::new()),
        modules_order: Mutex::new(VecDeque::new()),
        store,
        api_token,
    });

    let app = Router::new()
        .route("/execute", post(execute_handler))
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
    use sha2::{Digest, Sha256};

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

    let payload = match pb::ExecuteRequest::decode(body_bytes) {
        Ok(req) => req,
        Err(e) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                [("content-type", "text/plain")],
                format!("Failed to decode protobuf request: {}", e).into_bytes(),
            ).into_response();
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
    let wasm_store = state.store.clone();
    let initial_call_index = 0;

    // 3. Thread Separation & JIT Cache Eviction
    enum ModuleResolveResult {
        Ok(Module),
        NotFoundInCache,
        Err(String),
    }

    let module_res = if !payload.wasm_bytes.is_empty() {
        let mut hasher = Sha256::new();
        hasher.update(&payload.wasm_bytes);
        let hash_hex = format!("{:x}", hasher.finalize());

        let cached_module = {
            let modules = state.modules.read().unwrap();
            modules.get(&hash_hex).cloned()
        };

        match cached_module {
            Some(m) => ModuleResolveResult::Ok(m),
            None => {
                let engine_clone = engine.clone();
                let wasm_bytes_clone = payload.wasm_bytes.clone();
                let compile_res = tokio::task::spawn_blocking(move || {
                    Module::new(&engine_clone, &wasm_bytes_clone)
                }).await.unwrap();

                match compile_res {
                    Ok(m) => {
                        let mut modules = state.modules.write().unwrap();
                        let mut order = state.modules_order.lock().unwrap();

                        if !modules.contains_key(&hash_hex) {
                            modules.insert(hash_hex.clone(), m.clone());
                            order.push_back(hash_hex.clone());

                            if order.len() > 100 {
                                if let Some(oldest) = order.pop_front() {
                                    modules.remove(&oldest);
                                }
                            }
                        }
                        ModuleResolveResult::Ok(m)
                    }
                    Err(e) => ModuleResolveResult::Err(format!("Failed to compile dynamic WASM module: {}", e)),
                }
            }
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
            None => ModuleResolveResult::Err("No WASM module or hash provided in request and no default guest module loaded on host".to_string()),
        }
    };

    let module = match module_res {
        ModuleResolveResult::Ok(m) => m,
        ModuleResolveResult::NotFoundInCache => {
            let resp = pb::ExecuteResponse {
                crashed: true,
                error: "Module not found in cache".to_string(),
                final_deltas: HashMap::new(),
                final_oplog: vec![],
                checkpoints: vec![],
                response_bytes: vec![],
                module_not_found: true,
            };
            let mut buf = Vec::new();
            let _ = resp.encode(&mut buf);
            return (
                axum::http::StatusCode::OK,
                [("content-type", "application/x-protobuf")],
                buf,
            ).into_response();
        }
        ModuleResolveResult::Err(err_msg) => {
            let resp = pb::ExecuteResponse {
                crashed: true,
                error: err_msg,
                final_deltas: HashMap::new(),
                final_oplog: vec![],
                checkpoints: vec![],
                response_bytes: vec![],
                module_not_found: false,
            };
            let mut buf = Vec::new();
            let _ = resp.encode(&mut buf);
            return (
                axum::http::StatusCode::OK,
                [("content-type", "application/x-protobuf")],
                buf,
            ).into_response();
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
            wasm_store,
            &payload.exchange_buffer,
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
