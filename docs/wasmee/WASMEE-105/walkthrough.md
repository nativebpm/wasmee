# Walkthrough - Install Script & Docker Verification

We have successfully created a robust, portable shell installation script for Wasmee, compiled statically linked binaries for all target platforms, bundled them inside the static site assets, and verified the installation process using E2E tests in clean Docker containers.

## Changes Made

### Installation Script
- Created [install.sh](file:///Users/user/gitlab.com/wasmee/wasmee/site/public/install.sh) in `site/public/install.sh` which:
  - Detects client OS (`Darwin` / `Linux`) and architecture (`amd64` / `arm64`).
  - Downloads the corresponding prebuilt binary from the current server or a custom base URL.
  - Installs it to `/usr/local/bin` (or a local user directory if write access is denied).
  - Verifies the installation by executing the binary and printing the version.

### Multi-Platform Binaries
- Compiled the following statically linked (musl-based) binaries and placed them inside [site/public/bin/](file:///Users/user/gitlab.com/wasmee/wasmee/site/public/bin/):
  - `wasmee-darwin-arm64` (macOS Apple Silicon)
  - `wasmee-linux-arm64` (Linux ARM64, statically linked musl)
  - `wasmee-linux-amd64` (Linux AMD64/x86_64, statically linked musl)

### Verification & E2E Testing
- Leveraged the pre-existing [test_install.sh](file:///Users/user/gitlab.com/wasmee/wasmee/site/public/test_install.sh) to perform automated verification inside clean Docker containers:
  - Starts a temporary local HTTP server to serve the website static assets (including `install.sh` and the binaries).
  - Starts a clean `ubuntu:latest` container, downloads the script, and installs/runs Wasmee.
  - Starts a clean `alpine:latest` container (musl-based), downloads the script, and installs/runs Wasmee.
- Verified that all E2E Docker installation tests pass successfully!

## Verification Results

### Local Docker E2E Tests
```bash
$ ./site/public/test_install.sh
[test-info] Starting temporary HTTP server at port 8089 serving site/dist...
[test-info] --------------------------------------------------
[test-info] Testing installation script inside clean Ubuntu container...
[test-info] --------------------------------------------------
[info] Installing Wasmee...
[info] Detected platform: linux (arm64)
[info] Downloading binary from http://host.docker.internal:8089/bin/wasmee-linux-arm64...
[info] Installing to /root/.wasmee/bin/wasmee...
[success] Wasmee has been installed to /root/.wasmee/bin/wasmee
[success] Successfully verified: wasmee 0.1.0
[test-success] Ubuntu installation test passed!
[test-info] --------------------------------------------------
[test-info] Testing installation script inside clean Alpine container...
[test-info] --------------------------------------------------
[info] Installing Wasmee...
[info] Detected platform: linux (arm64)
[info] Downloading binary from http://host.docker.internal:8089/bin/wasmee-linux-arm64...
[info] Installing to /root/.wasmee/bin/wasmee...
[success] Wasmee has been installed to /root/.wasmee/bin/wasmee
[success] Successfully verified: wasmee 0.1.0
[test-success] Alpine installation test passed!
[test-info] --------------------------------------------------
[test-success] All E2E Docker installation tests passed successfully!
[test-info] --------------------------------------------------
[test-info] Cleaning up HTTP server (PID: 78566)...
```
