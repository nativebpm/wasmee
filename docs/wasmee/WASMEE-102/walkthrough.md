# Walkthrough: WASM Fiddle, Git Pre-Warming, & Stateless Runtime Refactoring

We have successfully implemented Git loading, ZIP archive decompression, JIT pre-warming, sandbox constraint controls, the interactive WASM Fiddle, and refactored the runtime into a stateless execution engine.

## Changes Made

### 1. Cargo.toml & Protobuf Definitions
- Added `reqwest` (with `rustls-tls` support) and `zip` (for in-memory ZIP decompression) to [Cargo.toml](file:///Users/user/gitlab.com/wasmee/wasmee/Cargo.toml).
- Added `tower-http` to [Cargo.toml](file:///Users/user/gitlab.com/wasmee/wasmee/Cargo.toml) with the `cors` feature enabled.
- Extended [wasmee.proto](file:///Users/user/gitlab.com/wasmee/wasmee/wasmee.proto):
  - Added `GitSource` struct to target a specific Git repository, git reference (branch/tag/commit), and internal file path.
  - Added `SandboxConfig` message containing `max_fuel` and `max_memory_mb`.
  - Added `WarmupRequest` and `WarmupResponse` definitions.
- Configured [build.rs](file:///Users/user/gitlab.com/wasmee/wasmee/build.rs) to automatically derive `serde::Serialize` and `serde::Deserialize` on all generated Protobuf structs.

### 2. Rust Backend Daemon (git_resolver & main)
- Created [src/git_resolver.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/git_resolver.rs) to:
  - Construct RAW HTTP download URLs for GitHub and GitLab repositories.
  - Fetch files securely with optional authentication tokens.
  - Decompress ZIP archives in-memory and extract `.wasm` binaries.
  - Run comprehensive unit tests verifying url builders and ZIP extraction.
- Enhanced [src/main.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/main.rs):
  - Refactored compiler/cache module handler into `AppState::get_or_compile_module`.
  - Added `/warmup` HTTP POST endpoint to pull modules in advance.
  - Integrated Git resolution into the `/execute` handler.
  - Added full support for JSON requests and responses on both endpoints based on `Content-Type: application/json` headers (enabling simple web client connectivity).
  - Integrated CORS layer to allow frontend requests from external ports.

### 3. Wasmtime Sandboxing Config & Stateless Refactoring (WASMEE-102)
- Extended [src/engine.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/engine.rs) to read `SandboxConfig` values.
- Integrated `wasmtime::StoreLimits` and `StoreLimitsBuilder` into `VMState` and applied them to the Store via `store_obj.limiter(...)` to strictly enforce memory limits during execution.
- Dynamically applied `max_fuel` constraints to protect CPU resource abuse.
- **Stateless Refactoring**:
  - Removed `RustStore` global state database from [src/engine.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/engine.rs) and [src/main.rs](file:///Users/user/gitlab.com/wasmee/wasmee/src/main.rs).
  - Updated host bindings (`checkpoint`, `host_get_time`, `host_call_api`) to write logs and snapshots only to local VMState structures rather than global in-memory databases.
  - Removed the `store` field from `AppState` and updated `execute_handler` signatures.

### 4. Interactive Fiddle UI
- Upgraded landing page [site/index.html](file:///Users/user/gitlab.com/wasmee/wasmee/site/index.html):
  - Added a new "Live Fiddle" tab to the editor dashboard.
  - Added fields for Git repository URL, branch/tag ref, WASM/ZIP path, optional tokens, gas budgets, and memory limits.
  - Integrated **Monaco Editor** (VS Code engine) via CDN loader to provide syntax highlighting, auto-bracket completion, and live JSON validation for the payload editor.
- Configured [site/src/style.css](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/style.css) with form styles.
- Programmed [site/src/main.ts](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/main.ts):
  - Added event listeners for "Pre-Warm" and "Run" buttons.
  - Dispatched JSON payloads to the Wasmee daemon.
  - Formatted and printed response data (crashed flags, return values, oplog steps, saved checkpoints, memory deltas) directly into the simulated terminal.

## Verification Results

### Automated Tests
Ran `cargo test` verifying the unit tests for URL formatting and ZIP decompression after stateless refactoring:
```bash
running 4 tests
test git_resolver::tests::test_build_raw_url_gitlab ... ok
test git_resolver::tests::test_build_raw_url_invalid ... ok
test git_resolver::tests::test_build_raw_url_github ... ok
test git_resolver::tests::test_zip_decompression ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Manual Verification
Built TypeScript files in frontend using `npm run build`:
```bash
vite v8.0.16 building client environment for production...
transforming...✓ 5 modules transformed.
rendering chunks...
dist/index.html                 21.42 kB │ gzip: 5.68 kB
dist/assets/index-Bz18NdNI.css  13.54 kB │ gzip: 3.24 kB
dist/assets/index-QipPp66b.js    7.30 kB │ gzip: 2.79 kB
✓ built in 59ms
```
Both backend daemon and frontend build and run successfully.
