# Implementation Plan — Multi-Domain Workflow Example with ServiceDesk SLA and Auth (TASK-302)

This plan details the implementation of a comprehensive example demonstrating the Wasmee Durable WebAssembly engine across three domains: a simple Todo checklist, a Kanban task board, and a full-fledged ITIL ServiceDesk with incident SLA management, role-based authorization, and a responsive frontend dashboard.

## User Review Required

> [!IMPORTANT]
> **Guest Signatures and Compilation**:
> To support custom parameters (e.g. passing role information for authorization, separate schema boundaries, and current time for SLAs), the Rust guest crate will define explicit entrypoints: `execute`, `resume`, and `tick`. This example will reside in a new folder `examples/servicedesk` and will compile to a standalone guest Wasm binary `servicedesk_guest.wasm`.
>
> **Role-Based Authentication**:
> Authentication will be simulated via simple session selectors on the client (e.g., logging in as Customer, Support Engineer, or Service Manager). The Go backend will verify actions and pass the caller's role to the Wasmee execution context for Wasm-level authorization checks.

## Proposed Changes

### Guest Engine (Rust)

A new Rust guest package will be created under `examples/servicedesk/guest` to implement the domain state machines.

---

#### [NEW] [Cargo.toml](file:///Users/user/gitlab.com/wasmee/wasmee/examples/servicedesk/guest/Cargo.toml)
Defines the `servicedesk-guest` crate configuration and its dependencies (`serde`, `serde_json`).

#### [NEW] [lib.rs](file:///Users/user/gitlab.com/wasmee/wasmee/examples/servicedesk/guest/src/lib.rs)
Implements:
- 1MB Shared static exchange buffer.
- `execute(workflow_type_offset: u32, workflow_type_len: u32, variables_offset: u32, variables_len: u32)`: Initializes a workflow instance of type `"todo"`, `"kanban"`, or `"incident"`.
- `resume(instance_offset: u32, instance_len: u32, active_task_offset: u32, active_task_len: u32, input_offset: u32, input_len: u32, role_offset: u32, role_len: u32)`: Validates role permissions, updates variables, and transitions workflow states.
- `tick(instance_offset: u32, instance_len: u32, current_time_ns: u64)`: Evaluates SLA countdowns for active incidents and auto-escalates priority/triggers breaches.

---

### Go Backend & Frontend Web Application

A new Go server with embedded visual frontend assets.

---

#### [NEW] [main.go](file:///Users/user/gitlab.com/wasmee/wasmee/examples/servicedesk/main.go)
Implements:
- Memory state storage for multiple concurrent incident/task process sessions.
- Go routes for:
  - `POST /api/auth/login`: sets the active role (Customer, Support, Manager).
  - `GET /api/auth/me`: returns current user session metadata.
  - `GET /api/instances`: returns all active processes.
  - `POST /api/instances/create`: starts a new process instance.
  - `GET /api/instances/{id}`: returns process state and dynamic schema UI widgets.
  - `POST /api/instances/{id}/submit`: posts task completion parameters.
- Background ticker routine that invokes the `tick` guest entrypoint on active instances to enforce SLA states asynchronously.
- Embeds the HTML/JS/CSS frontend in the web server response.

---

### Task Documentation (Semantic Store)

---

#### [NEW] [TASK-302.md](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/TASK-302/TASK-302.md)
Detailed JIRA-style task sheet recording the specification of the example.

#### [NEW] [implementation_plan.md](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/TASK-302/implementation_plan.md)
A permanent copy of this implementation plan inside the Git documentation store.

#### [MODIFY] [index.md](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/index.md)
#### [MODIFY] [index_ru.md](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/index_ru.md)
Registers `TASK-302` in the master task logs.

---

## Verification Plan

### Automated Tests
- Run `cargo build --target wasm32-wasip1 --release --package servicedesk-guest` to compile the guest.
- Compile and run the Go backend: `go run examples/servicedesk/main.go`.
- Run Go unit tests in the workspace to verify nothing is broken: `go test -v ./...`.

### Manual Verification
1. Open `http://localhost:8085` in the web browser.
2. Select role `Customer`, create a new Todo task and verify it is completed.
3. Switch workspace to `Kanban Board`, create a task, log in as `Support` to move it from `Backlog` to `In Progress` and then to `Review`.
4. Switch to `ServiceDesk`, create a new Incident as `Customer` with `High` priority.
5. Log in as `Support Engineer` and verify you can view the SLA timer countdown. Verify that attempting to assign an incident as `Customer` fails with an authorization error.
6. Wait for SLA reaction limit (or simulate via API) and verify that the SLA state switches to warning/breached.
