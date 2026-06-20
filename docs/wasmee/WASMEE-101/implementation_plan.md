# Implementation Plan: Git Integration, Module Pre-Warming, and WASM Fiddle

This plan outlines the design and implementation of Git-based WASM module loading (with zip decompression and pre-warming/compilation caching) and the creation of an interactive WebAssembly Fiddle for Wasmee.

## User Review Required

> [!IMPORTANT]
> To support Git integration, Wasmee will need to make outbound HTTP requests to download files from Git providers (e.g., GitHub, GitLab). We will use a lightweight HTTP client in Rust (`reqwest` or `ureq`).

> [!TIP]
> Pre-warming allows developers to register Git sources in advance. The WASM module is downloaded, unzipped, and compiled into a `wasmtime::Module` in-memory. Subsequent execution requests using the module's hash will execute with microsecond latency (cold start bypassed).

## Proposed Changes

We will split the implementation into two components:
1. **Wasmee Daemon (Rust Engine)**: Add Git downloading, ZIP extraction, and a `/warmup` HTTP endpoint.
2. **Wasmee Site (Fiddle)**: Turn the static code playground on the landing page into a live interactive Fiddle that communicates with a local/hosted Wasmee instance.

---

### Component 1: Wasmee Daemon (Rust Engine)

We need to add dependencies to `Cargo.toml`:
- `reqwest` (with `rustls-tls` to avoid dependency on system openssl, and `async` capabilities)
- `zip` (for extracting `.zip` files in memory)

#### [MODIFY] [Cargo.toml](file:///Users/user/gitlab.com/wasmee/wasmee/Cargo.toml)
Add the following dependencies:
```toml
reqwest = { version = "0.11", default-features = false, features = ["rustls-tls", "tokio-rustls"] }
zip = { version = "0.6", default-features = false, features = ["deflate"] }
```

#### [MODIFY] [wasmee.proto](file:///Users/user/gitlab.com/wasmee/wasmee/wasmee.proto)
Add `GitSource` message and update `ExecuteRequest` to support resolving modules via Git:
```protobuf
message GitSource {
  string repository = 1; // e.g. "https://github.com/user/repo"
  string branch = 2;     // e.g. "main"
  string file_path = 3;  // e.g. "dist/app.zip" or "app.wasm"
  string git_token = 4;  // Optional authorization token for private repos
}

message ExecuteRequest {
  // ... existing fields ...
  bytes wasm_bytes = 8;
  string wasm_hash = 9;
  
  // New field
  GitSource git_source = 10;
}

// New messages for pre-warming API
message WarmupRequest {
  GitSource git_source = 1;
}

message WarmupResponse {
  bool success = 1;
  string wasm_hash = 2;
  string error = 3;
}
```

#### [MODIFY] [src/main.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/main.rs)
- Implement `/warmup` axum endpoint.
- Enhance `/execute` handler: if `wasm_bytes` and `wasm_hash` are not provided (or hash is not in cache) but `git_source` is defined, asynchronously fetch the file, extract zip if needed, compile, cache, and execute.
- Extract Git downloading and unzip logic to a separate helper module.

#### [NEW] [src/git_resolver.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/git_resolver.rs)
Create a helper to resolve Git files:
1. Translate Git repo URL and file path to a direct RAW download URL (supporting GitHub, GitLab, and generic HTTP).
2. Fetch the bytes over HTTP.
3. If the path ends in `.zip`, read the zip archive in-memory using the `zip` crate and extract the `.wasm` file.
4. Return the raw WASM bytes.

---

### Component 2: WASM Fiddle (Web UI)

We will upgrade the existing website in `site/` to provide a live Fiddle.

#### [MODIFY] [site/index.html](file:///Users/user/gitlab.com/wasmee/wasmee/site/index.html)
- Add editable input panels for variables (JSON).
- Make the code block editable (or simulate code modification for QuickJS JavaScript tasks).
- Add an "Execute on Wasmee" button.
- Add visualization of checkpoints, memory deltas, and oplog records returned from the engine execution.

#### [MODIFY] [site/src/main.ts](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/main.ts)
- Implement HTTP requests to `/execute` endpoint of Wasmee.
- Parse Protobuf or JSON response (we can add JSON output option to Wasmee `/execute` endpoint for easier web integration, or decode Protobuf in JS using a lightweight decoder/protobuf.js).
- Update the execution console dynamically to display memory snapshots, step execution, and state recovery.

## Verification Plan

### Automated Tests
- Test module resolution via Mock HTTP server (using `wiremock` or similar, or unit tests with mock responses).
- Test ZIP extraction with valid and invalid ZIP structures.
- Verify JIT cache pre-warming: `/warmup` successfully populates JIT cache, and subsequent `/execute` by hash does not perform network calls.

### Manual Verification
1. Push a sample WASM file compressed as `.zip` to a test GitHub repository.
2. Trigger `/warmup` pointing to this repository.
3. Execute the module by hash and measure execution time (verify it is under 100 microseconds).
4. Run the interactive Fiddle in browser, edit input state, hit "Run", crash the execution, and resume it to verify state preservation.
