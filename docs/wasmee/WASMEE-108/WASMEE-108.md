---
task: WASMEE-108
status: In Progress
summary: Implement a robust and native Go-module integration for wasm-modules using go:embed and versioned packaging, avoiding external runtime fetching.
---

# WASMEE-108: Go-Module Style Integration for WebAssembly Modules

## Context & Goal
The user wants to implement a convenient, safe, and reliable way to work with `wasm-modules` as if they were Go modules, without custom hacks (костылей). 

## Requirements
1. Define a clear architecture for versioning and distributing WASM modules.
2. Support Go-native compilation and integration.
3. Compare different packaging/distribution approaches.
4. Implement a reference layout and compile scripts for `wasm-modules`.
