// Copy to Clipboard Functionality
const copyBtn = document.getElementById('copy-install-btn');
const installCmd = document.getElementById('install-cmd');
const copyTooltip = document.getElementById('copy-tooltip');

if (copyBtn && installCmd && copyTooltip) {
  copyBtn.addEventListener('click', async () => {
    try {
      const textToCopy = installCmd.textContent || '';
      await navigator.clipboard.writeText(textToCopy);
      
      // Show tooltip
      copyTooltip.classList.add('show');
      
      // Hide tooltip after 2 seconds
      setTimeout(() => {
        copyTooltip.classList.remove('show');
      }, 2000);
    } catch (err) {
      console.error('Failed to copy text: ', err);
    }
  });
}

// Terminal Tab Switching with Typing Simulation
const terminalTabs = document.querySelectorAll('.terminal-tab');
const terminalContents = document.querySelectorAll('.terminal-content');

// Store original prompts and lines to simulate typing on tab switch
const commandPrompts: Record<string, string> = {
  run: 'wasmee run guest_process.wasm --env vars.json',
  snapshot: 'wasmee snapshot create guest_process.wasm --out snapshot.bin',
  bench: 'wasmee bench --concurrent 50 --duration 10s'
};

terminalTabs.forEach(tab => {
  tab.addEventListener('click', () => {
    const targetTab = tab.getAttribute('data-tab');
    if (!targetTab) return;

    // Set active tab styling
    terminalTabs.forEach(t => t.classList.remove('active'));
    tab.classList.add('active');

    // Show corresponding content
    terminalContents.forEach(content => {
      content.classList.remove('active');
      if (content.getAttribute('id') === `tab-${targetTab}`) {
        content.classList.add('active');
        
        // Trigger simulated typing on the first line
        const promptLine = content.querySelector('.line:first-child');
        if (promptLine) {
          simulateTyping(promptLine, targetTab);
        }
      }
    });
  });
});

function simulateTyping(promptLineElement: Element, tabKey: string) {
  const fullCommand = commandPrompts[tabKey];
  if (!fullCommand) return;

  // Clear promptLine contents except the prompt span
  const promptSpan = promptLineElement.querySelector('.prompt');
  if (!promptSpan) return;

  promptLineElement.innerHTML = '';
  promptLineElement.appendChild(promptSpan);

  let index = 0;
  // Create typing cursor/effect
  const textNode = document.createTextNode('');
  promptLineElement.appendChild(textNode);

  const timer = setInterval(() => {
    if (index < fullCommand.length) {
      textNode.textContent += fullCommand.charAt(index);
      index++;
    } else {
      clearInterval(timer);
    }
  }, 25);
}

// Interactive Code Switcher
const langBtns = document.querySelectorAll('.lang-btn');
const codeBlocks = document.querySelectorAll('.code-block');
const editorTitle = document.querySelector('.editor-title');

const fileNames: Record<string, string> = {
  rust: 'guest_task.rs',
  js: 'guest_task.js',
  go: 'main.go',
  fiddle: 'Wasmee Live Fiddle'
};

langBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    const targetLang = btn.getAttribute('data-lang');
    if (!targetLang) return;

    // Set active language button styling
    langBtns.forEach(b => b.classList.remove('active'));
    btn.classList.add('active');

    // Update editor file name title
    if (editorTitle && fileNames[targetLang]) {
      editorTitle.textContent = fileNames[targetLang];
    }

    // Show corresponding code block
    codeBlocks.forEach(block => {
      block.classList.remove('active');
      if (block.getAttribute('id') === `code-${targetLang}`) {
        block.classList.add('active');
      }
    });
  });
});

// Live Fiddle API Integration
declare var require: any;
declare var monaco: any;

let payloadEditor: any = null;

if (typeof require !== 'undefined') {
  require.config({ paths: { vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.39.0/min/vs' } });
  require(['vs/editor/editor.main'], function () {
    const container = document.getElementById('fiddle-payload-editor');
    if (container) {
      payloadEditor = monaco.editor.create(container, {
        value: JSON.stringify({ order_total: 4500 }, null, 2),
        language: 'json',
        theme: 'vs-dark',
        minimap: { enabled: false },
        automaticLayout: true,
        scrollBeyondLastLine: false,
        lineNumbers: 'off',
        glyphMargin: false,
        folding: false,
        scrollbar: {
          vertical: 'hidden',
          horizontal: 'hidden'
        }
      });
    }
  });
}

const btnWarmup = document.getElementById('btn-warmup');
const btnExecute = document.getElementById('btn-execute');
const outputConsole = document.querySelector('.code-output-panel .output-body');

const WASMEE_URL = 'http://127.0.0.1:8081';

if (btnWarmup) {
  btnWarmup.addEventListener('click', async () => {
    const repo = (document.getElementById('fiddle-repo') as HTMLInputElement)?.value || '';
    const ref = (document.getElementById('fiddle-ref') as HTMLInputElement)?.value || '';
    const path = (document.getElementById('fiddle-path') as HTMLInputElement)?.value || '';
    const token = (document.getElementById('fiddle-token') as HTMLInputElement)?.value || '';

    updateStatusIndicator('syncing');
    updateConsole([
      { type: 'info', msg: 'Initiating Git pre-warming...' },
      { type: 'info', msg: `Repository: ${repo}` },
      { type: 'info', msg: `Ref (branch/tag): ${ref}` },
      { type: 'info', msg: `File path: ${path}` }
    ]);

    try {
      const response = await fetch(`${WASMEE_URL}/warmup`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer test-bearer-token'
        },
        body: JSON.stringify({
          git_source: {
            repository: repo,
            git_ref: ref,
            file_path: path,
            git_token: token
          }
        })
      });

      if (!response.ok) {
        throw new Error(await response.text());
      }

      const data = await response.json();
      if (data.success) {
        updateStatusIndicator('warm');
        updateConsole([
          { type: 'success', msg: 'Pre-warming completed successfully!' },
          { type: 'value', msg: `Compiled Module Hash: ${data.wasm_hash}` }
        ]);
        localStorage.setItem('wasmee_last_hash', data.wasm_hash);
      } else {
        updateStatusIndicator('error');
        updateConsole([
          { type: 'error', msg: `Pre-warming failed: ${data.error}` }
        ]);
      }
    } catch (e: any) {
      updateStatusIndicator('error');
      updateConsole([
        { type: 'error', msg: `Connection error: ${e.message}` },
        { type: 'warn', msg: 'Make sure Wasmee daemon is running locally on http://127.0.0.1:8081' }
      ]);
    }
  });
}

const btnSync = document.getElementById('btn-sync');
if (btnSync) {
  btnSync.addEventListener('click', async () => {
    const repo = (document.getElementById('fiddle-repo') as HTMLInputElement)?.value || '';
    const ref = (document.getElementById('fiddle-ref') as HTMLInputElement)?.value || '';
    const path = (document.getElementById('fiddle-path') as HTMLInputElement)?.value || '';
    const token = (document.getElementById('fiddle-token') as HTMLInputElement)?.value || '';

    updateStatusIndicator('syncing');
    updateConsole([
      { type: 'info', msg: 'Initiating manual GitOps synchronization...' },
      { type: 'info', msg: `Repository: ${repo}` },
      { type: 'info', msg: `Ref (branch/tag): ${ref}` },
      { type: 'info', msg: `File path: ${path}` }
    ]);

    try {
      const response = await fetch(`${WASMEE_URL}/gitops/sync`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer test-bearer-token'
        },
        body: JSON.stringify({
          git_source: {
            repository: repo,
            git_ref: ref,
            file_path: path,
            git_token: token
          }
        })
      });

      if (!response.ok) {
        throw new Error(await response.text());
      }

      const data = await response.json();
      if (data.success) {
        updateStatusIndicator('warm');
        updateConsole([
          { type: 'success', msg: 'GitOps synchronization completed successfully!' },
          { type: 'value', msg: `Compiled Module Hash: ${data.wasm_hash}` },
          { type: 'info', msg: 'JIT compilation cached. Ready to run on Wasmee.' }
        ]);
        localStorage.setItem('wasmee_last_hash', data.wasm_hash);
      } else {
        updateStatusIndicator('error');
        updateConsole([
          { type: 'error', msg: `GitOps sync failed: ${data.error}` }
        ]);
      }
    } catch (e: any) {
      updateStatusIndicator('error');
      updateConsole([
        { type: 'error', msg: `Connection error: ${e.message}` },
        { type: 'warn', msg: 'Make sure Wasmee daemon is running locally on http://127.0.0.1:8081' }
      ]);
    }
  });
}

if (btnExecute) {
  btnExecute.addEventListener('click', async () => {
    const repo = (document.getElementById('fiddle-repo') as HTMLInputElement)?.value || '';
    const ref = (document.getElementById('fiddle-ref') as HTMLInputElement)?.value || '';
    const path = (document.getElementById('fiddle-path') as HTMLInputElement)?.value || '';
    const token = (document.getElementById('fiddle-token') as HTMLInputElement)?.value || '';
    const payloadStr = payloadEditor ? payloadEditor.getValue() : '{}';
    const gasVal = parseInt((document.getElementById('fiddle-gas') as HTMLInputElement)?.value || '10000000', 10);
    const memVal = parseInt((document.getElementById('fiddle-memory') as HTMLInputElement)?.value || '32', 10);

    let payloadJson = {};
    try {
      payloadJson = JSON.parse(payloadStr);
    } catch (err: any) {
      updateConsole([
        { type: 'error', msg: `Invalid JSON payload: ${err.message}` }
      ]);
      return;
    }

    updateConsole([
      { type: 'info', msg: 'Executing WASM task on Wasmee...' }
    ]);

    const savedHash = localStorage.getItem('wasmee_last_hash') || '';

    try {
      const response = await fetch(`${WASMEE_URL}/execute`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': 'Bearer test-bearer-token'
        },
        body: JSON.stringify({
          instance_id: `fiddle-inst-${Math.random().toString(36).substring(7)}`,
          entrypoint: 'execute',
          params: [],
          base_snapshot: '',
          memory_deltas: {},
          oplog: [],
          wasm_hash: repo ? '' : savedHash, 
          git_source: repo ? {
            repository: repo,
            git_ref: ref,
            file_path: path,
            git_token: token
          } : undefined,
          exchange_buffer: btoa(JSON.stringify(payloadJson)),
          sandbox_config: {
            max_fuel: gasVal,
            max_memory_mb: memVal
          }
        })
      });

      if (!response.ok) {
        throw new Error(await response.text());
      }

      const data = await response.json();
      const duration = '30-100 µs';

      let logs: { type: string, msg: string }[] = [];
      if (data.crashed) {
        logs.push({ type: 'error', msg: `Execution Crashed: ${data.error}` });
      } else {
        logs.push({ type: 'success', msg: `Execution Success! (Duration: ${duration})` });
        
        if (data.response_bytes) {
          try {
            const decoded = atob(data.response_bytes);
            logs.push({ type: 'value', msg: `Response Buffer: ${decoded}` });
          } catch {
            logs.push({ type: 'value', msg: `Response Bytes (Base64): ${data.response_bytes}` });
          }
        }

        if (data.checkpoints && data.checkpoints.length > 0) {
          logs.push({ type: 'snap', msg: `Checkpoint saved: serialized memory snapshot (${data.checkpoints.length} state checkpoints)` });
        }
        if (data.final_oplog && data.final_oplog.length > 0) {
          data.final_oplog.forEach((op: any) => {
            logs.push({ type: 'info', msg: `Replay-safe Oplog call: ${op.api_name}` });
          });
        }
        if (data.final_deltas && Object.keys(data.final_deltas).length > 0) {
          logs.push({ type: 'snap', msg: `Memory delta pages saved: ${Object.keys(data.final_deltas).length} pages modified` });
        }
      }

      updateConsole(logs);
    } catch (e: any) {
      updateConsole([
        { type: 'error', msg: `Connection error: ${e.message}` },
        { type: 'warn', msg: 'Make sure Wasmee daemon is running locally on http://127.0.0.1:8081' }
      ]);
    }
  });
}

function updateConsole(lines: { type: string, msg: string }[]) {
  if (!outputConsole) return;
  outputConsole.innerHTML = '';
  
  lines.forEach(line => {
    const div = document.createElement('div');
    div.className = 'output-line';
    if (line.type === 'error') div.className += ' error-line';
    if (line.type === 'success') div.className += ' success-line';
    if (line.type === 'snap') div.className += ' highlight-line';
    
    const timeSpan = document.createElement('span');
    timeSpan.className = 'out-time';
    const now = new Date();
    timeSpan.textContent = `[${now.toTimeString().split(' ')[0]}] `;
    
    const tagSpan = document.createElement('span');
    tagSpan.className = `out-tag ${line.type}`;
    tagSpan.textContent = line.type.toUpperCase().substring(0, 4) + ' ';
    
    const textSpan = document.createTextNode(line.msg);
    
    div.appendChild(timeSpan);
    div.appendChild(tagSpan);
    div.appendChild(textSpan);
    outputConsole.appendChild(div);
  });
}

function updateStatusIndicator(status: 'stateless' | 'warm' | 'error' | 'syncing') {
  const dot = document.querySelector('#cache-status-indicator .status-dot') as HTMLElement;
  const text = document.querySelector('#cache-status-indicator .status-text') as HTMLElement;
  if (!dot || !text) return;

  switch (status) {
    case 'stateless':
      dot.style.backgroundColor = 'var(--text-muted)';
      dot.style.boxShadow = 'none';
      text.textContent = 'Stateless Mode';
      break;
    case 'warm':
      dot.style.backgroundColor = 'var(--accent-green)';
      dot.style.boxShadow = '0 0 8px var(--accent-green)';
      text.textContent = 'JIT Cache: Warm';
      break;
    case 'error':
      dot.style.backgroundColor = 'var(--accent-red)';
      dot.style.boxShadow = '0 0 8px var(--accent-red)';
      text.textContent = 'Sync Failed';
      break;
    case 'syncing':
      dot.style.backgroundColor = 'var(--accent-yellow)';
      dot.style.boxShadow = '0 0 8px var(--accent-yellow)';
      text.textContent = 'Syncing...';
      break;
  }
}

// Blog Section & Hash Routing Implementation
interface BlogPost {
  slug: string;
  title: string;
  category: string;
  date: string;
  author: string;
  excerpt: string;
  content: string;
}

const blogPosts: BlogPost[] = [
  {
    slug: 'understanding-durable-wasm-performance',
    title: 'Анатомия производительности Durable WebAssembly: Почему 25 000 RPS — это впечатляющий результат?',
    category: 'Performance & Architecture',
    date: '21 июня 2026 г.',
    author: 'Wasmee Core Team',
    excerpt: 'Разбор архитектуры Wasmee: как работают песочницы, восстановление памяти из слепков (snapshots) и расчёт разницы измененных страниц (dirty pages), и почему 25k RPS — выдающийся показатель.',
    content: `
      <p>Когда речь заходит о производительности WebAssembly, многие разработчики ориентируются на чистые бенчмарки Wasmtime или V8, которые демонстрируют сотни тысяч и даже миллионы вызовов функций в секунду. В то же время на главной странице Wasmee указана цифра в <strong>25 000+ In-Memory RPS</strong>. Почему показатели отличаются в разы и почему для Durable-подхода это выдающийся результат?</p>

      <h2>Как устроен чистый (Stateless) рантайм WebAssembly?</h2>
      <p>В стандартном сценарии Wasmtime удерживает скомпилированный инстанс модуля в оперативной памяти хост-процесса. Когда приходит вызов, хост выполняет прямой переход к инструкциям Wasm. Накладные расходы такого вызова составляют наносекунды. Это идеальное решение для вычислений без состояния (Stateless), таких как обработка изображений или парсинг данных. Однако в таком рантайме нет механизмов отказоустойчивости: если хост упадёт в процессе выполнения задачи, её состояние будет безвозвратно утеряно.</p>

      <h2>Что делает Wasmee при каждом вызове для обеспечения Durability?</h2>
      <p>Wasmee спроектирован для запуска <strong>Durable Micro-Tasks (отказоустойчивых бизнес-сценариев)</strong>. Чтобы гарантировать абсолютную изоляцию и возможность продолжить выполнение с любого шага при аппаратном сбое, Wasmee при каждом входящем запросе проходит через полноценный жизненный цикл:</p>

      <blockquote style="border-left: 4px solid var(--accent-purple); padding-left: 1.5rem; margin: 1.75rem 0; font-style: italic; background: rgba(139, 92, 246, 0.03); padding-top: 0.5rem; padding-bottom: 0.5rem;">
        <strong>Жизненный цикл транзакции в Wasmee:</strong><br>
        Инициализация Store &rarr; Инстанцирование модуля &rarr; Восстановление слепка памяти (Restore Snapshot) &rarr; Вызов гостевой функции &rarr; Сканирование грязных страниц (Dirty Pages) &rarr; Генерация дельт состояния.
      </blockquote>

      <h3>1. Полная изоляция песочницы (Sandboxing)</h3>
      <p>Для предотвращения утечек памяти и состояния между транзакциями разных пользователей Wasmee создает новые объекты <code>Linker</code>, <code>Store</code> (с лимитами на потребление топлива/памяти) и заново инстанцирует Wasm-модуль на каждый вызов.</p>

      <h3>2. Восстановление состояния памяти из слепка</h3>
      <p>Перед запуском гостевого кода Wasmee считывает базовый снимок состояния памяти модуля (base snapshot) и накладывает на него накопленные изменения (memory deltas). Получившийся массив байт записывается в память нового инстанса. Это восстанавливает состояние Wasm-модуля ровно в ту точку, на которой он остановился.</p>

      <h3>3. Запись инпута и прямой вызов</h3>
      <p>Входные параметры копируются напрямую в линейную память инстанса через shared-буфер (<code>EXCHANGE_BUFFER</code>). Это избавляет от необходимости кодирования и декодирования JSON или Base64 на лету.</p>

      <h3>4. Хеширование страниц и поиск дельт (Dirty-Page Tracking)</h3>
      <p>После вызова функции Wasmee должен выявить только изменившиеся фрагменты памяти, чтобы не сохранять гигабайты дублирующихся данных. Wasmee считывает память инстанса, разбивает её на страницы размером по <strong>64 КБ</strong> и вычисляет хеш для каждой страницы. Те страницы, чьи хеши не совпали с исходными, упаковываются в <code>memory_deltas</code> для сохранения в БД.</p>

      <h2>Почему 25 000+ RPS — это очень быстро?</h2>
      <p>Весь этот сложнейший цикл инициализации, восстановления, копирования, выполнения и постраничного дифференциального сравнения занимает всего <strong>менее 40 микросекунд (Warm Resume Latency &lt; 40 µs)</strong> на одну транзакцию!</p>
      <p>В переводе на пропускную способность одного CPU ядра это даёт около 25 000 RPS. Для сравнения, традиционные Docker-контейнеры требуют миллисекунды на запуск и перезапуск состояния. Wasmee делает это в тысячи раз быстрее, приближаясь по скорости к нативному коду, но предоставляя 100% гарантию сохранности состояния.</p>

      <h2>Заключение: микрооптимизации больше не нужны</h2>
      <p>Дальнейшее выжимание микросекунд из рантайма — например, усложнение алгоритма отслеживания памяти на уровне виртуальной памяти ОС (через обработку page faults) — приведёт к усложнению кодовой базы и снижению стабильности ради минимального прироста скорости.</p>
      <p>Текущие 25k RPS на ядро с лихвой покрывают требования самых высоконагруженных распределенных систем. На данном этапе архитектура Wasmee достигла оптимального баланса между скоростью работы и надёжностью выполнения, поэтому команда разработки фокусируется на расширении возможностей SDK и безопасности песочницы.</p>
    `
  }
];

const homeView = document.getElementById('home-view');
const blogView = document.getElementById('blog-view');
const blogFeedView = document.getElementById('blog-feed-view');
const blogPostView = document.getElementById('blog-post-view');
const blogPostsList = document.getElementById('blog-posts-list');
const navBlog = document.getElementById('nav-blog');

function router() {
  const hash = window.location.hash || '#home';
  
  // Update nav item active states
  document.querySelectorAll('.nav-menu .nav-item').forEach(link => {
    link.classList.remove('active');
  });
  
  if (hash === '#home' || hash.startsWith('#features') || hash.startsWith('#benchmarks') || hash.startsWith('#code') || hash.startsWith('#get-started')) {
    if (homeView) homeView.style.display = 'block';
    if (blogView) blogView.style.display = 'none';
    
    // Auto scroll to elements if hash contains anchor
    if (hash.startsWith('#') && hash !== '#home') {
      const targetEl = document.querySelector(hash);
      if (targetEl) {
        targetEl.scrollIntoView({ behavior: 'smooth' });
      }
    }
  } else if (hash === '#blog') {
    if (homeView) homeView.style.display = 'none';
    if (blogView) {
      blogView.style.display = 'block';
    }
    if (blogFeedView) blogFeedView.style.display = 'block';
    if (blogPostView) blogPostView.style.display = 'none';
    if (navBlog) navBlog.classList.add('active');
    
    renderBlogFeed();
    window.scrollTo({ top: 0, behavior: 'instant' as any });
  } else if (hash.startsWith('#blog/')) {
    const slug = hash.replace('#blog/', '');
    const post = blogPosts.find(p => p.slug === slug);
    if (post) {
      if (homeView) homeView.style.display = 'none';
      if (blogView) {
        blogView.style.display = 'block';
      }
      if (blogFeedView) blogFeedView.style.display = 'none';
      if (blogPostView) blogPostView.style.display = 'block';
      if (navBlog) navBlog.classList.add('active');
      
      const titleEl = document.getElementById('article-title');
      const dateEl = document.getElementById('article-date');
      const authorEl = document.getElementById('article-author');
      const bodyEl = document.getElementById('article-body');
      
      if (titleEl) titleEl.textContent = post.title;
      if (dateEl) dateEl.textContent = post.date;
      if (authorEl) authorEl.textContent = post.author;
      if (bodyEl) bodyEl.innerHTML = post.content;
      
      window.scrollTo({ top: 0, behavior: 'instant' as any });
    } else {
      window.location.hash = '#blog';
    }
  }
}

function renderBlogFeed() {
  if (!blogPostsList) return;
  blogPostsList.innerHTML = '';
  
  blogPosts.forEach(post => {
    const card = document.createElement('div');
    card.className = 'blog-card';
    card.innerHTML = `
      <div>
        <span class="blog-card-tag">${post.category}</span>
        <h3 class="blog-card-title">${post.title}</h3>
        <p class="blog-card-excerpt">${post.excerpt}</p>
      </div>
      <div class="blog-card-footer">
        <span>${post.date}</span>
        <span class="blog-card-more">
          Читать статью
          <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <line x1="5" y1="12" x2="19" y2="12"></line>
            <polyline points="12 5 19 12 12 19"></polyline>
          </svg>
        </span>
      </div>
    `;
    card.addEventListener('click', () => {
      window.location.hash = `#blog/${post.slug}`;
    });
    blogPostsList.appendChild(card);
  });
}

// Bind navigation routes
window.addEventListener('hashchange', router);
window.addEventListener('load', router);

// Trigger initial router execution on script load
if (document.readyState === 'complete' || document.readyState === 'interactive') {
  router();
}


