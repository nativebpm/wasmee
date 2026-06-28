# Stage 1: Build the Rust runner daemon
FROM rust:1.96-slim AS builder

WORKDIR /app

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

# Copy files
COPY Cargo.toml Cargo.lock build.rs wasmee.proto ./
COPY src ./src
COPY guest ./guest

# Build the release binary
RUN cargo build --release --bin wasmee

# Stage 2: Runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install ca-certificates (required for HTTPS calls to Git hosts)
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/wasmee /app/wasmee
COPY --from=builder /app/guest/target/wasm32-wasip1/release/wasmee_guest.wasm /app/target/wasm32-wasip1/release/wasmee_guest.wasm

# Expose dynamic port
ENV PORT=8081
EXPOSE 8081

CMD ["/app/wasmee"]
