# Implementation Plan: Stateless Wasmee Engine Refactoring

This plan outlines the refactoring of Wasmee into a stateless compute node by removing the global `RustStore` persistence layer, while preserving the internal GitOps module loader and Fiddle UI integration.

## User Review Required

> [!IMPORTANT]
> By removing the global `RustStore`, the Wasmee daemon will no longer keep track of memory checkpoints, snapshots, or execution oplogs in its memory between HTTP requests. 
> The caller (e.g. NativeBPM orchestrator) must supply the `base_snapshot`, `memory_deltas`, and the previous `oplog` in every `/execute` request, and store the returned updated state.

## Proposed Changes

We will remove `RustStore` from the core engine state, making `/execute` completely stateless in terms of session storage.

---

### Component 1: Wasmee Core Engine (Rust Daemon)

#### [MODIFY] [src/engine.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/engine.rs)
- Remove `RustStore` struct definition (lines 88-95).
- Remove `store: RustStore` field from `VMState` struct.
- Update `run_wasm` and `run_wasm_precompiled` signatures to remove the `store: RustStore` parameter.
- Update the `checkpoint` host function import:
  - Remove all code writing memory, deltas, and metadata to the global `local_store`.
  - Keep calculations for local `state.checkpoints` and local `state.page_hashes` (needed to calculate state transitions).
- Update the `host_get_time` and `host_call_api` host functions:
  - Remove all code writing oplog entries to the global `local_store.oplogs`.

#### [MODIFY] [src/main.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/main.rs)
- Remove `store` field from `AppState` struct (lines 19-20).
- Remove the initialization of `RustStore` inside `main()` (line 50) and inside `AppState` creation.
- Update `execute_handler` to avoid passing `state.store` to `run_wasm_precompiled`.

---

### Component 2: Verification and Cleanup

#### [MODIFY] [src/git_resolver.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/git_resolver.rs)
- Verify that tests do not rely on `RustStore`. (Our unit tests in `git_resolver` only test raw url builders and ZIP extraction, so they are not affected).

## Verification Plan

### Automated Tests
- Run `cargo test` to verify compilation and passing of unit tests.
- Add an integration-level test to verify that `/execute` correctly processes inputs, calculates deltas, and returns checkpoints without saving state internally.

### Manual Verification
- Deploy and start the stateless Wasmee daemon.
- Execute a WASM task multiple times via the Fiddle console, verifying that the returned deltas and checkpoints are correctly generated and sent back to the browser.
