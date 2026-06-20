---
task: WASMEE-101
status: In Progress
summary: Implementation of WASM Fiddle and Git pre-warming integration for Wasmee runtime.
---

# WASMEE-101: WebAssembly Fiddle & Git Pre-Warming Integration

## Context

Wasmee is a high-performance Durable WebAssembly engine. Currently, executing a guest WASM module requires sending the raw WASM bytes or requesting a module that is already cached in memory by its hash. 

To improve the developer experience and system integration, we want to:
1. **Support Git-native loading**: Allow running modules directly by providing a Git repository URL, branch/tag, and file path.
2. **Support compressed binaries**: Allow uploading and downloading `.zip` archives containing the `.wasm` file to keep git repositories lightweight.
3. **Module Pre-Warming**: Provide a `/warmup` API endpoint to pre-compile and cache modules from Git, avoiding network and compilation overhead during execution.
4. **Interactive WASM Fiddle**: Turn the static site demo page into a live playground (Fiddle) where users can execute, inspect snapshots, and recover state interactively.

## Requirements

1. **Git Integration**:
   - Extract repository name, owner, branch, and file path.
   - Support GitHub RAW URLs (`raw.githubusercontent.com`) and GitLab RAW URLs for downloading.
   - Support authentication tokens for private repositories.
2. **ZIP Decompression**:
   - Detect if the downloaded file is a ZIP archive (by file extension or magic bytes).
   - Unzip in-memory and extract the WASM file.
3. **Pre-warming API**:
   - Implement POST `/warmup` endpoint.
   - Return compiled module hash.
4. **Fiddle Dashboard**:
   - Expand `site/index.html` with interactive inputs and control buttons.
   - Integrate with the Wasmee server to run codes and render memory state differences.
