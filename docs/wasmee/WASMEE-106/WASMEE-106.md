---
task: WASMEE-106
status: In Progress
summary: Build the guest WebAssembly module, commit and push it to the Git repository, and run/verify it using the local wasmee daemon and the Fiddle UI.
---

# WASMEE-106: Running and Verifying Fiddle Code via Git

## Context & Goal
The user wants to verify and run the WebAssembly code specified in the Fiddle UI. To achieve this, the compiled guest `.wasm` file must be present in the Git repository at the exact path expected by the Fiddle (`guest/target/wasm32-wasip1/release/wasmee_guest.wasm`). We need to build this module locally, place it in the correct path within the Git repository, push it, run the local wasmee daemon, and verify it using the Fiddle.

## Requirements
1. Build the guest WASM module: `cargo build --target wasm32-wasip1 --release --package wasmee-guest`.
2. Copy the resulting binary from `target/wasm32-wasip1/release/wasmee_guest.wasm` to `guest/target/wasm32-wasip1/release/wasmee_guest.wasm`.
3. Add and commit the compiled `.wasm` binary to the Git repository.
4. Push the changes to the remote GitLab repository.
5. Start the local wasmee daemon on port 8081.
6. Verify synchronization and execution from the Fiddle.
