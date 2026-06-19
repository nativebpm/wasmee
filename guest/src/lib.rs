pub mod dmn;
pub mod bpmn;

use bpmn::{GraphDefinition, ProcessInstance};

// Declare 1MB shared static exchange buffer.
const MAX_BUFFER_SIZE: usize = 1024 * 1024;
static mut EXCHANGE_BUFFER: [u8; MAX_BUFFER_SIZE] = [0u8; MAX_BUFFER_SIZE];

#[no_mangle]
pub extern "C" fn get_exchange_buffer_pointer() -> *mut u8 {
    std::ptr::addr_of_mut!(EXCHANGE_BUFFER) as *mut u8
}

#[no_mangle]
pub extern "C" fn execute(graph_len: u32, variables_len: u32) -> i32 {
    let total_len = graph_len + variables_len;
    if total_len as usize > MAX_BUFFER_SIZE {
        return -1;
    }

    let graph_bytes = unsafe { &EXCHANGE_BUFFER[..graph_len as usize] };
    let variables_bytes = unsafe {
        &EXCHANGE_BUFFER[graph_len as usize..total_len as usize]
    };

    let graph: GraphDefinition = match serde_json::from_slice(graph_bytes) {
        Ok(g) => g,
        Err(e) => {
            println!("[WASM Guest] Failed to unmarshal graph definition: {}", e);
            return -2;
        }
    };

    let vars: std::collections::HashMap<String, serde_json::Value> =
        match serde_json::from_slice(variables_bytes) {
            Ok(v) => v,
            Err(e) => {
                println!("[WASM Guest] Failed to unmarshal variables: {}", e);
                return -3;
            }
        };

    let mut instance = ProcessInstance {
        id: format!("inst_{}", graph.id),
        process_id: graph.id.clone(),
        active_activity_instances: vec![graph.start_node_id.clone()],
        waiting_activity_instances: vec![],
        completed_nodes: vec![],
        variables: vars,
        completed: false,
    };

    bpmn::run(&graph, &mut instance);

    let res_bytes = match serde_json::to_vec(&instance) {
        Ok(b) => b,
        Err(_) => return -4,
    };

    if res_bytes.len() > MAX_BUFFER_SIZE {
        return -5;
    }

    unsafe {
        EXCHANGE_BUFFER[..res_bytes.len()].copy_from_slice(&res_bytes);
    }

    res_bytes.len() as i32
}

#[no_mangle]
pub extern "C" fn resume(
    graph_len: u32,
    instance_len: u32,
    completed_task_id_ptr: u32,
    completed_task_id_len: u32,
) -> i32 {
    let total_len = graph_len + instance_len;
    if total_len as usize > MAX_BUFFER_SIZE {
        return -1;
    }

    let graph_bytes = unsafe { &EXCHANGE_BUFFER[..graph_len as usize] };
    let instance_bytes = unsafe {
        &EXCHANGE_BUFFER[graph_len as usize..total_len as usize]
    };

    let graph: GraphDefinition = match serde_json::from_slice(graph_bytes) {
        Ok(g) => g,
        Err(_) => return -2,
    };

    let mut instance: ProcessInstance = match serde_json::from_slice(instance_bytes) {
        Ok(i) => i,
        Err(_) => return -3,
    };

    // Read completed task ID from EXCHANGE_BUFFER at relative offset
    let start_idx = completed_task_id_ptr as usize;
    let end_idx = start_idx + completed_task_id_len as usize;
    if end_idx > MAX_BUFFER_SIZE {
        return -6;
    }
    let task_id_bytes = unsafe { &EXCHANGE_BUFFER[start_idx..end_idx] };
    let completed_task_id = match std::str::from_utf8(task_id_bytes) {
        Ok(s) => s,
        Err(_) => return -7,
    };

    // Remove from waiting list
    let mut found = false;
    let mut new_waiting = Vec::new();
    for id in instance.waiting_activity_instances {
        if id == completed_task_id {
            found = true;
        } else {
            new_waiting.push(id);
        }
    }
    instance.waiting_activity_instances = new_waiting;

    if found {
        instance.completed_nodes.push(completed_task_id.to_string());
        bpmn::transition_outbound(&graph, &mut instance, completed_task_id);
        bpmn::run(&graph, &mut instance);
    }

    let res_bytes = match serde_json::to_vec(&instance) {
        Ok(b) => b,
        Err(_) => return -4,
    };

    if res_bytes.len() > MAX_BUFFER_SIZE {
        return -5;
    }

    unsafe {
        EXCHANGE_BUFFER[..res_bytes.len()].copy_from_slice(&res_bytes);
    }

    res_bytes.len() as i32
}

#[no_mangle]
pub extern "C" fn run_test() -> i32 {
    let api_name = "test_api";
    let req1 = b"hello";
    let mut resp_buf = vec![0u8; 1024];

    let _res1 = unsafe {
        bpmn::host_call_api(
            api_name.as_ptr(),
            api_name.len(),
            req1.as_ptr(),
            req1.len(),
            resp_buf.as_mut_ptr(),
            resp_buf.len(),
        )
    };

    unsafe {
        bpmn::checkpoint();
    }

    // Modify memory at offset 70000 to trigger dirty-page tracking
    unsafe {
        let ptr = 70000 as *mut i32;
        std::ptr::write_volatile(ptr, 42);
    }

    let req2 = b"world";
    let _res2 = unsafe {
        bpmn::host_call_api(
            api_name.as_ptr(),
            api_name.len(),
            req2.as_ptr(),
            req2.len(),
            resp_buf.as_mut_ptr(),
            resp_buf.len(),
        )
    };

    unsafe {
        bpmn::checkpoint();
    }

    0
}

