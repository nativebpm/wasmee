# Implementation Plan - Install Script & Docker Verification

Create a beautiful installation script and bundle binaries on the static site to allow seamless curl installations.

## User Review Required

> [!NOTE]
> The `wasmee` binary needs to be cross-compiled for Linux (amd64 and arm64) to support installation on both standard Linux servers/Docker containers (typically x86_64/amd64) and Apple Silicon/ARM64 servers/Docker environments.
> We will compile these binaries inside Docker to ensure they are statically linked / compatible with musl/glibc versions on most Linux distributions.

## Proposed Changes

### Build and Package System

#### [NEW] [install.sh](file:///Users/user/gitlab.com/wasmee/wasmee/site/public/install.sh)
- Detects operating system (`uname -s`) and architecture (`uname -m`).
- Determines the download URL dynamically based on the current origin of the script execution (or falling back to `test.wasmee.com` / `wasmee.com`).
- Downloads the binary and installs it to `/usr/local/bin/wasmee` or fallback `~/.local/bin/wasmee`.
- Validates the installation by printing the version.

#### [NEW] [binaries](file:///Users/user/gitlab.com/wasmee/wasmee/site/public/bin/)
- Prebuilt `wasmee-linux-amd64`
- Prebuilt `wasmee-linux-arm64`
- Prebuilt `wasmee-darwin-arm64`

### Docker Testing

#### [NEW] [test_install.sh](file:///Users/user/gitlab.com/wasmee/wasmee/site/test_install.sh)
- Starts a local container (e.g. `ubuntu` or `alpine`), installs curl, retrieves the script from the local build folder or local server, and executes it.

## Verification Plan

### Automated Tests
- Run `test_install.sh` locally to launch Docker container tests verifying binary execution.

### Manual Verification
- Deploy to `wasmee-site-test` using `deploy_wasmee_site.sh test`.
- Verify `curl -sSf https://wasmee-site-test-133825711702.us-central1.run.app/install.sh | sh` successfully installs and runs wasmee.
