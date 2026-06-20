# WASMEE-108: Go-Module Style Integration for WebAssembly Modules

Implement a convenient, safe, and reliable way to work with `wasm-modules` as if they were Go modules, leveraging standard Go mechanisms without third-party registry overhead.

## Architecture Proposal & Comparison

We compared two primary architectures for distributing and using WASM modules within Go host applications:

### Option 1: Go-Native Packaging (`go:embed`) — RECOMMENDED

Compile the guest WASM binary and publish it inside a versioned Go package under `gitlab.com/wasmee/wasm-modules/guest`. A simple Go file embeds the bytes using `//go:embed`.

| Aspect | Details |
| :--- | :--- |
| **Security & Safety** | 100% reliable. The WASM module is compiled directly into the host Go binary. No runtime network calls. |
| **Versioning** | Uses native Go modules. Versioning, pinning, and checksum validation (`go.sum`) are handled by Go natively (e.g., `go get gitlab.com/wasmee/wasm-modules@v1.0.0`). |
| **Tooling** | Uses standard Go compiler and cache. No external CLI, package manager, or registry required. |
| **Trade-offs** | Host binary size increases slightly. Updating the WASM code requires rebuilding the host application. |

### Option 2: Runtime OCI Registry / GitOps Download

Publish the WASM module to an OCI registry (or Git repository) and fetch it dynamically over HTTPS at runtime during execution.

| Aspect | Details |
| :--- | :--- |
| **Security & Safety** | Vulnerable to network downtime, API rate limits, or registry failures at runtime. |
| **Versioning** | Managed by custom tags. If tags are mutable (e.g., `latest`), execution behavior can change unexpectedly. |
| **Tooling** | Requires setting up an OCI registry or dealing with private Git repository credentials inside the host daemon. |
| **Trade-offs** | Host binary size remains small, and updates can be hot-reloaded. |

> [!TIP]
> **Option 1 (Go-Native Packaging)** is the recommended path because it is entirely standard, uses zero "hacks", and guarantees production-grade reliability and security since all dependencies are verified by `go.sum` and embedded at compile-time.

---

## Proposed Implementation (Option 1)

1. Create a `go.mod` file at the root of `wasm-modules`.
2. Under each module folder (e.g., `guest/`), place:
   * `guest.wasm`: The compiled WebAssembly binary.
   * `guest.go`: The embedding Go code.
3. Users import it directly:
   ```go
   import "gitlab.com/wasmee/wasm-modules/guest"
   
   // guest.WasmBytes is now available as a []byte slice
   ```

### Proposed Files

#### [NEW] [go.mod](file:///Users/user/gitlab.com/wasmee/wasm-modules/go.mod)
Initialize the Go module for the repository.

#### [NEW] [guest.go](file:///Users/user/gitlab.com/wasmee/wasm-modules/guest/guest.go)
Expose the embedded WASM bytes.

## Verification Plan

### Manual Verification
1. Create the `wasm-modules` Go module project structure locally.
2. Embed the guest WASM binary.
3. Write a small Go test script that imports this local module and executes it using `wasmee.NewRunner` to verify integration works.
