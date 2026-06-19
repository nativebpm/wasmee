use std::collections::HashMap;
use std::sync::Arc;
use axum::{routing::post, Json, Router};
use wasmtime::{Engine, Module};

pub mod engine;

struct AppState {
    engine: Engine,
    module: Module,
    store: engine::RustStore,
}

#[derive(serde::Deserialize)]
struct ExecuteRequest {
    instance_id: String,
    entrypoint: String,
    params: Vec<u64>,
    base_snapshot: Vec<u8>,
    #[serde(deserialize_with = "deserialize_memory_deltas")]
    memory_deltas: HashMap<i32, Vec<u8>>,
    oplog: Vec<engine::OplogEntry>,
}

fn deserialize_memory_deltas<'de, D>(deserializer: D) -> Result<HashMap<i32, Vec<u8>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let map = HashMap::<String, Vec<u8>>::deserialize(deserializer)?;
    let mut result = HashMap::new();
    for (k, v) in map {
        if let Ok(key) = k.parse::<i32>() {
            result.insert(key, v);
        }
    }
    Ok(result)
}

#[derive(serde::Serialize)]
struct ExecuteResponse {
    crashed: bool,
    error: String,
    #[serde(serialize_with = "serialize_memory_deltas")]
    final_deltas: HashMap<i32, Vec<u8>>,
    final_oplog: Vec<engine::OplogEntry>,
    checkpoints: Vec<engine::CheckpointData>,
}

fn serialize_memory_deltas<S>(map: &HashMap<i32, Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeMap;
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        map_ser.serialize_entry(&k.to_string(), v)?;
    }
    map_ser.end()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = "target/wasm32-wasip1/release/wasmee_guest.wasm";
    let wasm_bytes = std::fs::read(wasm_path)
        .or_else(|_| std::fs::read("../wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .or_else(|_| std::fs::read("../../wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .or_else(|_| std::fs::read("wasmee/target/wasm32-wasip1/release/wasmee_guest.wasm"))
        .expect("failed to read guest WASM binary");

    let engine = Engine::default();
    let module = Module::new(&engine, &wasm_bytes).expect("failed to precompile guest WASM module");
    let store = engine::RustStore::default();

    let state = Arc::new(AppState {
        engine,
        module,
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
    Json(payload): Json<ExecuteRequest>,
) -> Json<ExecuteResponse> {
    let engine = state.engine.clone();
    let module = state.module.clone();
    let wasm_store = state.store.clone();
    let initial_call_index = 0;

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
        )
    }).await.unwrap();

    Json(ExecuteResponse {
        crashed: res.crashed,
        error: res.error,
        final_deltas: res.final_deltas,
        final_oplog: res.final_oplog,
        checkpoints: res.checkpoints,
    })
}
