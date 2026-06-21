use serde::{Serialize, Deserialize};
use std::collections::HashMap;

const BUFFER_SIZE: usize = 1024 * 1024;
static mut EXCHANGE_BUFFER: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn checkpoint();
    pub fn host_get_time() -> i64;
}

#[no_mangle]
pub extern "C" fn get_exchange_buffer_pointer() -> *mut u8 {
    std::ptr::addr_of_mut!(EXCHANGE_BUFFER) as *mut u8
}

#[derive(Serialize, Deserialize, Clone)]
struct ProcessInstance {
    id: String,
    process_id: String, // "todo", "kanban", "incident"
    active_activity_instances: Vec<String>,
    waiting_activity_instances: Vec<String>,
    completed_nodes: Vec<String>,
    variables: HashMap<String, serde_json::Value>,
    completed: bool,
}

#[no_mangle]
pub extern "C" fn execute(type_len: u32, variables_len: u32) -> i32 {
    let type_bytes = unsafe { &EXCHANGE_BUFFER[..type_len as usize] };
    let workflow_type = std::str::from_utf8(type_bytes).unwrap_or("todo");

    let vars_bytes = unsafe { &EXCHANGE_BUFFER[type_len as usize..(type_len + variables_len) as usize] };
    let variables: HashMap<String, serde_json::Value> = serde_json::from_slice(vars_bytes).unwrap_or_default();

    let mut instance = ProcessInstance {
        id: "servicedesk-session".to_string(),
        process_id: workflow_type.to_string(),
        active_activity_instances: vec![],
        waiting_activity_instances: vec![],
        completed_nodes: vec!["start".to_string()],
        variables,
        completed: false,
    };

    let now = unsafe { host_get_time() };

    match workflow_type {
        "todo" => {
            instance.waiting_activity_instances.push("add_todo".to_string());
        }
        "kanban" => {
            instance.waiting_activity_instances.push("create_task".to_string());
        }
        "incident" => {
            instance.variables.insert("status".to_string(), serde_json::json!("New"));
            instance.variables.insert("created_at".to_string(), serde_json::json!(now));
            instance.waiting_activity_instances.push("create_incident".to_string());
        }
        _ => return -1,
    }

    // Save initial state checkpoint
    unsafe {
        checkpoint();
    }

    write_instance_to_buffer(&instance)
}

#[no_mangle]
pub extern "C" fn resume(instance_len: u32, active_task_len: u32, input_len: u32, role_len: u32) -> i32 {
    let mut offset = 0usize;

    let instance_bytes = unsafe { &EXCHANGE_BUFFER[offset..offset + instance_len as usize] };
    let mut instance: ProcessInstance = match serde_json::from_slice(instance_bytes) {
        Ok(inst) => inst,
        Err(_) => return -1,
    };
    offset += instance_len as usize;

    let active_task_bytes = unsafe { &EXCHANGE_BUFFER[offset..offset + active_task_len as usize] };
    let active_task = std::str::from_utf8(active_task_bytes).unwrap_or("");
    offset += active_task_len as usize;

    let input_bytes = unsafe { &EXCHANGE_BUFFER[offset..offset + input_len as usize] };
    let input: HashMap<String, serde_json::Value> = serde_json::from_slice(input_bytes).unwrap_or_default();
    offset += input_len as usize;

    let role_bytes = unsafe { &EXCHANGE_BUFFER[offset..offset + role_len as usize] };
    let role = std::str::from_utf8(role_bytes).unwrap_or("Customer");

    // Role-based Authorization check
    if instance.process_id == "incident" {
        // Customers can only create incidents
        if active_task != "create_incident" && role == "Customer" {
            return -403; // Forbidden
        }
        // Support and Managers can perform support workflows
        if (active_task == "new_incident" || active_task == "investigating" || active_task == "resolved")
            && (role != "Support" && role != "Manager") {
            return -403; // Forbidden
        }
    } else if instance.process_id == "kanban" {
        // Restrict moving task to progress/review to Support or Manager
        if (active_task == "backlog" || active_task == "in_progress" || active_task == "in_review")
            && (role != "Support" && role != "Manager") {
            return -403; // Forbidden
        }
    }

    // Merge input variables
    for (k, v) in input {
        instance.variables.insert(k, v);
    }

    // Remove active task from waiting list
    if let Some(pos) = instance.waiting_activity_instances.iter().position(|t| t == active_task) {
        instance.waiting_activity_instances.remove(pos);
    }
    instance.completed_nodes.push(active_task.to_string());

    let now = unsafe { host_get_time() };

    // Process State Transitions
    match instance.process_id.as_str() {
        "todo" => {
            match active_task {
                "add_todo" => {
                    instance.waiting_activity_instances.push("complete_todo".to_string());
                }
                "complete_todo" => {
                    instance.completed = true;
                    instance.completed_nodes.push("end".to_string());
                }
                _ => return -2,
            }
        }
        "kanban" => {
            match active_task {
                "create_task" => {
                    instance.variables.insert("column".to_string(), serde_json::json!("backlog"));
                    instance.waiting_activity_instances.push("backlog".to_string());
                }
                "backlog" => {
                    instance.variables.insert("column".to_string(), serde_json::json!("in_progress"));
                    instance.waiting_activity_instances.push("in_progress".to_string());
                }
                "in_progress" => {
                    instance.variables.insert("column".to_string(), serde_json::json!("in_review"));
                    instance.waiting_activity_instances.push("in_review".to_string());
                }
                "in_review" => {
                    instance.variables.insert("column".to_string(), serde_json::json!("done"));
                    instance.completed = true;
                    instance.completed_nodes.push("end".to_string());
                }
                _ => return -2,
            }
        }
        "incident" => {
            let mut history = get_history(&instance);
            match active_task {
                "create_incident" => {
                    let priority = instance.variables.get("priority").and_then(|v| v.as_str()).unwrap_or("Medium").to_string();
                    
                    // SLA limits in nanoseconds
                    let (reaction_dur, resolution_dur) = match priority.as_str() {
                        "Critical" => (15 * 60 * 1_000_000_000i64, 60 * 60 * 1_000_000_000i64),
                        "High" => (60 * 60 * 1_000_000_000i64, 4 * 60 * 60 * 1_000_000_000i64),
                        "Medium" => (4 * 60 * 60 * 1_000_000_000i64, 24 * 60 * 60 * 1_000_000_000i64),
                        _ => (24 * 60 * 60 * 1_000_000_000i64, 72 * 60 * 60 * 1_000_000_000i64),
                    };

                    instance.variables.insert("status".to_string(), serde_json::json!("New"));
                    instance.variables.insert("sla_reaction_limit".to_string(), serde_json::json!(now + reaction_dur));
                    instance.variables.insert("sla_resolution_limit".to_string(), serde_json::json!(now + resolution_dur));
                    instance.variables.insert("sla_reaction_breached".to_string(), serde_json::json!(false));
                    instance.variables.insert("sla_resolution_breached".to_string(), serde_json::json!(false));
                    
                    history.push(format!("Incident created with {} priority. SLA limits initiated.", priority));
                    set_history(&mut instance, history);

                    instance.waiting_activity_instances.push("new_incident".to_string());
                }
                "new_incident" => {
                    instance.variables.insert("status".to_string(), serde_json::json!("Assigned"));
                    instance.variables.insert("assigned_at".to_string(), serde_json::json!(now));
                    
                    // Check SLA breach on reaction
                    let limit = instance.variables.get("sla_reaction_limit").and_then(|v| v.as_i64()).unwrap_or(0);
                    if now > limit {
                        instance.variables.insert("sla_reaction_breached".to_string(), serde_json::json!(true));
                        history.push("Reaction SLA Breached: assignment took too long.".to_string());
                    } else {
                        history.push("Reaction SLA satisfied.".to_string());
                    }
                    
                    history.push("Incident assigned to support engineer.".to_string());
                    set_history(&mut instance, history);

                    instance.waiting_activity_instances.push("investigating".to_string());
                }
                "investigating" => {
                    instance.variables.insert("status".to_string(), serde_json::json!("Investigating"));
                    history.push("Investigation details updated.".to_string());
                    set_history(&mut instance, history);

                    instance.waiting_activity_instances.push("resolved".to_string());
                }
                "resolved" => {
                    instance.variables.insert("status".to_string(), serde_json::json!("Resolved"));
                    instance.variables.insert("resolved_at".to_string(), serde_json::json!(now));

                    // Check SLA breach on resolution
                    let limit = instance.variables.get("sla_resolution_limit").and_then(|v| v.as_i64()).unwrap_or(0);
                    if now > limit {
                        instance.variables.insert("sla_resolution_breached".to_string(), serde_json::json!(true));
                        history.push("Resolution SLA Breached: resolution took too long.".to_string());
                    } else {
                        history.push("Resolution SLA satisfied.".to_string());
                    }

                    history.push("Incident marked as resolved.".to_string());
                    set_history(&mut instance, history);

                    instance.completed = true;
                    instance.completed_nodes.push("end".to_string());
                }
                _ => return -2,
            }
        }
        _ => return -2,
    }

    // Save state checkpoint
    unsafe {
        checkpoint();
    }

    write_instance_to_buffer(&instance)
}

#[no_mangle]
pub extern "C" fn tick(instance_len: u32) -> i32 {
    let instance_bytes = unsafe { &EXCHANGE_BUFFER[..instance_len as usize] };
    let mut instance: ProcessInstance = match serde_json::from_slice(instance_bytes) {
        Ok(inst) => inst,
        Err(_) => return -1,
    };

    if instance.process_id != "incident" || instance.completed {
        return write_instance_to_buffer(&instance);
    }

    let now = unsafe { host_get_time() };
    let mut modified = false;
    let mut history = get_history(&instance);

    // 1. Check reaction SLA breach (only if status is "New")
    let status = instance.variables.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if status == "New" {
        let reaction_breached = instance.variables.get("sla_reaction_breached").and_then(|v| v.as_bool()).unwrap_or(false);
        if !reaction_breached {
            let limit = instance.variables.get("sla_reaction_limit").and_then(|v| v.as_i64()).unwrap_or(0);
            if now > limit {
                instance.variables.insert("sla_reaction_breached".to_string(), serde_json::json!(true));
                history.push("SLA Warning: Reaction SLA Breached! Auto-escalating priority.".to_string());
                
                // Auto-escalation: medium -> high, high -> critical
                let current_priority = instance.variables.get("priority").and_then(|v| v.as_str()).unwrap_or("Medium").to_string();
                let new_priority = match current_priority.as_str() {
                    "Low" => "Medium",
                    "Medium" => "High",
                    "High" => "Critical",
                    _ => "Critical",
                };
                
                if new_priority != current_priority {
                    instance.variables.insert("priority".to_string(), serde_json::json!(new_priority));
                    history.push(format!("Priority auto-escalated to {}.", new_priority));
                }
                
                modified = true;
            }
        }
    }

    // 2. Check resolution SLA breach (if not resolved yet)
    if status != "Resolved" {
        let resolution_breached = instance.variables.get("sla_resolution_breached").and_then(|v| v.as_bool()).unwrap_or(false);
        if !resolution_breached {
            let limit = instance.variables.get("sla_resolution_limit").and_then(|v| v.as_i64()).unwrap_or(0);
            if now > limit {
                instance.variables.insert("sla_resolution_breached".to_string(), serde_json::json!(true));
                history.push("SLA Danger: Resolution SLA Breached! Manager Alert triggered.".to_string());
                modified = true;
            }
        }
    }

    if modified {
        set_history(&mut instance, history);
        unsafe {
            checkpoint();
        }
    }

    write_instance_to_buffer(&instance)
}

// Helpers for buffer read/write and history logging
fn write_instance_to_buffer(instance: &ProcessInstance) -> i32 {
    let serialized = serde_json::to_vec(instance).unwrap();
    if serialized.len() > BUFFER_SIZE {
        return -3;
    }
    unsafe {
        EXCHANGE_BUFFER[..serialized.len()].copy_from_slice(&serialized);
    }
    serialized.len() as i32
}

fn get_history(instance: &ProcessInstance) -> Vec<String> {
    instance.variables.get("history")
        .and_then(|v| serde_json::from_value::<Vec<String>>(v.clone()).ok())
        .unwrap_or_default()
}

fn set_history(instance: &mut ProcessInstance, history: Vec<String>) {
    instance.variables.insert("history".to_string(), serde_json::json!(history));
}
