use std::collections::HashMap;
use wasmtime::{Caller, Engine, Linker, Module, Store};
use crate::pb::{OplogEntry, CheckpointData};


pub const PAGE_SIZE: usize = 65536;

pub mod base64_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = BASE64.encode(bytes);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        BASE64.decode(s).map_err(serde::de::Error::custom)
    }
}

pub fn hash_page(data: &[u8]) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

pub fn calculate_deltas(
    current: &[u8],
    previous_hashes: &HashMap<i32, u64>,
) -> (HashMap<i32, Vec<u8>>, HashMap<i32, u64>) {
    let mut deltas = HashMap::new();
    let mut new_hashes = HashMap::new();
    let num_pages = (current.len() + PAGE_SIZE - 1) / PAGE_SIZE;

    for i in 0..num_pages {
        let start = i * PAGE_SIZE;
        let mut end = start + PAGE_SIZE;
        if end > current.len() {
            end = current.len();
        }

        let page_data = &current[start..end];
        let h = hash_page(page_data);
        new_hashes.insert(i as i32, h);

        let prev_hash = previous_hashes.get(&(i as i32));
        if prev_hash.is_none() || *prev_hash.unwrap() != h {
            deltas.insert(i as i32, page_data.to_vec());
        }
    }
    (deltas, new_hashes)
}

pub fn restore_memory(base: &[u8], deltas: &HashMap<i32, Vec<u8>>) -> Vec<u8> {
    let mut max_page = -1;
    for &p in deltas.keys() {
        if p > max_page {
            max_page = p;
        }
    }

    let needed_size = ((max_page + 1) as usize) * PAGE_SIZE;
    let base_size = base.len();
    let final_size = std::cmp::max(needed_size, base_size);

    let mut restored = vec![0u8; final_size];
    restored[..base_size].copy_from_slice(base);

    for (&p, data) in deltas {
        let offset = (p as usize) * PAGE_SIZE;
        restored[offset..offset + data.len()].copy_from_slice(data);
    }
    restored
}


pub struct VMState {
    pub instance_id: String,
    pub call_index: i32,
    pub oplog: Vec<OplogEntry>,
    pub page_hashes: HashMap<i32, u64>,
    pub checkpoints: Vec<CheckpointData>,
    pub limits: wasmtime::StoreLimits,
}

pub struct RunResult {
    pub final_oplog: Vec<OplogEntry>,
    pub final_deltas: HashMap<i32, Vec<u8>>,
    pub checkpoints: Vec<CheckpointData>,
    pub crashed: bool,
    pub error: String,
    pub response_bytes: Vec<u8>,
}

pub fn run_wasm(
    wasm_bytes: &[u8],
    entrypoint: &str,
    params: &[u64],
    instance_id: String,
    base_snapshot: &[u8],
    memory_deltas: &HashMap<i32, Vec<u8>>,
    initial_oplog: Vec<OplogEntry>,
    initial_call_index: i32,
    exchange_buffer: &[u8],
) -> RunResult {
    let mut config = wasmtime::Config::new();
    config.consume_fuel(true);
    let engine = match Engine::new(&config) {
        Ok(e) => e,
        Err(e) => return RunResult {
            final_oplog: vec![],
            final_deltas: HashMap::new(),
            checkpoints: vec![],
            crashed: true,
            error: format!("Failed to create engine: {}", e),
            response_bytes: vec![],
        },
    };
    let module = match Module::new(&engine, wasm_bytes) {

        Ok(m) => m,
        Err(e) => return RunResult {
            final_oplog: vec![],
            final_deltas: HashMap::new(),
            checkpoints: vec![],
            crashed: true,
            error: format!("Failed to compile module: {}", e),
            response_bytes: vec![],
        },
    };
    run_wasm_precompiled(
        &engine,
        &module,
        entrypoint,
        params,
        instance_id,
        base_snapshot,
        memory_deltas,
        initial_oplog,
        initial_call_index,
        exchange_buffer,
        None,
    )
}

pub fn run_wasm_precompiled(
    engine: &Engine,
    module: &Module,
    entrypoint: &str,
    params: &[u64],
    instance_id: String,
    base_snapshot: &[u8],
    memory_deltas: &HashMap<i32, Vec<u8>>,
    initial_oplog: Vec<OplogEntry>,
    initial_call_index: i32,
    exchange_buffer: &[u8],
    sandbox_config: Option<&crate::pb::SandboxConfig>,
) -> RunResult {
    let mut linker = Linker::new(engine);

    // Register WASI placeholders
    linker.func_wrap("wasi_snapshot_preview1", "proc_exit", |exit_code: i32| {
        println!("Guest exited with code: {}", exit_code);
    }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "fd_write", |_: i32, _: i32, _: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "fd_close", |_: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "fd_seek", |_: i32, _: i64, _: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "environ_get", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "environ_sizes_get", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "clock_time_get", |_: i32, _: i64, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "args_sizes_get", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "args_get", |_: i32, _: i32| -> i32 { 0 }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "random_get", |mut caller: Caller<'_, VMState>, buf_ptr: i32, buf_len: i32| -> i32 {
        let mem = match caller.get_export("memory") {
            Some(wasmtime::Extern::Memory(m)) => m,
            _ => return -1,
        };
        let dummy = vec![0u8; buf_len as usize];
        mem.write(&mut caller, buf_ptr as usize, &dummy).unwrap();
        0
    }).unwrap();
    linker.func_wrap("wasi_snapshot_preview1", "sched_yield", || -> i32 { 0 }).unwrap();

    // Register host imports in the "env" module
    linker.func_wrap("env", "checkpoint", |mut caller: Caller<'_, VMState>| {
        let mem = match caller.get_export("memory") {
            Some(wasmtime::Extern::Memory(m)) => m,
            _ => panic!("WASM memory export not found"),
        };

        let data = mem.data(&caller).to_vec();

        // Save full snapshot in intermediate checkpoints list
        let state = caller.data_mut();
        let oplog_len = state.oplog.len();
        state.checkpoints.push(CheckpointData {
            memory: data.clone(),
            oplog_len: oplog_len as i32,
        });

        // Calculate and update page hashes
        let (_, new_hashes) = calculate_deltas(&data, &state.page_hashes);
        state.page_hashes = new_hashes;
    }).unwrap();

    linker.func_wrap("env", "host_get_time", |mut caller: Caller<'_, VMState>| -> i64 {
        let state = caller.data_mut();
        state.call_index += 1;
        let call_idx = state.call_index;

        // Check oplog replay
        if (call_idx - 1) < state.oplog.len() as i32 {
            let entry = &state.oplog[(call_idx - 1) as usize];
            if entry.api_name == "host_get_time" {
                let time_str = String::from_utf8_lossy(&entry.response_payload);
                return time_str.parse::<i64>().unwrap_or(0);
            }
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;

        let entry = OplogEntry {
            call_index: call_idx,
            api_name: "host_get_time".to_string(),
            request_payload: vec![],
            response_payload: now.to_string().into_bytes(),
        };

        let state = caller.data_mut();
        state.oplog.push(entry.clone());

        now
    }).unwrap();

    linker.func_wrap("env", "host_call_api", |mut caller: Caller<'_, VMState>, api_name_ptr: i32, api_name_len: i32, req_ptr: i32, req_len: i32, resp_ptr: i32, resp_max_len: i32| -> i32 {
        let mem = match caller.get_export("memory") {
            Some(wasmtime::Extern::Memory(m)) => m,
            _ => return -1,
        };

        let mem_data = mem.data(&caller);

        if api_name_ptr < 0 || api_name_len < 0 || (api_name_ptr as usize + api_name_len as usize) > mem_data.len() {
            return -1;
        }
        let api_name = String::from_utf8_lossy(&mem_data[api_name_ptr as usize..(api_name_ptr + api_name_len) as usize]).into_owned();

        if req_ptr < 0 || req_len < 0 || (req_ptr as usize + req_len as usize) > mem_data.len() {
            return -1;
        }
        let request = mem_data[req_ptr as usize..(req_ptr + req_len) as usize].to_vec();

        // Use scoped block to drop state borrow before mem.write
        let (call_idx, replayed_payload) = {
            let state = caller.data_mut();
            state.call_index += 1;
            let call_idx = state.call_index;

            // Oplog replay
            if (call_idx - 1) < state.oplog.len() as i32 {
                let entry = &state.oplog[(call_idx - 1) as usize];
                if entry.api_name == api_name {
                    (call_idx, Some(entry.response_payload.clone()))
                } else {
                    (call_idx, None)
                }
            } else {
                (call_idx, None)
            }
        };

        if let Some(payload) = replayed_payload {
            if payload.len() > resp_max_len as usize {
                return -2;
            }
            let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
            mem.write(&mut caller, resp_ptr as usize, &payload).unwrap();
            return payload.len() as i32;
        }

        // Live host call
        let response = match api_name.as_str() {
            "test_api" => {
                format!("resp_for_{}_call_{}", String::from_utf8_lossy(&request), call_idx).into_bytes()
            }
            _ => {
                return -3;
            }
        };

        if response.len() > resp_max_len as usize {
            return -2;
        }

        let mem = caller.get_export("memory").unwrap().into_memory().unwrap();
        mem.write(&mut caller, resp_ptr as usize, &response).unwrap();

        let entry = OplogEntry {
            call_index: call_idx,
            api_name,
            request_payload: request,
            response_payload: response,
        };

        let state = caller.data_mut();
        state.oplog.push(entry.clone());

        state.oplog.last().unwrap().response_payload.len() as i32
    }).unwrap();

    // Calculate initial page hashes
    let restored_mem = restore_memory(base_snapshot, memory_deltas);
    let (_, initial_page_hashes) = calculate_deltas(&restored_mem, &HashMap::new());

    let (max_fuel, max_mem_mb) = if let Some(config) = sandbox_config {
        (
            if config.max_fuel == 0 { 10_000_000 } else { config.max_fuel },
            if config.max_memory_mb == 0 { 32 } else { config.max_memory_mb }
        )
    } else {
        (10_000_000, 32)
    };

    let limits = wasmtime::StoreLimitsBuilder::new()
        .memory_size(max_mem_mb as usize * 1024 * 1024)
        .build();

    let mut store_obj = Store::new(
        module.engine(),
        VMState {
            instance_id,
            call_index: initial_call_index,
            oplog: initial_oplog,
            page_hashes: initial_page_hashes,
            checkpoints: vec![],
            limits,
        },
    );
    store_obj.limiter(|state| &mut state.limits);
    let _ = store_obj.set_fuel(max_fuel);


    // Instantiate module
    let instance = match linker.instantiate(&mut store_obj, module) {
        Ok(i) => i,
        Err(e) => return RunResult {
            final_oplog: store_obj.data().oplog.clone(),
            final_deltas: HashMap::new(),
            checkpoints: vec![],
            crashed: true,
            error: format!("Failed to instantiate module: {}", e),
            response_bytes: vec![],
        },
    };

    // If fresh run, invoke WASI start to initialize Go/TinyGo runtime (allocators, GC, etc.)
    if base_snapshot.is_empty() {
        if let Some(start_func) = instance.get_func(&mut store_obj, "_start") {
            if let Err(e) = start_func.call(&mut store_obj, &[], &mut []) {
                return RunResult {
                    final_oplog: store_obj.data().oplog.clone(),
                    final_deltas: HashMap::new(),
                    checkpoints: vec![],
                    crashed: true,
                    error: format!("Failed to initialize runtime via _start: {}", e),
                    response_bytes: vec![],
                };
            }
        }
    }

    // Restore memory snapshot in VM
    if !restored_mem.is_empty() {
        if let Some(wasmtime::Extern::Memory(m)) = instance.get_export(&mut store_obj, "memory") {
            let current_pages = m.size(&store_obj);
            let needed_pages = (restored_mem.len() + PAGE_SIZE - 1) / PAGE_SIZE;
            if needed_pages > current_pages as usize {
                let grow_pages = (needed_pages - current_pages as usize) as u64;
                m.grow(&mut store_obj, grow_pages).unwrap();
            }
            m.write(&mut store_obj, 0, &restored_mem).unwrap();
        }
    }

    // Write inputs to exchange buffer
    let mut buf_ptr = 0usize;
    if !exchange_buffer.is_empty() {
        if let Some(get_buf_ptr_func) = instance.get_func(&mut store_obj, "get_exchange_buffer_pointer") {
            let mut results = vec![wasmtime::Val::I32(0)];
            if let Ok(_) = get_buf_ptr_func.call(&mut store_obj, &[], &mut results) {
                if let Some(wasmtime::Val::I32(ptr)) = results.get(0) {
                    buf_ptr = *ptr as usize;
                    if let Some(wasmtime::Extern::Memory(m)) = instance.get_export(&mut store_obj, "memory") {
                        m.write(&mut store_obj, buf_ptr, exchange_buffer).unwrap();
                    }
                }
            }
        }
    }

    // Call entrypoint function
    let func = match instance.get_func(&mut store_obj, entrypoint) {
        Some(f) => f,
        None => return RunResult {
            final_oplog: store_obj.data().oplog.clone(),
            final_deltas: HashMap::new(),
            checkpoints: vec![],
            crashed: true,
            error: format!("Entrypoint '{}' not found", entrypoint),
            response_bytes: vec![],
        },
    };

    let func_type = func.ty(&store_obj);
    let expected_types: Vec<wasmtime::ValType> = func_type.params().collect();
    let mut wasmtime_params = Vec::with_capacity(expected_types.len());
    for (i, val_type) in expected_types.iter().enumerate() {
        let p = params.get(i).copied().unwrap_or(0);
        let val = match val_type {
            wasmtime::ValType::I32 => wasmtime::Val::I32(p as i32),
            wasmtime::ValType::I64 => wasmtime::Val::I64(p as i64),
            _ => wasmtime::Val::I64(p as i64),
        };
        wasmtime_params.push(val);
    }
    let mut wasmtime_results = vec![wasmtime::Val::I32(0)];

    match func.call(&mut store_obj, &wasmtime_params, &mut wasmtime_results) {
        Ok(_) => {
            // Get final memory and calculate final deltas
            let final_mem = if let Some(wasmtime::Extern::Memory(m)) = instance.get_export(&mut store_obj, "memory") {
                m.data(&store_obj).to_vec()
            } else {
                vec![]
            };

            let base_hashes = if base_snapshot.is_empty() {
                HashMap::new()
            } else {
                let (_, h) = calculate_deltas(base_snapshot, &HashMap::new());
                h
            };

            let (final_deltas, _) = calculate_deltas(&final_mem, &base_hashes);

            let mut response_bytes = vec![];
            let mut crashed = false;
            let mut error = "".to_string();

            if let Some(wasmtime::Val::I32(result_len)) = wasmtime_results.get(0) {
                let len = *result_len;
                if len < 0 {
                    crashed = true;
                    error = format!("WASM core execution failed with error code: {}", len);
                } else if len > 0 {
                    if let Some(wasmtime::Extern::Memory(m)) = instance.get_export(&mut store_obj, "memory") {
                        let mut buf = vec![0u8; len as usize];
                        m.read(&store_obj, buf_ptr, &mut buf).unwrap();
                        response_bytes = buf;
                    }
                }
            }

            RunResult {
                final_oplog: store_obj.data().oplog.clone(),
                final_deltas,
                checkpoints: store_obj.data().checkpoints.clone(),
                crashed,
                error,
                response_bytes,
            }
        }
        Err(e) => RunResult {
            final_oplog: store_obj.data().oplog.clone(),
            final_deltas: HashMap::new(),
            checkpoints: store_obj.data().checkpoints.clone(),
            crashed: true,
            error: format!("Execution failed: {}", e),
            response_bytes: vec![],
        },
    }
}
