# WASMEE-109: Multi-Language SDK & Guest Examples

Design and implement a multi-language SDK repository architecture for Wasmee host connectors, and provide concrete guest module compilation examples across different languages.

## User Review Required

> [!IMPORTANT]
> 1. **New Public SDK Repository**: We propose creating a new public Git repository: `https://gitlab.com/wasmee/wasmee-sdk.git` (or GitHub version) to house the host client SDKs.
> 2. **Code Extraction**: We will extract the Go host client (currently `client.go`, `olme/`, `pb/` in the `wasmee` repository) and move it to `wasmee-sdk/go/` so that it is properly separated from the core Rust daemon repository.

---

## Proposed Architecture

### 1. Repository Layouts

#### `wasmee` (Core Daemon Engine)
Only contains the Rust-based execution daemon, website code, and proto files.

#### `wasmee-sdk` (Host Connectors) — [NEW REPOSITORY]
Contains SDKs to connect to the daemon from various host languages:
* **go/**: Go host client (transferred from `wasmee` repo).
* **rust/**: Rust host client (making HTTP/Protobuf requests to the daemon).
* **js/**: Node.js/TS host client.

#### `wasm-modules` (Multi-Language Guest Modules)
Contains guest code examples compiling to standard WASM:
* **rust/guest/**: Rust guest code compiling to `wasm32-wasip1`.
* **go/guest/**: TinyGo guest code compiling to `wasm32-wasip1` via `tinygo build -target=wasi`.
* **js/guest/**: JavaScript source files intended to run in a WASM-based JS interpreter.

---

## Proposed Changes

### Guest Examples in `wasm-modules`

#### [NEW] [go/guest/main.go](file:///Users/user/gitlab.com/wasm-modules/go/guest/main.go)
Create a TinyGo-compatible guest module example.

#### [NEW] [rust/guest/src/lib.rs](file:///Users/user/gitlab.com/wasm-modules/rust/guest/src/lib.rs)
Create a Rust guest module example.

#### [NEW] [js/guest/app.js](file:///Users/user/gitlab.com/wasm-modules/js/guest/app.js)
Create a JS guest module example running on a WASM QuickJS interpreter.

### Host SDK Extraction (`wasmee-sdk`)

#### [NEW] [wasmee-sdk](file:///Users/user/gitlab.com/wasmee-sdk)
Initialize the SDK monorepo structure.

---

## Verification Plan

### Automated Tests
- Verify compilation of all guest modules (Rust cargo build, TinyGo build).
- Run host Go client tests pointing to the local daemon.
