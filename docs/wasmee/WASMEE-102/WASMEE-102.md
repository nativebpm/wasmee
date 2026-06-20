---
task: WASMEE-102
status: In Progress
summary: Refactoring Wasmee to a stateless runtime by removing the global RustStore persistence layer.
---

# WASMEE-102: Stateless Runtime Refactoring

## Context

Wasmee is currently designed with an in-memory persistence store (`RustStore`) that saves memory snapshots, checkpoints, and execution oplogs globally within the daemon. While useful for local testing, this stateful nature limits the scalability of Wasmee. 

To enable horizontal scaling, we want to:
1. **Remove global session state**: Eliminate `RustStore` from the backend daemon.
2. **Shift state responsibility**: Ensure that Wasmee acts as a pure stateless compute node. The caller (orchestrator) is responsible for supplying the previous session state (`memory_deltas`, `oplog`, `base_snapshot`) in the execution request, and saving the updated state returned in the execution response.
3. **Preserve GitOps & Fiddle**: Keep the Git loading, pre-warming, and interactive Fiddle frontend fully functional, as they are stateless utility components.

## Requirements

1. **Rust Backend**:
   - Delete `RustStore` and remove it from `VMState` and function signatures.
   - Refactor host function imports (`checkpoint`, `host_call_api`, `host_get_time`) to write only to local session logs within the virtual machine instance context.
2. **Axum Handlers**:
   - Clean up `AppState` and Axum handlers to remove stateful database variables.
3. **Testing**:
   - Ensure the build is clean and tests pass without compilation issues.
