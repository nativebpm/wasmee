#!/bin/bash
# E2E test for the Wasmee install script inside clean Docker containers

set -euo pipefail

PORT=8089
BASE_URL="http://host.docker.internal:${PORT}"

# Colors
info() { printf "\033[1;34m[test-info]\033[0m %s\n" "$1"; }
success() { printf "\033[1;32m[test-success]\033[0m %s\n" "$1"; }
error() { printf "\033[1;31m[test-error]\033[0m %s\n" "$1" >&2; }

# 1. Start Python HTTP server in site/dist
info "Starting temporary HTTP server at port ${PORT} serving site/dist..."
cd site/dist
python3 -m http.server "$PORT" >/dev/null 2>&1 &
HTTP_PID=$!
cd ../..

# Ensure HTTP server is killed on exit
cleanup() {
    info "Cleaning up HTTP server (PID: ${HTTP_PID})..."
    kill "$HTTP_PID" || true
}
trap cleanup EXIT

sleep 2

# 2. Test inside Ubuntu (AMD64/ARM64 depending on host, dynamic link check)
info "--------------------------------------------------"
info "Testing installation script inside clean Ubuntu container..."
info "--------------------------------------------------"
docker run --rm --add-host=host.docker.internal:host-gateway ubuntu:latest \
  bash -c "apt-get update -y && apt-get install -y curl && curl -sSf ${BASE_URL}/install.sh | WASMEE_BASE_URL=${BASE_URL} sh"
success "Ubuntu installation test passed!"

# 3. Test inside Alpine (checks MUSL library compilation and arm64 native execution)
info "--------------------------------------------------"
info "Testing installation script inside clean Alpine container..."
info "--------------------------------------------------"
docker run --rm --add-host=host.docker.internal:host-gateway alpine:latest \
  sh -c "apk add --no-cache curl && curl -sSf ${BASE_URL}/install.sh | WASMEE_BASE_URL=${BASE_URL} sh"
success "Alpine installation test passed!"

info "--------------------------------------------------"
success "All E2E Docker installation tests passed successfully!"
info "--------------------------------------------------"
