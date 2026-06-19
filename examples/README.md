# WASMEE Guest & Host Examples

This directory demonstrates how to write and execute domain-agnostic, durable guest WebAssembly workflows on the WASMEE runner.

## 1. Custom Guest Example (`guest_custom`)

The `guest_custom` directory contains a minimal guest module written in Rust. It illustrates:
- How to receive arbitrary inputs through the shared memory **Exchange Buffer**.
- How to invoke host-side APIs via **`host_call_api`**.
- How to mark execution milestones via **`checkpoint`**.

### Compiling the Guest

To compile the custom guest Wasm module into a WebAssembly WASI binary:

```bash
# From the wasmee repository root:
cargo build --target wasm32-wasip1 --release --package guest-custom
```

This generates the compiled Wasm file at:
`target/wasm32-wasip1/release/guest_custom.wasm`

---

## 2. Running the WASMEE Server

To start the WASMEE HTTP server daemon locally (by default on port `8081`):

```bash
# Start the host server daemon with a secure API token
API_TOKEN="my-secret-key" cargo run --release
```

---

## 3. Client Execution Example

You can write clients in Go (using the [wasmee Go client](file:///Users/user/github.com/nativebpm/connectors/wasmee)) to upload the compiled guest Wasm bytes and trigger execution.

### Go Client Snippet

```go
package main

import (
	"context"
	"fmt"
	"os"

	"github.com/nativebpm/connectors/wasmee"
	"github.com/nativebpm/connectors/wasmee/olme"
)

func main() {
	// Read custom guest Wasm bytes
	wasmBytes, _ := os.ReadFile("target/wasm32-wasip1/release/guest_custom.wasm")

	ctx := context.Background()
	os.Setenv("API_TOKEN", "my-secret-key")

	// Create wasmee HTTP runner targeting the localhost daemon
	runner, _ := wasmee.NewRunner(ctx, wasmBytes, "http://localhost:8081")
	
	// Create OLME session state store
	store := newMemoryStore() // implementing olme.SnapshotStore
	state := olme.NewSessionState("session-1", store)
	_ = state.Load(ctx)
	session := wasmee.NewSession("session-1", state)

	// Call the "run_durable_workflow" entrypoint in guest-custom Wasm
	input := []byte("Alice")
	crashed, result, err := runner.Execute(ctx, session, "run_durable_workflow", input)
	
	if err != nil {
		fmt.Printf("Error: %v\n", err)
		return
	}
	
	fmt.Printf("Result: %s\n", string(result))
}
```
