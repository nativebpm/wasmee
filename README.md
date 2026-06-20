# Wasmee — The Durable WebAssembly Engine

Wasmee is a high-performance, sandboxed WebAssembly runtime built in Rust on top of Wasmtime. Engineered for high-density, crash-resilient executions, providing microsecond startup times and native state checkpointing.

## Performance & Load Testing

Wasmee is engineered for maximum execution speed. Since guest Wasm module execution runs entirely in-memory within the host process, it achieves over **25,000+ RPS** per single CPU core.

To execute Wasm tasks from remote clients, Wasmee uses an optimized **Protobuf over HTTP/1.1** protocol. By transmitting raw binary snapshots, deltas, and oplogs (bypassing Base64-encoding and JSON-parsing overhead), it drastically reduces CPU usage and loopback network bandwidth.

Under full end-to-end benchmark conditions (reconstructing Wasm memory deltas, replaying execution oplogs, writing checkpoints to an in-memory store, and verifying optimistic concurrency controls), the Go Client communicating with the Rust Daemon via Protobuf achieves **382.25 RPS** locally with an average latency of **130.4ms** under 50 concurrent VUs.

### Performance Trade-offs: Durable vs Stateless

While a simple, stateless WebAssembly function call in Wasmtime can exceed **100,000+ RPS** by reusing a single global instance, Wasmee is architected for **durable, crash-resilient executions**. To achieve this fault tolerance and safety, Wasmee does the following for every execution:
1. **Sandboxed Isolation**: Creates a new `Linker`, `Store` (with memory limits), and `Instance` per task to prevent any memory or execution state leakage.
2. **State Restoration**: Reconstructs the guest's memory from the base snapshot and memory deltas, copying it into the newly instantiated Wasm memory.
3. **Dirty Page Hashing & Tracking**: Scans and hashes the guest's linear memory in 64KB pages after execution to identify and output dirty memory deltas for checkpoint serialization.

This entire recovery + execution + hashing lifecycle is completed in **< 40 microseconds** per call, which translates to the **25,000+ in-memory RPS** on a single CPU core. This overhead is a minor and necessary price to pay for deterministic replayability and native-speed fault tolerance.


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

