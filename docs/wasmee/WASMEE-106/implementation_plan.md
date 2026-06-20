# WASMEE-106: Running and Verifying Fiddle Code via Git

Provide the ability to run and verify the WASM code specified in the Fiddle UI by building the guest WASM binary, tracking it in Git at the correct relative path, starting the local wasmee daemon, and verifying GitOps sync and execution.

## User Review Required

> [!IMPORTANT]
> Since the remote repository `https://gitlab.com/wasmee/wasmee.git` is private, we will push the changes to GitLab. To run verification from the Fiddle, the GitLab personal access token should be provided in the **Git Token (Optional)** input of the Fiddle (if they are using the public website) or we can use local testing endpoints to ensure it works.

## Open Questions

None.

## Proposed Changes

### Guest Compilation and Tracking

#### [NEW] [wasmee_guest.wasm](file:///Users/user/gitlab.com/wasmee/wasmee/guest/target/wasm32-wasip1/release/wasmee_guest.wasm)
Compile and store the guest WASM module under the path expected by the Fiddle.

## Verification Plan

### Automated Tests
- Run `cargo test` to ensure there are no regressions.

### Manual Verification
1. Compile the guest module.
2. Commit and push `guest/target/wasm32-wasip1/release/wasmee_guest.wasm` to Git.
3. Start the local wasmee daemon: `./target/release/wasmee`.
4. Open the Fiddle and verify:
   - Provide the repo URL: `https://gitlab.com/wasmee/wasmee.git` (or the HTTPS URL).
   - Enter a valid Git token for authorization.
   - Click **Pre-Warm** / **Sync from Git**.
   - Click **Run on Wasmee**.
