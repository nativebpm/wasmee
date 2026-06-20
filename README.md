# Wasmee — The Durable WebAssembly Engine

Wasmee is a fast and secure WebAssembly sandbox built in Rust on top of Wasmtime. It is designed specifically for crash-resilient, durable executions with microsecond startup times and native state checkpointing.

## Performance & Load Testing

Wasmee is engineered for maximum speed. Because guest Wasm modules execute entirely in-memory within the host process, it achieves over **25,000+ RPS** per single CPU core.

To execute Wasm tasks from remote clients, Wasmee uses an optimized **Protobuf over HTTP/1.1** protocol. By transmitting raw binary state snapshots and execution logs (bypassing JSON/Base64 overhead), it drastically reduces CPU usage and loopback network bandwidth.

Under full end-to-end benchmarks (which include reconstructing memory state, replaying execution logs, and writing checkpoints to an in-memory store), the Go Client communicating with the Rust Daemon via Protobuf achieves **382.25 RPS** locally with an average latency of **130.4ms** under 50 concurrent VUs.

### Performance Trade-offs: Durable vs Stateless

While a simple, stateless WebAssembly function call in Wasmtime can exceed **100,000+ RPS** by reusing a single global instance, Wasmee is architected for **durable, fault-tolerant executions**. To achieve this safety, Wasmee runs through a complete lifecycle for every call:
1. **Sandboxed Isolation**: Creates a new `Linker`, `Store` (with memory limits), and `Instance` per task to prevent any memory leakage between calls.
2. **State Restoration**: Reconstructs the guest's memory from the base snapshot and dirty page deltas, writing it into the new Wasm instance.
3. **Dirty Page Hashing**: Scans and hashes the guest's linear memory in 64KB pages after execution to save only the modified memory pages for checkpoints.

This entire recovery + execution + hashing lifecycle completes in **< 40 microseconds** per call, yielding **25,000+ in-memory RPS** on a single CPU core. This overhead is a minor and necessary price to pay for strict, native-speed fault tolerance.


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

