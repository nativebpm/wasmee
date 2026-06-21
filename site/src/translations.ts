export interface TranslationDict {
  [key: string]: {
    en: string;
    ru: string;
  };
}

export const translations: TranslationDict = {
  // Navigation
  "nav-features": {
    en: "Features",
    ru: "Возможности"
  },
  "nav-benchmarks": {
    en: "Benchmarks",
    ru: "Бенчмарки"
  },
  "nav-how-it-works": {
    en: "How it Works",
    ru: "Принцип работы"
  },
  "nav-blog": {
    en: "Blog",
    ru: "Блог"
  },
  "nav-use-cases": {
    en: "Use Cases",
    ru: "Примеры использования"
  },
  "nav-get-started": {
    en: "Get Started",
    ru: "Начать работу"
  },

  // Hero Section
  "hero-release-badge": {
    en: "Wasmee v0.1.0 is officially released",
    ru: "Официальный релиз Wasmee v0.1.0"
  },
  "hero-title": {
    en: "Run WebAssembly with Extreme Speed & Durable State",
    ru: "Запуск WebAssembly с экстремальной скоростью и Durable State"
  },
  "hero-subtitle": {
    en: "Wasmee is a sandboxed WebAssembly execution engine written in Rust on top of Wasmtime. Engineered for high-density, crash-resilient executions, providing microsecond startup times and native state checkpointing.",
    ru: "Wasmee — это sandboxed-движок выполнения WebAssembly, написанный на Rust поверх Wasmtime. Спроектирован для высокоплотных (high-density) и устойчивых к сбоям (crash-resilient) вычислений, обеспечивая микросекундный запуск и нативное создание чекпоинтов состояния (state checkpointing)."
  },
  "hero-btn-install": {
    en: "Install Wasmee",
    ru: "Установить Wasmee"
  },
  "hero-btn-explore": {
    en: "Explore Features",
    ru: "Возможности"
  },

  // Terminal tabs
  "tab-title-run": {
    en: "wasmee run",
    ru: "wasmee run"
  },
  "tab-title-snapshot": {
    en: "wasmee snapshot",
    ru: "wasmee snapshot"
  },
  "tab-title-bench": {
    en: "wasmee bench",
    ru: "wasmee bench"
  },

  // Terminal Run lines
  "term-run-compiling": {
    en: "Compiling guest_process.wasm (Wasmtime JIT)...",
    ru: "Компиляция guest_process.wasm (Wasmtime JIT)..."
  },
  "term-run-initialized": {
    en: "Instance initialized. Gas configured: 10,000,000. Memory limit: 32MB.",
    ru: "Инстанс инициализирован. Конфигурация газа (gas): 10 000 000. Лимит памяти: 32 МБ."
  },
  "term-run-evaluating": {
    en: "Evaluating payload: { \"order_total\": 4500, \"customer\": \"ACME Corp\" }",
    ru: "Обработка payload: { \"order_total\": 4500, \"customer\": \"ACME Corp\" }"
  },
  "term-run-exchanged": {
    en: "Host variable exchanged. Calling set_variable(\"tax_rate\", 0.15)",
    ru: "Синхронизация хост-переменных выполнена (host variable exchanged). Вызов set_variable(\"tax_rate\", 0.15)"
  },
  "term-run-success": {
    en: "Task executed in 39.6 microseconds. Memory footprint: 4.2 MB.",
    ru: "Задача выполнена за 39.6 микросекунд. Memory footprint: 4.2 МБ."
  },

  // Terminal Snapshot lines
  "term-snap-step1": {
    en: "Running process step 1 of 5...",
    ru: "Выполнение шага процесса 1 из 5..."
  },
  "term-snap-warn": {
    en: "Snapshot triggered at guest execution checkpoint (Step 2).",
    ru: "Создан snapshot на чекпоинте гостевого выполнения (Шаг 2)."
  },
  "term-snap-serializing": {
    en: "Serializing VM instance memory layout (14 pages)...",
    ru: "Сериализация memory layout инстанса VM (14 страниц)..."
  },
  "term-snap-success": {
    en: "Snapshot binary written to database (280 KB). VM state saved.",
    ru: "Бинарный файл snapshot записан в БД (280 КБ). Состояние VM сохранено."
  },
  "term-snap-resume": {
    en: "VM state restored. Resuming step 3 with full memory context.",
    ru: "Состояние VM восстановлено. Возобновление шага 3 с полным memory context."
  },

  // Terminal Bench lines
  "term-bench-warmup": {
    en: "Warmup completed. Starting load benchmark...",
    ru: "Прогрев завершен. Запуск нагрузочного теста..."
  },
  "term-bench-running": {
    en: "Running 50 virtual users.",
    ru: "Запуск 50 виртуальных пользователей."
  },
  "term-bench-latency-p50": {
    en: "Warm Resume Latency (p50):",
    ru: "Warm Resume Latency (p50):"
  },
  "term-bench-latency-p95": {
    en: "Warm Resume Latency (p95):",
    ru: "Warm Resume Latency (p95):"
  },
  "term-bench-throughput-vm": {
    en: "Throughput (In-Memory VM):",
    ru: "Throughput (In-Memory VM):"
  },
  "term-bench-throughput-http": {
    en: "Throughput (HTTP API):",
    ru: "Throughput (HTTP API):"
  },
  "term-bench-sla": {
    en: "SLA Checks Pass Rate:",
    ru: "SLA Checks Pass Rate:"
  },

  // Metrics Section
  "metric-label-latency": {
    en: "Warm Resume Latency",
    ru: "Warm Resume Latency"
  },
  "metric-desc-latency": {
    en: "Fastest state recovery among modern WebAssembly runner platforms.",
    ru: "Самое быстрое восстановление состояния (Warm Resume) среди современных WebAssembly-платформ."
  },
  "metric-label-sandboxing": {
    en: "Memory Sandboxing",
    ru: "Memory Sandboxing"
  },
  "metric-desc-sandboxing": {
    en: "Zero-trust memory boundary controls preventing CPU leaks and heap-overruns.",
    ru: "Контроль границ памяти по принципу Zero-Trust, предотвращающий утечки ресурсов и переполнение кучи (heap overruns)."
  },
  "metric-label-throughput": {
    en: "In-Memory RPS",
    ru: "In-Memory RPS"
  },
  "metric-desc-throughput": {
    en: "Blazing throughput executing sandboxed WebAssembly tasks on a single node.",
    ru: "Экстремальная пропускная способность при выполнении sandboxed WebAssembly-задач на одном узле."
  },
  "metric-label-safety": {
    en: "Pure Sandboxed Safety",
    ru: "Pure Sandboxed Safety"
  },
  "metric-desc-safety": {
    en: "Completely blocks untrusted network access, filesystem writes, and process fork APIs.",
    ru: "Полное блокирование несанкционированного доступа к сети, записи в файловую систему и вызовов API создания процессов (fork)."
  },
  "metric-cta-text": {
    en: "Want to run these benchmarks yourself?",
    ru: "Хотите запустить эти бенчмарки самостоятельно?"
  },
  "metric-cta-btn": {
    en: "View Load Testing Guide & Source",
    ru: "Инструкция по нагрузочному тестированию"
  },

  // Features Section
  "features-title": {
    en: "Designed for Durable Micro-Tasks",
    ru: "Спроектировано для Durable Micro-Tasks"
  },
  "features-subtitle": {
    en: "Why teams choose Wasmee for executing business logic and untrusted user-submitted code.",
    ru: "Почему команды выбирают Wasmee для выполнения бизнес-логики и выполнения untrusted пользовательского кода."
  },
  "feature-title-snapshots": {
    en: "Durable Snapshots",
    ru: "Durable Snapshots"
  },
  "feature-desc-snapshots": {
    en: "Serialize full memory states to PostgreSQL or AWS S3. Restore execution seamlessly even if the host machine crashes mid-step.",
    ru: "Сериализация полного состояния памяти (memory state) в PostgreSQL или AWS S3. Бесшовное восстановление выполнения, даже если хост-машина дает сбой посреди шага (mid-step)."
  },
  "feature-title-jit": {
    en: "Rust-Native JIT Performance",
    ru: "Rust-Native JIT Performance"
  },
  "feature-desc-jit": {
    en: "Compiles guest WebAssembly modules directly into machine code via Wasmtime Cranelift JIT. No virtual containers, no JVM overhead.",
    ru: "Компиляция гостевых модулей WebAssembly напрямую в машинный код с помощью JIT-компилятора Wasmtime Cranelift. Без виртуальных контейнеров и накладных расходов JVM."
  },
  "feature-title-security": {
    en: "Zero-Trust Security",
    ru: "Zero-Trust Security"
  },
  "feature-desc-security": {
    en: "Strict execution constraints. Blocks unauthorized host network calls, directory listings, command lines, and OS threads.",
    ru: "Жесткие ограничения выполнения. Блокировка неавторизованных хост-вызовов сети, листинга директорий, доступа к командной строке и потоков ОС."
  },
  "feature-title-coldstarts": {
    en: "Microsecond Cold Starts",
    ru: "Microsecond Cold Starts (холодный старт)"
  },
  "feature-desc-coldstarts": {
    en: "Keeps a hot-cache pool of compiled modules ready to execute instantly. Reduces guest initialization overhead to near-zero.",
    ru: "Поддержание пула горячего кэша (hot-cache pool) скомпилированных модулей для мгновенного выполнения. Снижает оверхед на инициализацию гостя практически до нуля."
  },
  "feature-title-gas": {
    en: "Gas-Metered Execution",
    ru: "Gas-Metered Execution"
  },
  "feature-desc-gas": {
    en: "Prevent resource exhaustion and infinite loops by defining strict execution limits (gas budgets) for untrusted guest tasks.",
    ru: "Предотвращение бесконечных циклов и перерасхода ресурсов путем задания жестких лимитов выполнения (gas budget) для untrusted гостевых задач."
  },
  "feature-title-exchange": {
    en: "Seamless Variable Exchange",
    ru: "Seamless Variable Exchange"
  },
  "feature-desc-exchange": {
    en: "Bidirectional serialization mappings. Read/write execution context variables using simple import functions directly from guest Wasm.",
    ru: "Двунаправленная сериализация и маппинг. Чтение/запись переменных контекста выполнения с помощью простых функций импорта прямо из гостевого Wasm."
  },

  // Code Playground Section
  "code-title": {
    en: "Write Once. Run and Persist Anywhere.",
    ru: "Write Once. Run & Persist Anywhere."
  },
  "code-subtitle": {
    en: "How guest scripts leverage Wasmee host bindings for state-resilient executions.",
    ru: "Как гостевые скрипты используют host bindings в Wasmee для отказоустойчивого выполнения."
  },
  "fiddle-label-repo": {
    en: "Git Repository URL (GitHub/GitLab)",
    ru: "URL Git-репозитория (GitHub/GitLab)"
  },
  "fiddle-label-ref": {
    en: "Git Ref (Branch/Tag)",
    ru: "Git Ref (Ветка/Тег)"
  },
  "fiddle-label-path": {
    en: "File Path (WASM or ZIP)",
    ru: "Путь к файлу (WASM или ZIP)"
  },
  "fiddle-label-token": {
    en: "Git Token (Optional)",
    ru: "Git Token (необязательно)"
  },
  "fiddle-label-gas": {
    en: "Max Gas (Fuel Limit)",
    ru: "Max Gas (лимит выполнения)"
  },
  "fiddle-label-memory": {
    en: "Max Memory limit (MB)",
    ru: "Лимит памяти (МБ)"
  },
  "fiddle-label-input": {
    en: "Input State (JSON)",
    ru: "Входное состояние (JSON payload)"
  },
  "fiddle-btn-warm": {
    en: "Pre-Warm (JIT)",
    ru: "Pre-Warm (JIT)"
  },
  "fiddle-btn-sync": {
    en: "Sync from Git",
    ru: "Sync from Git"
  },
  "fiddle-btn-exec": {
    en: "Run on Wasmee",
    ru: "Run on Wasmee"
  },
  "fiddle-console-title": {
    en: "Execution Console",
    ru: "Execution Console"
  },
  "fiddle-gitops-title": {
    en: "⚡ GitOps Webhook (Zero Pipeline Hot-Reload)",
    ru: "⚡ GitOps Webhook (Zero Pipeline Hot-Reload)"
  },
  "fiddle-gitops-desc": {
    en: "Register this URL as a Webhook in GitHub or GitLab to automatically hot-reload the JIT cache on push:",
    ru: "Зарегистрируйте этот URL как Webhook в GitHub или GitLab для автоматического hot-reload JIT-кэша при push:"
  },

  "usecases-title": {
    en: "Real-World Use Cases",
    ru: "Примеры использования"
  },
  "usecases-subtitle": {
    en: "How organizations and developers leverage Wasmee for high-density, secure executions.",
    ru: "Как организации и разработчики используют Wasmee для высокоплотных и безопасных вычислений."
  },
  "usecase-title-business": {
    en: "Business & Workflow Automation",
    ru: "Автоматизация бизнеса и процессов"
  },
  "usecase-desc-business": {
    en: "Business automation platforms use Wasmee to execute custom business logic, untrusted script tasks, and declarative workflow transitions inside a secure, gas-metered sandbox with durable execution state.",
    ru: "Платформы автоматизации бизнеса используют Wasmee для выполнения пользовательской бизнес-логики, untrusted задач-сценариев (script tasks) и декларативных переходов процессов в безопасной sandboxed-среде с учетом газа (gas-metered) и Durable-состоянием выполнения (durable execution state)."
  },
  "usecase-title-gamedev": {
    en: "Game Development & Resilient Multiplayer",
    ru: "Game Development & Resilient Multiplayer"
  },
  "usecase-desc-gamedev": {
    en: "In multiplayer game development, Wasmee acts as a lightweight, state-synchronized engine. If a player disconnects or the host node crashes, Wasmee instantly restores the authoritative game memory state from the last page checkpoint, ensuring zero-overhead reconnection and fair play.",
    ru: "В разработке мультиплеерных игр Wasmee выступает в роли легковесного движка с синхронизацией состояния. При отключении игрока или сбое хост-сервера Wasmee мгновенно восстанавливает авторитетное состояние игровой памяти из последнего чекпоинта (page checkpoint), гарантируя быстрое переподключение без накладных расходов (zero-overhead) и честную игру."
  },
  "usecase-title-sync": {
    en: "Frontend & Backend State Synchronization",
    ru: "Синхронизация состояния Frontend и Backend"
  },
  "usecase-desc-sync": {
    en: "Wasmee acts as an authoritative, lightweight state store for multi-client applications. It enables seamless state synchronization between interactive frontends and backend services by persisting the exact virtual machine memory checkpoints, eliminating complex cache layers and synchronization protocols.",
    ru: "Wasmee выступает в качестве авторитетного легковесного хранилища состояния (state store) для мультиклиентских приложений. Он обеспечивает бесшовную синхронизацию состояния между интерактивными фронтендами и бэкенд-сервисами путем сохранения точных чекпоинтов памяти виртуальной машины, устраняя необходимость в сложных кэш-слоях и протоколах синхронизации."
  },

  // CTA Section
  "cta-title": {
    en: "Build High-Density Durable Executions",
    ru: "Создавайте высокоплотные Durable Executions"
  },
  "cta-subtitle": {
    en: "Run untrusted user logic, scale microservices, or orchestrate complex execution pipelines at the speed of native code.",
    ru: "Запускайте untrusted пользовательский код, масштабируйте микросервисы и оркеструйте распределенные процессы со скоростью нативного приложения."
  },
  "cta-btn-github": {
    en: "View on GitHub",
    ru: "Открыть на GitHub"
  },
  "cta-btn-docs": {
    en: "View SDK Guides",
    ru: "SDK Guides"
  },

  // Blog View Section
  "blog-title": {
    en: "Wasmee Blog",
    ru: "Блог Wasmee"
  },
  "blog-subtitle": {
    en: "Insights, technical deep-dives, and updates from the Wasmee core team.",
    ru: "Обзоры, технические статьи и новости от команды разработчиков Wasmee."
  },
  "blog-btn-back": {
    en: "Back to Blog",
    ru: "Назад в блог"
  },

  // Footer Section
  "footer-copyright": {
    en: "&copy; 2026 Wasmee Authors. Released under the Apache 2.0 License.",
    ru: "&copy; 2026 Авторы Wasmee. Лицензия Apache 2.0."
  }
};
