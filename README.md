# Wasmee — The Durable WebAssembly Engine

Wasmee is a high-performance, sandboxed WebAssembly runtime built in Rust on top of Wasmtime. Engineered for high-density, crash-resilient executions, providing microsecond startup times and native state checkpointing.

## Performance & Load Testing

Wasmee is designed for extreme throughput. Because guest Wasm module execution runs entirely in-memory within the host process, it achieves over **25,000+ RPS** per single CPU core.

When executing over HTTP, serialization and network bandwidth become the main constraints. With optimized Base64 serialization, the engine achieves **1,250+ RPS** directly over HTTP loopback, generating **3.6 GB/s** of memory delta traffic.

### Running Load Tests Locally

You can benchmark Wasmee locally using `k6`.

1. **Build and start the Wasmee daemon**:
   ```bash
   # Build the host daemon
   cargo build --release
   # Build the Wasm guest
   cargo build --target wasm32-wasip1 --release --package wasmee-guest
   # Start the daemon on port 8081
   ./target/release/wasmee
   ```

2. **Start the Go Benchmark Server** (requires Go workspace):
   ```bash
   # From the workspace root directory:
   go build -o connectors/wasmee/cmd/bench_server/bench_server connectors/wasmee/cmd/bench_server/main.go
   ./connectors/wasmee/cmd/bench_server/bench_server
   ```

3. **Run the k6 Load Test**:
   - **Direct load test** (bypassing the Go HTTP proxy, testing raw HTTP performance of the Rust server):
     ```bash
     BENCH_SERVER_URL=http://127.0.0.1:8081 k6 run platform/loadtest/k6/wasm_rust_direct.js --vus 50 --duration 10s
     ```
   - **Proxy load test** (testing the Go client with full state/oplog loading proxy):
     ```bash
     BENCH_SERVER_URL=http://127.0.0.1:8085 k6 run platform/loadtest/k6/wasm_rust.js --vus 50 --duration 10s
     ```
