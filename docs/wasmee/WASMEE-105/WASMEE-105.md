---
task: WASMEE-105
status: Done
summary: Create an install script served by the website, compile and bundle binaries for multiple platforms, and verify installation inside Docker.
---

# WASMEE-105: Install Script & Docker Verification

## Context & Motivation
Currently, the website page displays a installation command:
`curl -sSf https://wasmee.com/install.sh | sh`
However, the `install.sh` script does not exist in the repository or on the website, resulting in a 404 error when users try to download it.

We need to:
1. Create a robust, portable shell script (`install.sh`) that detects the client's OS and architecture, downloads the corresponding `wasmee` binary, and installs it in `/usr/local/bin` (or a local directory if permissions require).
2. Bundle the `wasmee` binaries directly inside the static website's static files (`site/public/bin/`) so that the script can download them from the same domain without relying on external CDNs or release assets.
3. Test that the script successfully downloads, installs, and runs `wasmee` inside a Docker container.

## Requirements
- Create `install.sh` in the website static assets directory (`site/public/install.sh`).
- Build `wasmee` for Linux (AMD64/ARM64) and macOS/Darwin (ARM64).
- Place pre-built binaries in `site/public/bin/` so they are deployed as static assets.
- Verify the installation script by running it in a clean Docker container.
