# Plan: Add Blog Section & Performance Documentation (WASMEE-104)

We will implement a premium Blog section on the Wasmee landing page and add detailed documentation explaining the execution trade-offs between durable state checkpointing and raw performance.

## User Review Required

> [!IMPORTANT]
> **Client-Side Routing Approach**
> We will implement a client-side hash-based router (`#home`, `#blog`, `#blog/durable-wasm-performance`) in [main.ts](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/main.ts). This avoids multi-page compilation complexity in Vite and provides instant, animated transitions between the landing page and the blog.

## Proposed Changes

---

### 1. Repository Documentation

We will add a new section in the README files explaining the performance trade-offs of durable execution (checkpointing, instantiation, and page-dirty-page hash comparisons) to explain why the throughput is 25,000+ RPS instead of 100,000+ RPS.

#### [MODIFY] [README.md](file:///Users/user/gitlab.com/wasmee/wasmee/README.md)
* Add a "Performance Trade-offs: Durable vs Stateless" section.
* Explain the overhead of re-instantiating, memory copying, and scanning pages for deltas.

#### [MODIFY] [README_ru.md](file:///Users/user/gitlab.com/wasmee/wasmee/README_ru.md)
* Add the Russian translation of the performance trade-offs section.

---

### 2. Website Frontend Layout & Styles

We will add the new Blog view, blog list cards, and full-article reader view. The styling will utilize the existing glassmorphic theme (glowing borders, deep dark backgrounds, and purple/blue blobs).

#### [MODIFY] [index.html](file:///Users/user/gitlab.com/wasmee/wasmee/site/index.html)
* Add "Blog" link to the header `<nav>` element.
* Wrap the main landing sections (Hero, Metrics, Features, Code/Fiddle, CTA) inside a `<div id="home-view">`.
* Add a `<div id="blog-view" style="display: none;">` section containing:
  * A list of blog posts in a grid card layout (Feed View).
  * A full article reader container with custom typography (Post View).

#### [MODIFY] [style.css](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/style.css)
* Add styles for the blog grid layout, blog cards (with hover scaling and neon glow), and article content.
* Add responsive styling for mobile.

---

### 3. Client-Side Routing & Blog Data

We will implement hash-based navigation and define the first blog post about the performance trade-offs of durable executions.

#### [MODIFY] [main.ts](file:///Users/user/gitlab.com/wasmee/wasmee/site/src/main.ts)
* Add a hash listener to switch between views:
  * `#home` (or empty): Show `#home-view`, hide `#blog-view`.
  * `#blog`: Show `#blog-view` in Feed mode, hide `#home-view`.
  * `#blog/<post-slug>`: Show `#blog-view` in Post mode (load article content), hide `#home-view`.
* Define the initial post: **"Understanding the Performance Trade-offs of Durable WebAssembly" / "Понимание компромиссов производительности в Durable WebAssembly"**.

---

### 4. Semantic Store Task Tracking

We will create a task checklist and maintain files under `docs/wasmee/WASMEE-104/` as required by project rules.

#### [NEW] [implementation_plan.md](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-104/implementation_plan.md)
* Duplicate of this implementation plan for the task directory.

#### [NEW] [task.md](file:///Users/user/gitlab.com/wasmee/wasmee/docs/wasmee/WASMEE-104/task.md)
* Checklist of tasks.

## Verification Plan

### Automated Tests
- Run `npm run build` inside the `site` directory to compile TypeScript and Vite assets without warnings or errors.

### Manual Verification
- Start the Vite development server using `npm run dev` in the `site` directory.
- Open the local website in the browser.
- Navigate to the "Blog" section, verify that the landing page animates out and the blog cards animate in.
- Click on the blog post about performance trade-offs, verify readability, styling, and navigation back to the main site.
