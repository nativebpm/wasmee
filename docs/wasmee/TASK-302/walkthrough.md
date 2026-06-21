# Walkthrough — Multi-Domain Workflow Example with ServiceDesk SLA and Auth (TASK-302)

This document summarizes the changes made to implement the multi-domain workflow example and the verification results.

## Changes Made

### 1. Rust Wasm Guest Crate (`servicedesk-guest`)
Created a new guest crate at [examples/servicedesk/guest](file:///Users/user/gitlab.com/wasmee/wasmee/examples/servicedesk/guest) containing:
- `Cargo.toml`: Package configuration.
- `lib.rs`: Implements the core domain logic for:
  - **Todo Checklist**: Sequential task flow.
  - **Kanban Task Board**: Multi-stage state machine.
  - **ITIL ServiceDesk**: Ticket states (`New`, `Assigned`, `Investigating`, `Resolved`), role-based authorization checks, priority-based SLA deadlines, history logs, and background SLA ticks.

### 2. Go Backend
Created the web server at [main.go](file:///Users/user/gitlab.com/wasmee/wasmee/examples/servicedesk/main.go) containing:
- In-memory `memoryStore` supporting multiple session instances.
- JSON APIs for instance creation, listing, retrieval of current state (with JSON schema widgets compiled via `jsonschema` library), and task submissions with user roles.
- Background SLA ticker routine that evaluates active tickets periodically.

### 3. Visual Frontend Dashboard
Created a responsive, premium HTML frontend at [index.html](file:///Users/user/gitlab.com/wasmee/wasmee/examples/servicedesk/index.html) featuring:
- Sleek dark theme withOutfit font, glassmorphism card layouts, and micro-animations.
- Role switcher to easily test permissions as `Customer`, `Support Engineer`, or `Service Manager`.
- Workspace view selector (Todo, Kanban, ServiceDesk).
- Visual SLA progress bars and remaining time countdowns.
- Audit history timeline showing SLA breaches and priority escalations.
- Wasmee execution logger showing JIT hot-reloads and checkpoint saves.

### 4. Wasmee Homepage Product Showcase
Updated [index.html](file:///Users/user/gitlab.com/wasmee/wasmee/site/index.html) and [translations.ts](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/translations.ts) on the landing page to transform the use cases section into a showcase of the three core products of the Wasmee ecosystem:
- **Wasmee Workflow** (Process automation for tasks, Todos, and Kanban boards).
- **Wasmee Game** (Low-latency state-synchronized engine for multiplayer reconnectivity).
- **Wasmee ServiceDesk** (Incident ticket desk with SLA tracking and role authorization).

---

## Validation Results

### 1. Compilation
The guest compiled successfully targeting Wasi:
```bash
cargo build --target wasm32-wasip1 --release --package servicedesk-guest
```

### 2. Go Unit Tests
Ran the entire workspace Go unit test suite to verify no regressions were introduced:
```bash
go test -v ./...
# Result: PASS (all tests ok)
```

### 3. Server Startup
Verified that the Go backend starts up successfully and binds to port `8086`:
```bash
go run examples/servicedesk/main.go
# Result: Wasmee Multi-Domain ServiceDesk App is running on http://localhost:8086
```
