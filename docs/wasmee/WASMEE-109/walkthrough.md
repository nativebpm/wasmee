# Walkthrough: Multi-Language SDK & Guest Examples (WASMEE-109)

We have successfully established a clean architectural separation by moving the host client SDK to a dedicated repository (`wasmee-sdk`) and providing multi-language guest code examples (Rust, Go/TinyGo, JS) in our `wasm-modules` repository.

## Architectural Boundary

### 1. `wasmee-sdk` (Host Connectors) — [REPOSOTORY URL](file:///Users/user/gitlab.com/wasmee-sdk)
* **go/**: Exposes the Go client logic and runner tools to establish communication with the daemon.
  * Extracted from the main `wasmee` repository to decouple client packages from core daemon logic.
  * Verified successful compilation of the package.
* **Layout Ready**: Placeholder structure for Node.js (`js/`) and Rust (`rust/`) host clients.

### 2. `wasm-modules` (Guest Modules) — [REPOSOTORY URL](file:///Users/user/gitlab.com/wasm-modules)
Contains source code examples compiling to standard WebAssembly:
* **rust/guest/**: Standard Rust crate that compiles to `wasm32-wasip1` target using Cargo.
* **go/source/**: TinyGo implementation using standard `//go:wasmimport` and `//export` syntax to export the exchange buffer and run handlers.
* **js/source/**: JavaScript script demonstrating sandbox execution within a WASM QuickJS interpreter environment.

---

## Verification

1. Verified compilation of the extracted Go SDK using `go build ./...`.
2. Verified compilation of the Rust guest package using `cargo check`.
3. Restructured layout, committed, and pushed changes for both `wasm-modules` and `wasmee-sdk` to their remote GitLab repositories.
