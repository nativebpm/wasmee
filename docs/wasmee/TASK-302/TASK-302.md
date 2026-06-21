---
task: TASK-302
status: In Progress
summary: Implement multi-domain examples (Todo, Kanban, ITIL ServiceDesk with SLA, role-based Auth/AuthZ, and custom UI).
---

# TASK-302: Multi-Domain Workflows with SLA and Auth

## Description
The objective of this task is to provide a complete, working example illustrating the usage of Wasmee across multiple domain complexities:
1. **Todo Checklist**: Simple sequential user interaction.
2. **Kanban Task Board**: Multi-stage state machine (Backlog, In Progress, In Review, Done).
3. **ITIL ServiceDesk**: Real-world incident workflow with priorities, reaction/resolution SLAs, background timer ticks, role-based authorization, and dynamic form rendering.

## Requirements
- **Guest Wasm Engine**: A Rust guest Wasm module capable of running all three workflow styles, persisting state, enforcing role restrictions, and processing background time ticks.
- **Go Backend**: Runs multiple session contexts using memory snapshotting, supports user roles (Customer, Support, Manager), and schedules tick calls.
- **Lightweight Frontend**: Built with rich dark-mode aesthetics, custom styling, smooth animations, and a real-time console showing durable checkpoints.
