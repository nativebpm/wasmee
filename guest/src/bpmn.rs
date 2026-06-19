use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[link(wasm_import_module = "env")]
extern "C" {
    pub fn checkpoint();
    pub fn host_call_api(
        api_name_ptr: *const u8,
        api_name_len: usize,
        req_ptr: *const u8,
        req_len: usize,
        resp_ptr: *mut u8,
        resp_max_len: usize,
    ) -> i32;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphDefinition {
    pub id: String,
    pub name: String,
    pub nodes: HashMap<String, GraphNode>,
    pub connections: Vec<Connection>,
    pub start_node_id: String,
    pub decisions: Option<HashMap<String, crate::dmn::DmnTable>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GraphNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub name: String,
    #[serde(rename = "decisionRef")]
    pub decision_ref: Option<String>,
    #[serde(rename = "mapDecisionResult")]
    pub map_decision_result: Option<String>,
    #[serde(rename = "resultVariable")]
    pub result_variable: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Connection {
    pub id: String,
    pub source_ref: String,
    pub target_ref: String,
    pub condition: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProcessInstance {
    pub id: String,
    pub process_id: String,
    pub active_activity_instances: Vec<String>,
    pub waiting_activity_instances: Vec<String>,
    pub completed_nodes: Vec<String>,
    pub variables: HashMap<String, serde_json::Value>,
    pub completed: bool,
}

fn count_inflows(graph: &GraphDefinition, node_id: &str) -> usize {
    graph.connections.iter().filter(|c| c.target_ref == node_id).count()
}

fn has_path(graph: &GraphDefinition, source: &str, target: &str) -> bool {
    let mut visited = std::collections::HashSet::new();
    fn dfs(
        graph: &GraphDefinition,
        curr: &str,
        target: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> bool {
        if curr == target {
            return true;
        }
        if visited.contains(curr) {
            return false;
        }
        visited.insert(curr.to_string());
        for conn in &graph.connections {
            if conn.source_ref == curr {
                if dfs(graph, &conn.target_ref, target, visited) {
                    return true;
                }
            }
        }
        false
    }

    for conn in &graph.connections {
        if conn.source_ref == source {
            if dfs(graph, &conn.target_ref, target, &mut visited) {
                return true;
            }
        }
    }
    false
}

fn call_host_evaluate_flow_condition(flow_id: &str) -> bool {
    let api_name = "evaluate_flow_condition";
    let mut resp_buf = vec![0u8; 16];
    let res = unsafe {
        host_call_api(
            api_name.as_ptr(),
            api_name.len(),
            flow_id.as_ptr(),
            flow_id.len(),
            resp_buf.as_mut_ptr(),
            resp_buf.len(),
        )
    };
    res >= 0 && std::str::from_utf8(&resp_buf[..res as usize]).unwrap_or("") == "true"
}

fn transition_exclusive(graph: &GraphDefinition, instance: &mut ProcessInstance, source_ref: &str) {
    for conn in &graph.connections {
        if conn.source_ref == source_ref {
            let mut is_true = false;
            if conn.condition.is_none() || conn.condition.as_ref().unwrap().is_empty() {
                is_true = true;
            } else if conn.condition.as_ref().unwrap().starts_with("wasm:") {
                is_true = call_host_evaluate_flow_condition(&conn.id);
            } else if let Some(decisions) = &graph.decisions {
                if let Some(dec_table) = decisions.get(&conn.id) {
                    match crate::dmn::evaluate(dec_table, &instance.variables) {
                        Ok(Some(res)) => {
                            if let Some(serde_json::Value::Bool(b)) = res.get("result") {
                                is_true = *b;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if is_true {
                instance.active_activity_instances.push(conn.target_ref.clone());
                return;
            }
        }
    }
    // Fallback to first flow
    for conn in &graph.connections {
        if conn.source_ref == source_ref {
            instance.active_activity_instances.push(conn.target_ref.clone());
            return;
        }
    }
}

fn transition_inclusive(graph: &GraphDefinition, instance: &mut ProcessInstance, source_ref: &str) {
    let mut activated_any = false;
    for conn in &graph.connections {
        if conn.source_ref == source_ref {
            let mut is_true = false;
            if conn.condition.is_none() || conn.condition.as_ref().unwrap().is_empty() {
                is_true = true;
            } else if conn.condition.as_ref().unwrap().starts_with("wasm:") {
                is_true = call_host_evaluate_flow_condition(&conn.id);
            } else if let Some(decisions) = &graph.decisions {
                if let Some(dec_table) = decisions.get(&conn.id) {
                    match crate::dmn::evaluate(dec_table, &instance.variables) {
                        Ok(Some(res)) => {
                            if let Some(serde_json::Value::Bool(b)) = res.get("result") {
                                is_true = *b;
                            }
                        }
                        _ => {}
                    }
                }
            }

            if is_true {
                instance.active_activity_instances.push(conn.target_ref.clone());
                activated_any = true;
            }
        }
    }
    if !activated_any {
        for conn in &graph.connections {
            if conn.source_ref == source_ref {
                instance.active_activity_instances.push(conn.target_ref.clone());
                return;
            }
        }
    }
}

pub fn transition_outbound(graph: &GraphDefinition, instance: &mut ProcessInstance, source_ref: &str) {
    for conn in &graph.connections {
        if conn.source_ref == source_ref {
            instance.active_activity_instances.push(conn.target_ref.clone());
        }
    }
}

pub fn run(graph: &GraphDefinition, instance: &mut ProcessInstance) {
    while !instance.active_activity_instances.is_empty() {
        let current_node_id = instance.active_activity_instances.remove(0);

        let node = match graph.nodes.get(&current_node_id) {
            Some(n) => n,
            None => {
                println!("[WASM Guest] Error: node not found: {}", current_node_id);
                return;
            }
        };

        instance.completed_nodes.push(current_node_id.clone());

        match node.node_type.as_str() {
            "StartEvent" => {
                transition_outbound(graph, instance, &current_node_id);
            }
            "ExclusiveGateway" => {
                transition_exclusive(graph, instance, &current_node_id);
            }
            "InclusiveGateway" => {
                let inflows_count = count_inflows(graph, &current_node_id);
                if inflows_count > 1 {
                    let mut can_more_tokens_arrive = false;
                    for active in &instance.active_activity_instances {
                        if active != &current_node_id && has_path(graph, active, &current_node_id) {
                            can_more_tokens_arrive = true;
                            break;
                        }
                    }
                    if !can_more_tokens_arrive {
                        for waiting in &instance.waiting_activity_instances {
                            if waiting != &current_node_id && has_path(graph, waiting, &current_node_id) {
                                can_more_tokens_arrive = true;
                                break;
                            }
                        }
                    }
                    if can_more_tokens_arrive {
                        instance.waiting_activity_instances.push(current_node_id.clone());
                        continue;
                    }
                    instance.waiting_activity_instances.retain(|x| x != &current_node_id);
                    instance.active_activity_instances.retain(|x| x != &current_node_id);
                }
                transition_inclusive(graph, instance, &current_node_id);
            }
            "ParallelGateway" => {
                let inflows_count = count_inflows(graph, &current_node_id);
                if inflows_count > 1 {
                    let mut tokens_on_gateway = 1;
                    for active in &instance.active_activity_instances {
                        if active == &current_node_id {
                            tokens_on_gateway += 1;
                        }
                    }
                    for waiting in &instance.waiting_activity_instances {
                        if waiting == &current_node_id {
                            tokens_on_gateway += 1;
                        }
                    }
                    if tokens_on_gateway < inflows_count {
                        instance.waiting_activity_instances.push(current_node_id.clone());
                        continue;
                    }
                    instance.waiting_activity_instances.retain(|x| x != &current_node_id);
                    instance.active_activity_instances.retain(|x| x != &current_node_id);
                }
                transition_outbound(graph, instance, &current_node_id);
            }
            "EventBasedGateway" => {
                transition_outbound(graph, instance, &current_node_id);
            }
            "ServiceTask" => {
                let payload_map = serde_json::json!({
                    "instance_id": instance.id,
                    "task_id": current_node_id,
                });
                let payload = serde_json::to_vec(&payload_map).unwrap();
                let api_name = "execute_service_task";

                unsafe {
                    checkpoint();
                }

                let mut resp_buf = vec![0u8; 1024];
                let res = unsafe {
                    host_call_api(
                        api_name.as_ptr(),
                        api_name.len(),
                        payload.as_ptr(),
                        payload.len(),
                        resp_buf.as_mut_ptr(),
                        resp_buf.len(),
                    )
                };

                if res >= 0 {
                    if let Ok(updated_vars) = serde_json::from_slice::<std::collections::HashMap<String, serde_json::Value>>(&resp_buf[..res as usize]) {
                        for (k, v) in updated_vars {
                            instance.variables.insert(k, v);
                        }
                    }
                    transition_outbound(graph, instance, &current_node_id);
                } else {
                    instance.waiting_activity_instances.push(current_node_id.clone());
                }
            }
            "BusinessRuleTask" => {
                if let Some(decisions) = &graph.decisions {
                    if let Some(decision_ref) = &node.decision_ref {
                        if let Some(dec_table) = decisions.get(decision_ref) {
                            match crate::dmn::evaluate(dec_table, &instance.variables) {
                                Ok(Some(res)) => {
                                    if let Some(result_var) = &node.result_variable {
                                        if node.map_decision_result.as_deref() == Some("singleEntry") {
                                            for (_, v) in res {
                                                instance.variables.insert(result_var.clone(), v);
                                                break;
                                            }
                                        } else {
                                            instance.variables.insert(
                                                result_var.clone(),
                                                serde_json::to_value(&res).unwrap(),
                                            );
                                        }
                                    } else {
                                        for (k, v) in res {
                                            instance.variables.insert(k, v);
                                        }
                                    }
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    println!("[WASM Guest] Error: DMN evaluation failed: {}", e);
                                }
                            }
                        } else {
                            println!("[WASM Guest] Warning: DMN decision table not found: {}", decision_ref);
                        }
                    }
                }
                transition_outbound(graph, instance, &current_node_id);
            }
            "UserTask" | "ReceiveTask" | "IntermediateCatchEvent" | "CallActivity" => {
                instance.waiting_activity_instances.push(current_node_id.clone());
            }
            "EndEvent" => {
                if instance.active_activity_instances.is_empty()
                    && instance.waiting_activity_instances.is_empty()
                {
                    instance.completed = true;
                }
            }
            _ => {}
        }
    }

    unsafe {
        checkpoint();
    }
}
