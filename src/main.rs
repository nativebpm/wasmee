use std::sync::Arc;
use std::collections::HashMap;
use std::sync::Mutex;
use axum::{routing::post, response::IntoResponse, Router};
use wasmtime::{Engine, Module};

pub mod engine;

pub mod pb {
    include!(concat!(env!("OUT_DIR"), "/wasmee.rs"));
}

struct AppState {
    engine: Engine,
    default_module: Option<Module>,
    modules: Mutex<HashMap<String, Module>>,
    store: engine::RustStore,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = "target/wasm32-wasip1/release/wasmee_guest.wasm";
    let wasm_bytes_opt = std::fs::read(wasm_path)
        .or_else(|_| std::fs::read("../wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .or_else(|_| std::fs::read("../../wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .or_else(|_| std::fs::read("wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .ok();

    let engine = Engine::default();
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

    let state = Arc::new(AppState {
        engine,
        default_module,
        modules: Mutex::new(HashMap::new()),
        store,
    });

    let app = Router::new()
        .route("/execute", post(execute_handler))
        .layer(axum::extract::DefaultBodyLimit::disable())
        .with_state(state);

    let http_addr = "0.0.0.0:8081";
    let http_listener = tokio::net::TcpListener::bind(http_addr).await?;
    println!("Wasmee Rust HTTP execution engine listening on http://{}", http_addr);

    axum::serve(http_listener, app).await?;

    Ok(())
}

async fn execute_handler(
    axum::extract::State(state): axum::extract::State<Arc<AppState>>,
    body_bytes: axum::body::Bytes,
) -> impl axum::response::IntoResponse {
    use prost::Message;
    use sha2::{Digest, Sha256};

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

    let engine = state.engine.clone();
    let wasm_store = state.store.clone();
    let initial_call_index = 0;

    let module_res = if !payload.wasm_bytes.is_empty() {
        let mut hasher = Sha256::new();
        hasher.update(&payload.wasm_bytes);
        let hash_hex = format!("{:x}", hasher.finalize());

        let cached_module = {
            let modules = state.modules.lock().unwrap();
            modules.get(&hash_hex).cloned()
        };

        match cached_module {
            Some(m) => Ok(m),
            None => {
                match Module::new(&engine, &payload.wasm_bytes) {
                    Ok(m) => {
                        let mut modules = state.modules.lock().unwrap();
                        modules.insert(hash_hex, m.clone());
                        Ok(m)
                    }
                    Err(e) => Err(format!("Failed to compile dynamic WASM module: {}", e)),
                }
            }
        }
    } else {
        match &state.default_module {
            Some(m) => Ok(m.clone()),
            None => Err("No WASM module provided in request and no default guest module loaded on host".to_string()),
        }
    };

    let module = match module_res {
        Ok(m) => m,
        Err(err_msg) => {
            let resp = pb::ExecuteResponse {
                crashed: true,
                error: err_msg,
                final_deltas: HashMap::new(),
                final_oplog: vec![],
                checkpoints: vec![],
                response_bytes: vec![],
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
