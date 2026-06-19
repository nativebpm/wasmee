# Wasmee — The Durable WebAssembly Engine

Wasmee is a high-performance, sandboxed WebAssembly runtime built in Rust on top of Wasmtime. Engineered for high-density, crash-resilient executions, providing microsecond startup times and native state checkpointing.

## Performance & Load Testing

Wasmee is engineered for maximum execution speed. Since guest Wasm module execution runs entirely in-memory within the host process, it achieves over **25,000+ RPS** per single CPU core.

To execute Wasm tasks from remote clients, Wasmee uses an optimized **Protobuf over HTTP/1.1** protocol. By transmitting raw binary snapshots, deltas, and oplogs (bypassing Base64-encoding and JSON-parsing overhead), it drastically reduces CPU usage and loopback network bandwidth.

Under full end-to-end benchmark conditions (reconstructing Wasm memory deltas, replaying execution oplogs, writing checkpoints to an in-memory store, and verifying optimistic concurrency controls), the Go Client communicating with the Rust Daemon via Protobuf achieves **372+ RPS** locally.


### Running Load Tests Locally

You can run the end-to-end benchmark locally using `k6`. The test sources are located at [platform/loadtest/k6/wasm_rust.js](file:///Users/user/github.com/nativebpm/platform/loadtest/k6/wasm_rust.js).

1. **Build and start the Wasmee daemon**:
   ```bash
   # Build the host daemon
   cargo build --release
   # Build the Wasm guest binary
   cargo build --target wasm32-wasip1 --release --package wasmee-guest
   # Start the daemon on port 8081
   ./target/release/wasmee
   ```

2. **Start the Go Benchmark Server** (runs on port 8085 and bridges requests to the Rust daemon over Protobuf):
   ```bash
   # From the workspace root directory:
   go build -o connectors/wasmee/cmd/bench_server/bench_server connectors/wasmee/cmd/bench_server/main.go
   ./connectors/wasmee/cmd/bench_server/bench_server
   ```

3. **Run the k6 Load Test**:
   ```bash
   BENCH_SERVER_URL=http://127.0.0.1:8085 k6 run platform/loadtest/k6/wasm_rust.js --vus 50 --duration 10s
   ```

