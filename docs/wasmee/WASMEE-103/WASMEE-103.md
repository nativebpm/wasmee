---
task: WASMEE-103
status: Done
summary: Integrate Wasmee directly with Git using webhooks, local configuration caching, and manual sync capability in the Fiddle UI to enable real GitOps without external CI/CD pipelines.
---

# WASMEE-103: GitOps without Pipeline (Webhooks & Cache)

## Problem Statement

To achieve zero downtime, hot reload, and GitOps workflows without the overhead of external CI/CD pipelines (e.g., GitHub Actions, GitLab CI/CD), the Wasmee daemon needs to be directly integrated with Git hosts.

Currently, Wasmee fetches the WASM or ZIP from Git on every request if `git_source` is supplied. This:
1. Causes unnecessary latency per execution request.
2. Risks hitting GitHub/GitLab rate limits.
3. Does not support automatic background updates (hot-reload) when a commit is pushed to Git.

## Requirements

1. **JIT Compilation Caching of Git Sources**:
   - Cache compiled `wasmtime::Module` objects mapped to the specific `GitSource` key `(repository, git_ref, file_path)`.
   - On `/execute` requests containing a `git_source`, check the cache first. If hit, execute immediately (latency drops from seconds to microseconds).

2. **Git Webhook Integration**:
   - Implement an endpoint `/git-webhook` or `/webhook/git` in Wasmee.
   - Support push webhooks from GitHub and GitLab.
   - When a webhook payload is received, identify the matching repository and branch/ref in the cache, pull the new WASM/ZIP in the background, compile it, and replace the cached entry.
   - Subsequent execution requests will seamlessly use the updated module without restarting Wasmee, ensuring hot-reload out-of-the-box.

3. **Manual Sync Endpoint**:
   - Implement `/gitops/sync` endpoint in the daemon.
   - Accepting a `GitSource` payload, this forces a fetch from Git, compiling the module, updating the cache, and returning the new hash and logs.

4. **UI Integration**:
   - Display a "GitOps Integration" panel or section in the Fiddle UI.
   - Show the target Webhook URL for the user to register in GitHub/GitLab.
   - Provide a "Manual Sync" button. When clicked, it calls `/gitops/sync` and prints the sync steps (e.g., fetching, JIT compilation, caching) in the execution console.
