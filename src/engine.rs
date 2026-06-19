use std::collections::HashMap;
use std::sync::mpsc::{Receiver as SyncReceiver, Sender as SyncSender};
use tokio::sync::mpsc::UnboundedSender;
use wasmtime::{Caller, Engine, Linker, Module, Store};

pub const PAGE_SIZE: usize = 65536;

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

#[derive(Clone, Debug)]
pub struct OplogEntry {
    pub call_index: i32,
    pub api_name: String,
    pub request_payload: Vec<u8>,
    pub response_payload: Vec<u8>,
}

pub enum EngineToHostMsg {
    HostCall {
        api_name: String,
        request: Vec<u8>,
        call_index: i32,
        resp_tx: SyncSender<Result<Vec<u8>, String>>,
    },
    Checkpoint {
        memory: Vec<u8>,
        resp_tx: SyncSender<Result<(), String>>,
    },
}

pub struct VMState {
    pub instance_id: String,
    pub call_index: i32,
    pub oplog: Vec<OplogEntry>,
    pub page_hashes: HashMap<i32, u64>,
    pub msg_tx: UnboundedSender<EngineToHostMsg>,
}

pub struct RunResult {
    pub final_oplog: Vec<OplogEntry>,
    pub final_deltas: HashMap<i32, Vec<u8>>,
    pub crashed: bool,
    pub error: String,
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
    msg_tx: UnboundedSender<EngineToHostMsg>,
) -> RunResult {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);

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

        // Send checkpoint request to host gRPC loop
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();
        if let Err(e) = caller.data().msg_tx.send(EngineToHostMsg::Checkpoint {
            memory: data.clone(),
            resp_tx,
        }) {
            panic!("failed to send checkpoint request: {}", e);
        }

        match resp_rx.recv() {
            Ok(Ok(())) => {
                // Successful checkpoint. Update local hashes!
                let state = caller.data_mut();
                let (_, new_hashes) = calculate_deltas(&data, &state.page_hashes);
                state.page_hashes = new_hashes;
            }
            Ok(Err(e)) => panic!("checkpoint failed: {}", e),
            Err(e) => panic!("checkpoint response channel error: {}", e),
        }
    }).unwrap();

    linker.func_wrap("env", "host_get_time", |mut caller: Caller<'_, VMState>| -> i64 {
        let state = caller.data_mut();
        state.call_index += 1;
        let call_idx = state.call_index;

        // 1. Check oplog replay
        if (call_idx - 1) < state.oplog.len() as i32 {
            let entry = &state.oplog[(call_idx - 1) as usize];
            if entry.api_name != "host_get_time" {
                panic!("oplog drift: expected host_get_time, found {}", entry.api_name);
            }
            let time_str = String::from_utf8_lossy(&entry.response_payload);
            return time_str.parse::<i64>().unwrap_or(0);
        }

        // 2. Perform fresh host call
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();
        if let Err(e) = state.msg_tx.send(EngineToHostMsg::HostCall {
            api_name: "host_get_time".to_string(),
            request: vec![],
            call_index: call_idx,
            resp_tx,
        }) {
            panic!("failed to send host call request: {}", e);
        }

        match resp_rx.recv() {
            Ok(Ok(resp)) => {
                let time_str = String::from_utf8_lossy(&resp);
                let time_val = time_str.parse::<i64>().unwrap_or(0);
                // Append to oplog
                let state = caller.data_mut();
                state.oplog.push(OplogEntry {
                    call_index: call_idx,
                    api_name: "host_get_time".to_string(),
                    request_payload: vec![],
                    response_payload: resp,
                });
                time_val
            }
            Ok(Err(e)) => panic!("host_get_time failed: {}", e),
            Err(e) => panic!("host_get_time channel error: {}", e),
        }
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

        let mut replayed_payload = None;
        let call_idx;
        {
            let state = caller.data_mut();
            state.call_index += 1;
            call_idx = state.call_index;

            // 1. Check oplog replay
            if (call_idx - 1) < state.oplog.len() as i32 {
                let entry = &state.oplog[(call_idx - 1) as usize];
                if entry.api_name != api_name {
                    panic!("oplog drift: expected {}, found {}", api_name, entry.api_name);
                }
                if entry.response_payload.len() > resp_max_len as usize {
                    return -2;
                }
                replayed_payload = Some(entry.response_payload.clone());
            }
        }

        if let Some(payload) = replayed_payload {
            // Write response to memory
            let mem = match caller.get_export("memory") {
                Some(wasmtime::Extern::Memory(m)) => m,
                _ => return -1,
            };
            mem.write(&mut caller, resp_ptr as usize, &payload).unwrap();
            return payload.len() as i32;
        }

        // 2. Perform fresh host call
        let (resp_tx, resp_rx) = std::sync::mpsc::channel();
        {
            let state = caller.data_mut();
            if let Err(e) = state.msg_tx.send(EngineToHostMsg::HostCall {
                api_name: api_name.clone(),
                request: request.clone(),
                call_index: call_idx,
                resp_tx,
            }) {
                panic!("failed to send host call request: {}", e);
            }
        }

        match resp_rx.recv() {
            Ok(Ok(resp)) => {
                if resp.len() > resp_max_len as usize {
                    return -2;
                }
                let mem = match caller.get_export("memory") {
                    Some(wasmtime::Extern::Memory(m)) => m,
                    _ => return -1,
                };
                mem.write(&mut caller, resp_ptr as usize, &resp).unwrap();

                // Append to oplog
                let state = caller.data_mut();
                state.oplog.push(OplogEntry {
                    call_index: call_idx,
                    api_name,
                    request_payload: request,
                    response_payload: resp.clone(),
                });
                resp.len() as i32
            }
            Ok(Err(e)) => panic!("host_call_api failed: {}", e),
            Err(e) => panic!("host_call_api channel error: {}", e),
        }
    }).unwrap();

    // Compile module
    let module = match Module::new(&engine, wasm_bytes) {
        Ok(m) => m,
        Err(e) => return RunResult {
            final_oplog: initial_oplog,
            final_deltas: HashMap::new(),
            crashed: true,
            error: format!("Failed to compile module: {}", e),
        },
    };

    // Calculate initial page hashes
    let restored_mem = restore_memory(base_snapshot, memory_deltas);
    let (_, initial_page_hashes) = calculate_deltas(&restored_mem, &HashMap::new());

    let mut store = Store::new(
        &engine,
        VMState {
            instance_id,
            call_index: initial_call_index,
            oplog: initial_oplog,
            page_hashes: initial_page_hashes,
            msg_tx,
        },
    );

    // Instantiate module
    let instance = match linker.instantiate(&mut store, &module) {
        Ok(i) => i,
        Err(e) => return RunResult {
            final_oplog: store.data().oplog.clone(),
            final_deltas: HashMap::new(),
            crashed: true,
            error: format!("Failed to instantiate module: {}", e),
        },
    };

    // Restore memory snapshot in VM
    if !restored_mem.is_empty() {
        if let Some(wasmtime::Extern::Memory(m)) = instance.get_export(&mut store, "memory") {
            let current_pages = m.size(&store);
            let needed_pages = (restored_mem.len() + PAGE_SIZE - 1) / PAGE_SIZE;
            if needed_pages > current_pages as usize {
                let grow_pages = (needed_pages - current_pages as usize) as u64;
                m.grow(&mut store, grow_pages).unwrap();
            }
            m.write(&mut store, 0, &restored_mem).unwrap();
        }
    }

    // Call entrypoint function
    let func = match instance.get_func(&mut store, entrypoint) {
        Some(f) => f,
        None => return RunResult {
            final_oplog: store.data().oplog.clone(),
            final_deltas: HashMap::new(),
            crashed: true,
            error: format!("Entrypoint '{}' not found", entrypoint),
        },
    };

    let wasmtime_params: Vec<wasmtime::Val> = params
        .iter()
        .map(|&p| wasmtime::Val::I64(p as i64))
        .collect();
    let mut wasmtime_results = vec![wasmtime::Val::I32(0)];

    match func.call(&mut store, &wasmtime_params, &mut wasmtime_results) {
        Ok(_) => {
            // Get final memory and calculate final deltas
            let final_mem = if let Some(wasmtime::Extern::Memory(m)) = instance.get_export(&mut store, "memory") {
                m.data(&store).to_vec()
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

            RunResult {
                final_oplog: store.data().oplog.clone(),
                final_deltas,
                crashed: false,
                error: "".to_string(),
            }
        }
        Err(e) => RunResult {
            final_oplog: store.data().oplog.clone(),
            final_deltas: HashMap::new(),
            crashed: true,
            error: format!("Execution failed: {}", e),
        },
    }
}
