---
task: WASMEE-104
status: In Progress
summary: Document the snapshot recovery vs execution speed trade-off and add a beautiful Blog section to the website.
---

# WASMEE-104: Blog Section & Performance Trade-off Documentation

## Context & Motivation
During performance testing, users may notice that the single-core throughput of Wasmee is around 25,000 RPS. This is highly impressive but lower than raw/stateless WebAssembly execution (which can exceed 100,000+ RPS). 

This difference is due to Wasmee's architectural features that enable durable executions (e.g., re-instantiation, memory snapshot restoration, and dirty page tracking for checkpointing).

To make these trade-offs transparent and educate users:
1. We need to document these trade-offs in the repository's `README.md` and `README_ru.md`.
2. We need to create a dedicated, highly polished Blog section on the website.
3. We need to publish a blog post explaining the trade-offs of snapshot-based memory recovery, highlighting why the overhead is a minor price to pay for crash-resiliency and durable executions.

## Requirements
- Add a new "Performance Trade-offs: Durable vs Stateless" section to `README.md` and `README_ru.md`.
- Add a "Blog" link to the site navigation.
- Implement a client-side router (hash-based) in `site/src/main.ts` supporting `#home`, `#blog`, and `#blog/<post-slug>` views.
- Design a premium, rich-aesthetic Blog interface in `site/index.html` and `site/src/style.css` matching the dark glassmorphic design of Wasmee.
- Write the initial blog post: "Understanding the Performance Trade-offs of Durable WebAssembly".
