---
task: WASMEE-109
status: In Progress
summary: Design and implement a multi-language SDK repository (wasmee-sdk) and create multi-language guest module examples.
---

# WASMEE-109: Multi-Language SDK & Guest Examples

## Context & Goal
The user wants to establish a clear SDK boundary by creating a separate public repository for Wasmee host SDKs (connectors) and providing concrete examples of guest WASM modules written in different languages (Rust, Go/TinyGo, JS/TS).

## Requirements
1. Define the repository structure for `wasmee-sdk` (Go, Rust, Node.js host clients).
2. Create guest module compilation examples for:
   * **Rust**
   * **Go/TinyGo**
   * **JavaScript/TypeScript**
3. Create the implementation plan and align with the user.
