use std::collections::HashMap;
use std::pin::Pin;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::UnboundedReceiverStream, Stream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

pub mod engine;

pub mod wasmee {
    tonic::include_proto!("wasmee");
}

use wasmee::wasmee_executor_server::{WasmeeExecutor, WasmeeExecutorServer};
use wasmee::{
    ExecuteMessage, HostCallRequest, CheckpointRequest, CompleteResponse,
    OplogEntry as ProtoOplogEntry,
};

pub struct WasmeeService;

#[tonic::async_trait]
impl WasmeeExecutor for WasmeeService {
    type ExecuteStream = Pin<Box<dyn Stream<Item = Result<ExecuteMessage, Status>> + Send>>;

    async fn execute(
        &self,
        request: Request<tonic::Streaming<ExecuteMessage>>,
    ) -> Result<Response<Self::ExecuteStream>, Status> {
        let mut in_stream = request.into_inner();

        // 1. Read first message (must be StartRequest)
        let first_msg = match in_stream.next().await {
            Some(Ok(msg)) => msg,
            Some(Err(e)) => return Err(Status::aborted(format!("Stream error: {}", e))),
            None => return Err(Status::invalid_argument("Empty stream")),
        };

        let start_req = match first_msg.message {
            Some(wasmee::execute_message::Message::Start(s)) => s,
            _ => return Err(Status::invalid_argument("First message must be StartRequest")),
        };

        // Convert oplogs to engine representation
        let initial_oplog: Vec<engine::OplogEntry> = start_req
            .oplog
            .into_iter()
            .map(|entry| engine::OplogEntry {
                call_index: entry.call_index,
                api_name: entry.api_name,
                request_payload: entry.request_payload,
                response_payload: entry.response_payload,
            })
            .collect();

        // Convert deltas to engine representation
        let mut memory_deltas = HashMap::new();
        for (page, data) in start_req.memory_deltas {
            memory_deltas.insert(page, data);
        }

        // Channels for Wasmtime thread to speak with tokio gRPC loop
        let (vm_msg_tx, mut vm_msg_rx) = mpsc::unbounded_channel::<engine::EngineToHostMsg>();

        // Spawn WASM execution in a blocking thread
        let wasm_bytes = start_req.wasm_bytes;
        let entrypoint = start_req.entrypoint;
        let params = start_req.params;
        let instance_id = start_req.instance_id;
        let base_snapshot = start_req.base_snapshot;
        let initial_call_index = start_req.call_index;

        let mut run_handle = tokio::task::spawn_blocking(move || {
            engine::run_wasm(
                &wasm_bytes,
                &entrypoint,
                &params,
                instance_id,
                &base_snapshot,
                &memory_deltas,
                initial_oplog,
                initial_call_index,
                vm_msg_tx,
            )
        });

        // gRPC output channel
        let (grpc_out_tx, grpc_out_rx) = mpsc::unbounded_channel::<Result<ExecuteMessage, Status>>();

        // Spawn coordination task
        tokio::spawn(async move {
            let mut run_completed = false;
            let mut run_result = None;

            loop {
                tokio::select! {
                    // Receive message from the Wasmtime runner thread
                    vm_msg = vm_msg_rx.recv(), if !run_completed => {
                        match vm_msg {
                            Some(engine::EngineToHostMsg::HostCall { api_name, request, call_index, resp_tx }) => {
                                // Send HostCallRequest to Go over gRPC stream
                                let msg = ExecuteMessage {
                                    message: Some(wasmee::execute_message::Message::HostCallRequest(HostCallRequest {
                                        api_name,
                                        request_payload: request,
                                        call_index,
                                    })),
                                };
                                if grpc_out_tx.send(Ok(msg)).is_err() {
                                    let _ = resp_tx.send(Err("gRPC send error".to_string()));
                                    break;
                                }

                                // Wait for Go response from gRPC input stream
                                match in_stream.next().await {
                                    Some(Ok(resp_msg)) => {
                                        match resp_msg.message {
                                            Some(wasmee::execute_message::Message::HostCallResponse(r)) => {
                                                if !r.error.is_empty() {
                                                    let _ = resp_tx.send(Err(r.error));
                                                } else {
                                                    let _ = resp_tx.send(Ok(r.response_payload));
                                                }
                                            }
                                            _ => {
                                                let _ = resp_tx.send(Err("Invalid message type received, expected HostCallResponse".to_string()));
                                            }
                                        }
                                    }
                                    Some(Err(e)) => {
                                        let _ = resp_tx.send(Err(format!("Stream error: {}", e)));
                                        break;
                                    }
                                    None => {
                                        let _ = resp_tx.send(Err("Stream closed by host".to_string()));
                                        break;
                                    }
                                }
                            }
                            Some(engine::EngineToHostMsg::Checkpoint { memory, resp_tx }) => {
                                // Send CheckpointRequest to Go over gRPC stream
                                let msg = ExecuteMessage {
                                    message: Some(wasmee::execute_message::Message::Checkpoint(CheckpointRequest {
                                        current_memory: memory,
                                    })),
                                };
                                if grpc_out_tx.send(Ok(msg)).is_err() {
                                    let _ = resp_tx.send(Err("gRPC send error".to_string()));
                                    break;
                                }

                                // Wait for Go checkpoint response
                                match in_stream.next().await {
                                    Some(Ok(resp_msg)) => {
                                        match resp_msg.message {
                                            Some(wasmee::execute_message::Message::CheckpointResponse(r)) => {
                                                if !r.error.is_empty() {
                                                    let _ = resp_tx.send(Err(r.error));
                                                } else {
                                                    let _ = resp_tx.send(Ok(()));
                                                }
                                            }
                                            _ => {
                                                let _ = resp_tx.send(Err("Invalid message type, expected CheckpointResponse".to_string()));
                                            }
                                        }
                                    }
                                    Some(Err(e)) => {
                                        let _ = resp_tx.send(Err(format!("Stream error: {}", e)));
                                        break;
                                    }
                                    None => {
                                        let _ = resp_tx.send(Err("Stream closed by host".to_string()));
                                        break;
                                    }
                                }
                            }
                            None => {}
                        }
                    }

                    // Await Wasmtime runner thread completion
                    res = &mut run_handle, if !run_completed => {
                        run_completed = true;
                        match res {
                            Ok(res) => run_result = Some(res),
                            Err(e) => {
                                let err_msg = format!("WASM task panicked: {}", e);
                                let msg = ExecuteMessage {
                                    message: Some(wasmee::execute_message::Message::Complete(CompleteResponse {
                                        crashed: true,
                                        error: err_msg,
                                        final_deltas: HashMap::new(),
                                        final_oplog: vec![],
                                    })),
                                };
                                let _ = grpc_out_tx.send(Ok(msg));
                            }
                        }
                    }
                }

                if run_completed {
                    if let Some(res) = run_result {
                        // Map deltas
                        let mut final_deltas = HashMap::new();
                        for (page, data) in res.final_deltas {
                            final_deltas.insert(page, data);
                        }

                        // Map oplog
                        let final_oplog = res.final_oplog
                            .into_iter()
                            .map(|entry| ProtoOplogEntry {
                                call_index: entry.call_index,
                                api_name: entry.api_name,
                                request_payload: entry.request_payload,
                                response_payload: entry.response_payload,
                            })
                            .collect();

                        let msg = ExecuteMessage {
                            message: Some(wasmee::execute_message::Message::Complete(CompleteResponse {
                                crashed: res.crashed,
                                error: res.error,
                                final_deltas,
                                final_oplog,
                            })),
                        };
                        let _ = grpc_out_tx.send(Ok(msg));
                    }
                    break;
                }
            }
        });

        let out_stream = UnboundedReceiverStream::new(grpc_out_rx);
        Ok(Response::new(Box::pin(out_stream) as Self::ExecuteStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = WasmeeService;

    println!("Wasmee Rust execution engine listening on {}", addr);

    Server::builder()
        .add_service(WasmeeExecutorServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
