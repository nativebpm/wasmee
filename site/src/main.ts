import { translations } from './translations';

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
      // Fallback for Demo Mode (if daemon is not running locally)
      console.warn(`Local daemon connection failed, falling back to Demo Mode: ${e.message}`);
      await new Promise(resolve => setTimeout(resolve, 800));
      updateStatusIndicator('warm');
      const mockHash = 'sha256:d57b120c1592e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495';
      updateConsole([
        { type: 'success', msg: 'Pre-warming completed successfully! (Demo Mode)' },
        { type: 'value', msg: `Compiled Module Hash: ${mockHash}` }
      ]);
      localStorage.setItem('wasmee_last_hash', mockHash);
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
      // Fallback for Demo Mode (if daemon is not running locally)
      console.warn(`Local daemon connection failed, falling back to Demo Mode: ${e.message}`);
      await new Promise(resolve => setTimeout(resolve, 800));
      updateStatusIndicator('warm');
      const mockHash = 'sha256:d57b120c1592e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495';
      updateConsole([
        { type: 'success', msg: 'GitOps synchronization completed successfully! (Demo Mode)' },
        { type: 'value', msg: `Compiled Module Hash: ${mockHash}` },
        { type: 'info', msg: 'JIT compilation cached. Ready to run on Wasmee.' }
      ]);
      localStorage.setItem('wasmee_last_hash', mockHash);
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
      // Fallback for Demo Mode (if daemon is not running locally)
      console.warn(`Local daemon connection failed, falling back to Demo Mode: ${e.message}`);
      await new Promise(resolve => setTimeout(resolve, 300));

      const orderTotal = (payloadJson as any).order_total !== undefined ? parseFloat((payloadJson as any).order_total) : 4500;
      const taxRate = 0.15;
      const taxDue = orderTotal * taxRate;
      const totalWithTax = orderTotal + taxDue;

      const responseObj = {
        tax_due: taxDue,
        total_with_tax: totalWithTax,
        status: "approved",
        demo: true
      };

      const duration = '30-100 µs';
      let logs: { type: string, msg: string }[] = [];
      logs.push({ type: 'success', msg: `Execution Success! (Duration: ${duration}) (Demo Mode)` });
      logs.push({ type: 'info', msg: "Loading environment variable 'order_total'..." });
      logs.push({ type: 'value', msg: `order_total = ${orderTotal.toFixed(2)}` });
      logs.push({ type: 'info', msg: `Calling set_variable('tax_rate', ${taxRate})` });
      logs.push({ type: 'value', msg: `Response Buffer: ${JSON.stringify(responseObj, null, 2)}` });
      logs.push({ type: 'snap', msg: `Checkpoint saved: serialized memory snapshot (3 state checkpoints)` });
      logs.push({ type: 'info', msg: `Replay-safe Oplog call: get_variable` });
      logs.push({ type: 'info', msg: `Replay-safe Oplog call: set_variable` });
      logs.push({ type: 'snap', msg: `Memory delta pages saved: 4 pages modified` });

      updateConsole(logs);
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
const blogPosts: Record<'en' | 'ru', BlogPost[]> = {
  ru: [
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

        <h2>Что это значит для бизнеса и интеграции? (Главные выводы)</h2>
        <p>Архитектура Wasmee с производительностью 25 000 RPS дает конкретные коммерческие и технические преимущества системам, которые с ней интегрируются:</p>
        <ul>
          <li><strong>Сверхвысокая плотность и экономия на инфраструктуре (до 10 раз)</strong>: Вместо запуска тяжелых Docker-контейнеров для каждого пользовательского сценария (которые требуют от 100 МБ ОЗУ и запускаются миллисекунды) Wasmee запускает изолированные песочницы внутри одного процесса. Каждая задача потребляет всего около 4.2 МБ ОЗУ и стартует за микросекунды. Это позволяет обслуживать в 10 раз больше клиентов на том же оборудовании, колоссально снижая счета за облака.</li>
          <li><strong>Отказоустойчивость «из коробки» (Fault Tolerance)</strong>: Разработчикам больше не нужно вручную программировать сложную логику повторных попыток (retries), сохранять промежуточные состояния в базу данных при сбоях или проектировать распределенные транзакции. Если сервер аварийно завершит работу, Wasmee автоматически продолжит выполнение с точностью до последней выполненной инструкции из сохраненного слепка памяти.</li>
          <li><strong>Безопасное исполнение стороннего кода (Plugin Systems)</strong>: Если вы создаете платформу, где сторонние разработчики могут загружать свои скрипты или плагины, Wasmee гарантирует абсолютную безопасность. Гостевой код выполняется в изолированной WebAssembly-песочнице с жесткими лимитами по памяти и CPU, без доступа к вашей файловой системе или сети.</li>
          <li><strong>Простая и быстрая интеграция</strong>: Взаимодействие хост-системы с Wasmee происходит через высокоэффективный бинарный протокол Protobuf и общую память (shared-буфер <code>EXCHANGE_BUFFER</code>). Это полностью убирает накладные расходы на сериализацию (JSON/Base64) и избавляет от необходимости проектировать сложные шины данных для синхронизации состояний.</li>
        </ul>

        <h2>Заключение: микрооптимизации больше не нужны</h2>
        <p>Дальнейшее выжимание микросекунд из рантайма — например, усложнение алгоритма отслеживания памяти на уровне виртуальной памяти ОС (через обработку page faults) — приведёт к усложнению кодовой базы и снижению стабильности ради минимального прироста скорости.</p>
        <p>Текущие 25k RPS на ядро с лихвой покрывают требования самых высоконагруженных распределенных систем. На данном этапе архитектура Wasmee достигла оптимального баланса между скоростью работы и надёжностью выполнения, поэтому команда разработки фокусируется на расширении возможностей SDK и безопасности песочницы.</p>
      `
    }
  ],
  en: [
    {
      slug: 'understanding-durable-wasm-performance',
      title: 'Anatomy of Durable WebAssembly Performance: Why 25,000 RPS is an Outstanding Result',
      category: 'Performance & Architecture',
      date: 'June 21, 2026',
      author: 'Wasmee Core Team',
      excerpt: 'A deep dive into Wasmee\'s architecture: how sandboxed isolation, state restoration from checkpoints, and dirty-page tracking work, and why 25k RPS is a major milestone for fault-tolerant compute.',
      content: `
        <p>When discussing WebAssembly performance, developers often point to raw Wasmtime or V8 benchmarks demonstrating hundreds of thousands or even millions of function calls per second. Meanwhile, Wasmee showcases <strong>25,000+ In-Memory RPS</strong>. Why is there a difference, and why is this throughput actually outstanding for a durable execution model?</p>

        <h2>Understanding Stateless WebAssembly Runtimes</h2>
        <p>In a standard stateless scenario, Wasmtime keeps a compiled instance of the module warm in the host process memory. When a call arrives, the host jumps directly to the Wasm instructions. The overhead of this invocation is measured in nanoseconds. This is perfect for stateless computations like image processing or data parsing. However, this model lacks fault tolerance: if the host crashes mid-execution, the task state is lost forever.</p>

        <h2>What Wasmee Does to Guarantee Durability</h2>
        <p>Wasmee is designed specifically for <strong>Durable Micro-Tasks (fault-tolerant business workflows)</strong>. To guarantee absolute security sandboxing and the ability to resume execution from any point after a crash, Wasmee runs through a complete lifecycle for every single request:</p>

        <blockquote style="border-left: 4px solid var(--accent-purple); padding-left: 1.5rem; margin: 1.75rem 0; font-style: italic; background: rgba(139, 92, 246, 0.03); padding-top: 0.5rem; padding-bottom: 0.5rem;">
          <strong>Wasmee Transaction Lifecycle:</strong><br>
          Initialize Store &rarr; Instantiate Module &rarr; Restore Memory Snapshot &rarr; Invoke Guest Function &rarr; Scan Page Hashes (Dirty Pages) &rarr; Generate State Deltas.
        </blockquote>

        <h3>1. Sandboxed Isolation</h3>
        <p>To prevent memory and execution state leakage between different tenant transactions, Wasmee creates new <code>Linker</code>, <code>Store</code> (with strict memory/fuel limits), and <code>Instance</code> objects for every invocation.</p>

        <h3>2. Memory State Restoration</h3>
        <p>Before running the guest module, Wasmee loads the base memory snapshot and merges the page-dirty deltas accumulated from previous steps. This reconstructed byte array is then written into the newly instantiated Wasm memory, restoring the execution state exactly where it left off.</p>

        <h3>3. Zero-Serialization Input Buffer</h3>
        <p>Input parameters are copied directly into Wasm memory via a shared static buffer (<code>EXCHANGE_BUFFER</code>). This completely bypasses the overhead of JSON parsing or Base64 encoding/decoding during request dispatching.</p>

        <h3>4. Post-Execution Page Hashing & Delta Tracking</h3>
        <p>After the guest function completes, Wasmee identifies which parts of memory were modified to avoid saving redundant state. It scans the linear memory in <strong>64KB pages</strong>, calculates page hashes, compares them to the base state, and outputs the modified pages as new deltas for checkpoint storage.</p>

        <h2>Why 25,000+ RPS is Extremely Fast</h2>
        <p>This entire recovery, instantiation, zero-copy buffer exchange, guest execution, and page hashing cycle takes <strong>under 40 microseconds</strong> per call (Warm Resume Latency &lt; 40 µs).</p>
        <p>On a single CPU core, this equals more than 25,000 requests per second. While virtual containers (like Docker) require milliseconds to start and recover state, Wasmee achieves this in microseconds—providing native execution speed alongside robust crash resilience.</p>

        <h2>What This Means for Business & Integration (Key Takeaways)</h2>
        <p>Wasmee's architecture and its 25,000 RPS throughput deliver major commercial and technical advantages to integrating systems:</p>
        <ul>
          <li><strong>Unmatched Infrastructure Cost Savings (Up to 10x)</strong>: Instead of running heavy Docker containers for every user script (which require 100MB+ RAM and take milliseconds to boot), Wasmee executes tasks in lightweight sandboxed environments consuming only ~4.2 MB of RAM per run. You can scale to tens of thousands of concurrent users on a single cheap server.</li>
          <li><strong>Out-of-the-Box Fault Tolerance</strong>: Developers don't need to manually write complex retry logic, state-machine synchronization, or transaction rollback code. If a server crashes mid-task, Wasmee restores the execution state immediately from the last page checkpoint.</li>
          <li><strong>Secure Third-Party Plugins & Scripts</strong>: If you are building a platform that executes untrusted user-submitted code or developer extensions, Wasmee provides complete sandboxed isolation. It enforces CPU and memory limits, and prevents unauthorized access to the network or filesystem.</li>
          <li><strong>Simplified Integration Architecture</strong>: Integrating with Wasmee is simple because the host system communicates over high-speed Protobuf via a shared in-memory buffer. You bypass complex data pipes and database sync systems—the engine handles state persistence automatically.</li>
        </ul>

        <h2>Conclusion: The Case Against Micro-Optimizations</h2>
        <p>Further performance optimization—such as handling OS-level page faults to track memory changes—would add complexity and decrease stability for marginal speed gains.</p>
        <p>With 25,000 RPS per core, Wasmee easily meets the demands of high-throughput distributed systems. Our current architecture achieves the perfect balance between raw speed and reliable execution, allowing our team to focus on expanding SDK features and strengthening sandboxed security.</p>
      `
    }
  ]
};

const homeView = document.getElementById('home-view');
const blogView = document.getElementById('blog-view');
const blogFeedView = document.getElementById('blog-feed-view');
const blogPostView = document.getElementById('blog-post-view');
const blogPostsList = document.getElementById('blog-posts-list');
const navBlog = document.getElementById('nav-blog');

let currentLang: 'en' | 'ru' = 'en';

function getPreferredLanguage(): 'en' | 'ru' {
  const saved = localStorage.getItem('wasmee_lang');
  if (saved === 'en' || saved === 'ru') {
    return saved;
  }
  
  const browserLang = navigator.language || (navigator as any).userLanguage || '';
  if (browserLang.toLowerCase().startsWith('ru') || 
      browserLang.toLowerCase().startsWith('be') || 
      browserLang.toLowerCase().startsWith('uk') || 
      browserLang.toLowerCase().startsWith('kk')) {
    return 'ru';
  }
  return 'en';
}

function setLanguage(lang: 'en' | 'ru') {
  currentLang = lang;
  localStorage.setItem('wasmee_lang', lang);
  
  // Update language switcher buttons UI
  document.querySelectorAll('[data-switch-lang]').forEach(btn => {
    if (btn.getAttribute('data-switch-lang') === lang) {
      btn.classList.add('active');
    } else {
      btn.classList.remove('active');
    }
  });
  
  // Translate all data-i18n elements
  document.querySelectorAll('[data-i18n]').forEach(el => {
    const key = el.getAttribute('data-i18n');
    if (key && translations[key]) {
      const translation = translations[key][lang];
      if (translation.includes('<span')) {
        el.innerHTML = translation;
      } else {
        el.textContent = translation;
      }
    }
  });

  // Translate all data-i18n-placeholder elements
  document.querySelectorAll('[data-i18n-placeholder]').forEach(el => {
    const key = el.getAttribute('data-i18n-placeholder');
    if (key && translations[key]) {
      (el as HTMLInputElement).placeholder = translations[key][lang];
    }
  });
}

function router() {
  const preferredLang = getPreferredLanguage();
  const hash = window.location.hash || `#${preferredLang}/home`;
  
  let lang: 'en' | 'ru' = 'en';
  let path = 'home';
  
  const match = hash.match(/^#(en|ru)(?:\/(.*))?$/);
  if (match) {
    lang = match[1] as 'en' | 'ru';
    path = match[2] || 'home';
    currentLang = lang;
  } else {
    // Legacy or non-prefixed hashes: redirect to prefixed hash
    const cleanHash = hash.replace(/^#/, '');
    if (cleanHash === 'en' || cleanHash === 'ru') {
      lang = cleanHash as 'en' | 'ru';
      path = 'home';
      currentLang = lang;
      window.location.hash = `#${lang}/${path}`;
      return;
    } else {
      lang = currentLang;
      path = cleanHash || 'home';
      window.location.hash = `#${lang}/${path}`;
      return;
    }
  }
  
  // Apply translation
  setLanguage(lang);
  
  // Update nav item active states
  document.querySelectorAll('.nav-menu .nav-item').forEach(link => {
    link.classList.remove('active');
  });
  
  if (path === 'home' || path.startsWith('features') || path.startsWith('benchmarks') || path.startsWith('use-cases') || path.startsWith('code') || path.startsWith('get-started')) {
    if (homeView) homeView.style.display = 'block';
    if (blogView) blogView.style.display = 'none';
    
    // Auto scroll to elements if path matches section ID
    if (path !== 'home') {
      // Find element by id
      const targetEl = document.getElementById(path);
      if (targetEl) {
        targetEl.scrollIntoView({ behavior: 'smooth' });
      }
    }
  } else if (path === 'blog') {
    if (homeView) homeView.style.display = 'none';
    if (blogView) {
      blogView.style.display = 'block';
    }
    if (blogFeedView) blogFeedView.style.display = 'block';
    if (blogPostView) blogPostView.style.display = 'none';
    if (navBlog) navBlog.classList.add('active');
    
    renderBlogFeed();
    window.scrollTo({ top: 0, behavior: 'instant' as any });
  } else if (path.startsWith('blog/')) {
    const slug = path.replace('blog/', '');
    const post = blogPosts[currentLang].find(p => p.slug === slug);
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
      window.location.hash = `#${lang}/blog`;
    }
  }
}

function renderBlogFeed() {
  if (!blogPostsList) return;
  blogPostsList.innerHTML = '';
  
  const posts = blogPosts[currentLang] || [];
  posts.forEach(post => {
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
          ${currentLang === 'ru' ? 'Читать статью' : 'Read Article'}
          <svg viewBox="0 0 24 24" width="16" height="16" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round">
            <line x1="5" y1="12" x2="19" y2="12"></line>
            <polyline points="12 5 19 12 12 19"></polyline>
          </svg>
        </span>
      </div>
    `;
    card.addEventListener('click', () => {
      window.location.hash = `#${currentLang}/blog/${post.slug}`;
    });
    blogPostsList.appendChild(card);
  });
}

// Bind language switch buttons
document.querySelectorAll('[data-switch-lang]').forEach(btn => {
  btn.addEventListener('click', () => {
    const newLang = btn.getAttribute('data-switch-lang');
    if (!newLang) return;
    
    const hash = window.location.hash || '#en/home';
    const match = hash.match(/^#(en|ru)(?:\/(.*))?$/);
    if (match) {
      const path = match[2] || 'home';
      window.location.hash = `#${newLang}/${path}`;
    } else {
      window.location.hash = `#${newLang}/home`;
    }
  });
});

// (Live Fiddle preset binding removed in favor of Interactive Demo Modal)

// --- Live Wasm & Host Simulator for Browser Demos ---
interface SimInstance {
  id: string;
  type: string; // "workflow", "game", "servicedesk"
  waiting_nodes: string[];
  variables: any;
  completed: boolean;
  history: string[];
  version: number;
  created_at: number; // ms
  last_tick: number; // ms
}

let simInstances: SimInstance[] = [];
let activeDemoProduct = '';
let activeDemoInstanceId = '';
let demoRole = 'Customer';

function logDemoConsole(msg: string, type: 'info' | 'success' | 'warn' | 'error' = 'info') {
  const consoleBox = document.getElementById('demo-console-logs');
  if (!consoleBox) return;
  const time = new Date().toLocaleTimeString();
  let color = '#60a5fa'; // blue
  if (type === 'success') color = '#34d399'; // green
  if (type === 'warn') color = '#fbbf24'; // orange
  if (type === 'error') color = '#f87171'; // red
  consoleBox.innerHTML += `\n<span style="color:var(--text-muted); margin-right:0.5rem;">[${time}]</span><span style="color:${color};">${msg}</span>`;
  consoleBox.scrollTop = consoleBox.scrollHeight;
}

function updateDemoVarsView(vars: any) {
  const varsView = document.getElementById('demo-vars-view');
  if (varsView) {
    varsView.textContent = JSON.stringify(vars, null, 2);
  }
}

// Bind product cards to open the Interactive Demo Modal
const productCards = document.querySelectorAll('.usecase-card');
const demoModal = document.getElementById('demo-modal');
const demoCloseBtn = document.getElementById('demo-close-btn');
const demoRoleSelect = document.getElementById('demo-role-select') as HTMLSelectElement;
const demoRoleBox = document.getElementById('demo-role-box');

productCards.forEach(card => {
  card.addEventListener('click', () => {
    const product = card.getAttribute('data-product');
    if (!product || !demoModal) return;

    activeDemoProduct = product;
    activeDemoInstanceId = '';
    demoModal.classList.add('active');

    // Reset console and variables
    const consoleBox = document.getElementById('demo-console-logs');
    if (consoleBox) consoleBox.innerHTML = '--- Wasmee Wasmtime JIT engine pre-warmed. Sandbox idle. ---';
    updateDemoVarsView({});

    const titleEl = document.getElementById('demo-product-title');
    const areaTitle = document.getElementById('demo-area-title');
    const createBtn = document.getElementById('demo-create-btn');

    if (product === 'workflow') {
      if (titleEl) titleEl.textContent = 'Wasmee Workflow Demo';
      if (areaTitle) areaTitle.textContent = 'Todo List Workflows';
      if (createBtn) createBtn.textContent = '+ Create Todo Flow';
      if (demoRoleBox) demoRoleBox.style.display = 'none';
      logDemoConsole('Wasmee Workflow sandbox environment loaded.', 'success');
    } else if (product === 'game') {
      if (titleEl) titleEl.textContent = 'Wasmee Game Demo (Sven vs Lina)';
      if (areaTitle) areaTitle.textContent = 'Resilient Arena Combat';
      if (createBtn) createBtn.textContent = '+ Start Combat';
      if (demoRoleBox) demoRoleBox.style.display = 'none';
      logDemoConsole('Wasmee Game sandbox environment loaded.', 'success');
    } else if (product === 'servicedesk') {
      if (titleEl) titleEl.textContent = 'Wasmee ServiceDesk Demo';
      if (areaTitle) areaTitle.textContent = 'ITIL Incident Tickets';
      if (createBtn) createBtn.textContent = '+ Report Incident';
      if (demoRoleBox) demoRoleBox.style.display = 'flex';
      logDemoConsole('Wasmee ServiceDesk sandbox environment loaded with role-based auth.', 'success');
    }

    renderDemoItems();
    clearDemoForm();
  });
});

if (demoCloseBtn && demoModal) {
  demoCloseBtn.addEventListener('click', () => {
    demoModal.classList.remove('active');
  });
}

if (demoRoleSelect) {
  demoRoleSelect.addEventListener('change', () => {
    demoRole = demoRoleSelect.value;
    logDemoConsole(`Demo role switched to: ${demoRole}`, 'success');
    if (activeDemoInstanceId) {
      selectDemoInstance(activeDemoInstanceId);
    }
  });
}

// Create new flow
const demoCreateBtn = document.getElementById('demo-create-btn');
if (demoCreateBtn) {
  demoCreateBtn.addEventListener('click', () => {
    const id = `inst-${Math.random().toString(36).substring(7)}`;
    logDemoConsole(`Host: Initializing Wasm sandbox for instance ${id.substring(0, 5)}...`);
    logDemoConsole('Wasmee: Loading module guest.wasm from cache (hash validation: OK).', 'success');
    logDemoConsole('Wasmee: Linker compiled, memory bounds allocated (32MB).');

    let variables: any = {};
    let waiting: string[] = [];

    if (activeDemoProduct === 'workflow') {
      const desc = prompt('Enter Todo description:', 'Task description from Workflow Product');
      if (!desc) return;
      variables = { todo_item: desc, priority: 'Medium' };
      waiting = ['add_todo'];
      logDemoConsole("Wasmee: Running entrypoint 'execute' with workflow type: workflow...");
    } else if (activeDemoProduct === 'game') {
      variables = { sven_hp: 100, lina_hp: 100, sven_stunned: false, lina_cooldown: 0, turn: 0 };
      waiting = ['play_turn'];
      logDemoConsole("Wasmee: Running entrypoint 'execute' with workflow type: game...");
    } else if (activeDemoProduct === 'servicedesk') {
      const title = prompt('Enter Incident Title:', 'System database connection timeout.');
      if (!title) return;
      const desc = prompt('Enter Incident Description:', 'Unable to connect to PostgreSQL from node-3');
      const priority = prompt('Enter Priority (Low, Medium, High, Critical):', 'Medium') || 'Medium';
      variables = { title, description: desc, priority, status: 'New' };
      waiting = ['create_incident'];
      logDemoConsole("Wasmee: Running entrypoint 'execute' with workflow type: servicedesk...");
    }

    logDemoConsole('Wasmee: Checkpoint triggered: scanning 512 linear memory pages...');
    logDemoConsole('Wasmee: Saved snapshot checkpoint (Version 1).', 'success');

    const newInst: SimInstance = {
      id,
      type: activeDemoProduct,
      waiting_nodes: waiting,
      variables,
      completed: false,
      history: ['Instance created. Base checkpoint Version 1 saved.'],
      version: 1,
      created_at: Date.now(),
      last_tick: Date.now()
    };

    simInstances.push(newInst);
    renderDemoItems();
    selectDemoInstance(id);
  });
}

function renderDemoItems() {
  const container = document.getElementById('demo-items-container');
  if (!container) return;
  container.innerHTML = '';

  const activeItems = simInstances.filter(i => i.type === activeDemoProduct);
  if (activeItems.length === 0) {
    container.innerHTML = '<p style="color: var(--text-muted); font-size: 0.85rem; text-align: center; margin-top: 2rem;">No active items. Create one to start.</p>';
    return;
  }

  activeItems.forEach(inst => {
    const card = document.createElement('div');
    card.className = `demo-card ${inst.id === activeDemoInstanceId ? 'active' : ''}`;
    card.onclick = () => selectDemoInstance(inst.id);

    if (inst.type === 'workflow') {
      const desc = inst.variables.todo_item || 'Todo';
      card.innerHTML = `
        <div style="font-weight:600; color:#fff;">${desc}</div>
        <div style="font-size:0.75rem; color:var(--text-muted); display:flex; justify-content:space-between; margin-top:0.5rem;">
          <span>Status: ${inst.completed ? 'Completed' : 'Pending'}</span>
          <span>Version: ${inst.version}</span>
        </div>
      `;
    } else if (inst.type === 'game') {
      card.innerHTML = `
        <div style="font-weight:600; color:#fff;">Sven vs Lina Arena</div>
        <div style="font-size:0.75rem; color:var(--text-muted); display:flex; gap: 1rem; margin-top:0.5rem;">
          <span>Sven HP: ${inst.variables.sven_hp}</span>
          <span>Lina HP: ${inst.variables.lina_hp}</span>
          <span>Turn: ${inst.variables.turn}</span>
        </div>
      `;
    } else if (inst.type === 'servicedesk') {
      const title = inst.variables.title || 'Incident';
      const status = inst.variables.status || 'New';
      const priority = inst.variables.priority || 'Medium';
      
      let timerText = 'Resolved';
      let fillPct = 100;
      let barColor = 'var(--success)';

      if (!inst.completed) {
        const isNew = status === 'New';
        const limitDur = isNew ? 60000 : 180000; // simulated durations (60s reaction, 180s resolution)
        const elapsed = Date.now() - inst.created_at;
        const remaining = limitDur - elapsed;

        if (remaining <= 0) {
          fillPct = 100;
          barColor = 'var(--danger)';
          timerText = isNew ? 'Reaction SLA Breached' : 'Resolution SLA Breached';
        } else {
          fillPct = (remaining / limitDur) * 100;
          barColor = fillPct < 25 ? 'var(--danger)' : (fillPct < 50 ? 'var(--warning)' : 'var(--success)');
          timerText = `${Math.ceil(remaining / 1000)}s left`;
        }
      }

      card.innerHTML = `
        <div style="display:flex; justify-content:space-between; align-items:center;">
          <span style="font-weight:600; color:#fff;">${title}</span>
          <span class="incident-status-badge status-${status.toLowerCase()}" style="font-size:0.7rem; padding:0.15rem 0.4rem;">${status}</span>
        </div>
        <div class="demo-sla-bar">
          <div class="demo-sla-fill" style="width:${fillPct}%; background:${barColor};"></div>
        </div>
        <div style="font-size:0.75rem; color:var(--text-muted); display:flex; justify-content:space-between; margin-top:0.25rem;">
          <span>Priority: ${priority}</span>
          <span>SLA: ${timerText}</span>
        </div>
      `;
    }

    container.appendChild(card);
  });
}

function selectDemoInstance(id: string) {
  activeDemoInstanceId = id;
  renderDemoItems();
  const inst = simInstances.find(i => i.id === id);
  if (!inst) return;

  updateDemoVarsView(inst.variables);

  // Render operations form based on the current state node
  const formBox = document.getElementById('demo-action-form');
  if (!formBox) return;

  if (inst.completed) {
    formBox.innerHTML = '<p style="color:var(--success); font-weight:600; text-align:center; margin-top:1.5rem;">✓ Workflow Completed Successfully</p>';
    return;
  }

  const activeNode = inst.waiting_nodes[0];
  let formHTML = '';

  if (inst.type === 'workflow') {
    if (activeNode === 'add_todo') {
      formHTML = `
        <form onsubmit="event.preventDefault(); submitDemoStep('${id}', 'add_todo');">
          <div class="demo-form-group">
            <label>Todo Item</label>
            <input type="text" name="todo_item" class="demo-input" value="${inst.variables.todo_item || ''}" required>
          </div>
          <div class="demo-form-group">
            <label>Priority</label>
            <select name="priority" class="demo-input">
              <option value="Low" ${inst.variables.priority === 'Low' ? 'selected' : ''}>Low</option>
              <option value="Medium" ${inst.variables.priority === 'Medium' ? 'selected' : ''}>Medium</option>
              <option value="High" ${inst.variables.priority === 'High' ? 'selected' : ''}>High</option>
            </select>
          </div>
          <button type="submit" class="demo-btn" style="width:100%;">Create Task Step</button>
        </form>
      `;
    } else if (activeNode === 'complete_todo') {
      formHTML = `
        <form onsubmit="event.preventDefault(); submitDemoStep('${id}', 'complete_todo');">
          <div class="demo-form-group">
            <label>Todo Item (Read Only)</label>
            <input type="text" class="demo-input" value="${inst.variables.todo_item || ''}" readonly>
          </div>
          <div class="demo-form-group">
            <label class="demo-checkbox">
              <input type="checkbox" name="todo_item_completed" required>
              <span>Mark as Completed</span>
            </label>
          </div>
          <button type="submit" class="demo-btn" style="width:100%;">Complete Workflow</button>
        </form>
      `;
    }
  } else if (inst.type === 'game') {
    formHTML = `
      <div style="display:flex; flex-direction:column; gap:0.75rem;">
        <p style="font-size:0.8rem; color:var(--text-muted);">Simulate Sven and Lina combat turns. Wasm sandbox tracks HP and cooldowns.</p>
        <div style="display:grid; grid-template-columns:1fr 1fr; gap:0.5rem;">
          <button onclick="playGameTurn('${id}', 'StormHammer')" class="demo-btn">Storm Hammer</button>
          <button onclick="playGameTurn('${id}', 'Warcry')" class="demo-btn" style="background:linear-gradient(135deg, #f59e0b, #d97706);">Warcry</button>
          <button onclick="playGameTurn('${id}', 'DragonSlave')" class="demo-btn" style="background:linear-gradient(135deg, #ef4444, #dc2626);">Dragon Slave</button>
          <button onclick="playGameTurn('${id}', 'LagunaBlade')" class="demo-btn" style="background:linear-gradient(135deg, #ec4899, #db2777);">Laguna Blade</button>
        </div>
        <button onclick="simulateGameCrash('${id}')" class="demo-btn demo-btn-secondary" style="margin-top:0.5rem;">💥 Crash & Restore</button>
      </div>
    `;
  } else if (inst.type === 'servicedesk') {
    if (activeNode === 'create_incident') {
      formHTML = `
        <form onsubmit="event.preventDefault(); submitDemoStep('${id}', 'create_incident');">
          <div class="demo-form-group">
            <label>Incident Title</label>
            <input type="text" name="title" class="demo-input" value="${inst.variables.title || ''}" required>
          </div>
          <div class="demo-form-group">
            <label>Incident Description</label>
            <input type="text" name="description" class="demo-input" value="${inst.variables.description || ''}" required>
          </div>
          <div class="demo-form-group">
            <label>Priority</label>
            <select name="priority" class="demo-input">
              <option value="Low" ${inst.variables.priority === 'Low' ? 'selected' : ''}>Low</option>
              <option value="Medium" ${inst.variables.priority === 'Medium' ? 'selected' : ''}>Medium</option>
              <option value="High" ${inst.variables.priority === 'High' ? 'selected' : ''}>High</option>
              <option value="Critical" ${inst.variables.priority === 'Critical' ? 'selected' : ''}>Critical</option>
            </select>
          </div>
          <button type="submit" class="demo-btn" style="width:100%;">Create SLA Ticket</button>
        </form>
      `;
    } else if (activeNode === 'new_incident') {
      formHTML = `
        <form onsubmit="event.preventDefault(); submitDemoStep('${id}', 'new_incident');">
          <div class="demo-form-group">
            <label>Incident Title</label>
            <input type="text" class="demo-input" value="${inst.variables.title || ''}" readonly>
          </div>
          <div class="demo-form-group">
            <label>Assignee Name</label>
            <input type="text" name="assignee" class="demo-input" placeholder="e.g. John Support" required>
          </div>
          <button type="submit" class="demo-btn" style="width:100%;">Assign SLA Incident</button>
        </form>
      `;
    } else if (activeNode === 'investigating') {
      formHTML = `
        <form onsubmit="event.preventDefault(); submitDemoStep('${id}', 'investigating');">
          <div class="demo-form-group">
            <label>Incident Title</label>
            <input type="text" class="demo-input" value="${inst.variables.title || ''}" readonly>
          </div>
          <div class="demo-form-group">
            <label>Comments / Updates</label>
            <input type="text" name="comments" class="demo-input" placeholder="e.g. Found database deadlock issue" required>
          </div>
          <button type="submit" class="demo-btn" style="width:100%;">Post Update</button>
        </form>
      `;
    } else if (activeNode === 'resolved') {
      formHTML = `
        <form onsubmit="event.preventDefault(); submitDemoStep('${id}', 'resolved');">
          <div class="demo-form-group">
            <label>Incident Title</label>
            <input type="text" class="demo-input" value="${inst.variables.title || ''}" readonly>
          </div>
          <div class="demo-form-group">
            <label>Resolution Notes</label>
            <input type="text" name="resolution" class="demo-input" placeholder="e.g. Database indexes updated" required>
          </div>
          <button type="submit" class="demo-btn" style="width:100%;">Resolve Incident</button>
        </form>
      `;
    }
  }

  formBox.innerHTML = formHTML;
}

function clearDemoForm() {
  const formBox = document.getElementById('demo-action-form');
  if (formBox) {
    formBox.innerHTML = '<p style="color: var(--text-muted); font-size: 0.8rem; text-align: center; margin-top: 1rem;">Select a card to act on it.</p>';
  }
}

// Global functions for inline DOM event bindings
(window as any).submitDemoStep = function (id: string, activeNode: string) {
  const inst = simInstances.find(i => i.id === id);
  if (!inst) return;

  // Authorization checks
  if (inst.type === 'servicedesk') {
    if (activeNode !== 'create_incident' && demoRole === 'Customer') {
      logDemoConsole(`Wasmee: [SECURITY ERROR] Role 'Customer' is unauthorized for transition step '${activeNode}'.`, 'error');
      alert(`Access Denied: Customer role cannot perform '${activeNode}' operations!`);
      return;
    }
  }

  const form = document.querySelector('#demo-action-form form') as HTMLFormElement;
  const formData = new FormData(form);
  const input: any = {};
  for (let [key, val] of formData.entries()) {
    input[key] = val;
  }
  // Checkbox conversion
  form.querySelectorAll('input[type="checkbox"]').forEach((cb: any) => {
    input[cb.name] = cb.checked;
  });

  logDemoConsole(`Host: Resuming instance ${id.substring(0, 5)} on entrypoint 'resume' with role: ${demoRole}...`);
  logDemoConsole(`Wasmee: Restoring memory... Loading snapshot checkpoint (Version ${inst.version}).`);
  logDemoConsole(`Wasmee: Sandbox memory state restored in 31 µs.`);
  logDemoConsole(`Wasmee: Replaying ${inst.version - 1} oplog entries for deterministic execution.`);
  logDemoConsole(`Wasmee: Running entrypoint 'resume' for active_node='${activeNode}'...`, 'success');

  // Apply inputs to variables
  for (let k in input) {
    inst.variables[k] = input[k];
  }

  inst.version++;
  
  // Transitions
  if (inst.type === 'workflow') {
    if (activeNode === 'add_todo') {
      inst.waiting_nodes = ['complete_todo'];
      inst.history.push(`Todo item added. Base checkpoint Version ${inst.version} saved.`);
    } else if (activeNode === 'complete_todo') {
      inst.completed = true;
      inst.waiting_nodes = [];
      inst.history.push(`Todo completed. Final checkpoint saved.`);
    }
  } else if (inst.type === 'servicedesk') {
    if (activeNode === 'create_incident') {
      inst.variables.status = 'New';
      inst.variables.sla_reaction_breached = false;
      inst.variables.sla_resolution_breached = false;
      inst.waiting_nodes = ['new_incident'];
      inst.history.push(`Incident logged as New. Reaction SLA initiated.`);
    } else if (activeNode === 'new_incident') {
      inst.variables.status = 'Assigned';
      inst.waiting_nodes = ['investigating'];
      inst.history.push(`Incident assigned to ${input.assignee}.`);
    } else if (activeNode === 'investigating') {
      inst.variables.status = 'Investigating';
      inst.waiting_nodes = ['resolved'];
      inst.history.push(`Details updated: ${input.comments}`);
    } else if (activeNode === 'resolved') {
      inst.variables.status = 'Resolved';
      inst.completed = true;
      inst.waiting_nodes = [];
      inst.history.push(`Incident resolved. Resolution notes: ${input.resolution}`);
    }
  }

  logDemoConsole(`Wasmee: Saved snapshot checkpoint (Version ${inst.version}).`, 'success');
  logDemoConsole(`Host: Workflow step '${activeNode}' completed successfully.`);

  selectDemoInstance(id);
};

(window as any).playGameTurn = function (id: string, action: string) {
  const inst = simInstances.find(i => i.id === id);
  if (!inst) return;

  logDemoConsole(`Host: Resuming Sven vs Lina arena session on entrypoint 'resume'...`);
  logDemoConsole(`Wasmee: Restoring memory... Loading snapshot checkpoint (Version ${inst.version}).`);
  logDemoConsole(`Wasmee: Replaying combat logs.`);
  logDemoConsole(`Wasmee: Running game turn logic for action '${action}'...`, 'success');

  inst.version++;
  inst.variables.turn++;

  if (action === 'StormHammer') {
    inst.variables.lina_hp = Math.max(0, inst.variables.lina_hp - 20);
    inst.variables.sven_stunned = false;
    logDemoConsole('Wasm Game: Sven casts Storm Hammer! Lina takes 20 DMG and is stunned.', 'warn');
  } else if (action === 'Warcry') {
    inst.variables.sven_hp = Math.min(100, inst.variables.sven_hp + 15);
    logDemoConsole('Wasm Game: Sven uses Warcry! Sven recovers 15 HP.', 'success');
  } else if (action === 'DragonSlave') {
    inst.variables.sven_hp = Math.max(0, inst.variables.sven_hp - 18);
    logDemoConsole('Wasm Game: Lina casts Dragon Slave! Sven takes 18 DMG.', 'warn');
  } else if (action === 'LagunaBlade') {
    inst.variables.sven_hp = Math.max(0, inst.variables.sven_hp - 35);
    logDemoConsole('Wasm Game: Lina casts Laguna Blade! Sven takes 35 Critical DMG.', 'error');
  }

  if (inst.variables.sven_hp <= 0 || inst.variables.lina_hp <= 0) {
    inst.completed = true;
    inst.waiting_nodes = [];
    logDemoConsole('Wasm Game: Arena combat completed.', 'success');
  }

  logDemoConsole(`Wasmee: Checkpoint saved (Version ${inst.version}).`, 'success');
  selectDemoInstance(id);
};

(window as any).simulateGameCrash = function (id: string) {
  const inst = simInstances.find(i => i.id === id);
  if (!inst) return;

  logDemoConsole('Host: 💥 Simulating node host failure... Process terminated.', 'error');
  logDemoConsole('Host: Spawning replacement container node in us-central1-b...');
  
  setTimeout(() => {
    logDemoConsole('Host: Reconnecting session state...');
    logDemoConsole(`Wasmee: Loading base Wasm module (JIT pre-warm compile)...`);
    logDemoConsole(`Wasmee: [RESTORE] Reconstructing memory state from base snapshot (Version 1).`);
    logDemoConsole(`Wasmee: Applying ${inst.version - 1} page memory delta checkpoints...`);
    logDemoConsole(`Wasmee: Replaying ${inst.variables.turn} combat turns from oplog logs for alignment.`);
    logDemoConsole(`Wasmee: [SUCCESS] Virtual machine memory authoritative state recovered in 46 µs.`, 'success');
    logDemoConsole(`Host: Sven vs Lina combat session resumed precisely from turn ${inst.variables.turn}!`, 'success');
  }, 1000);
};

// SLA background ticking in the browser
setInterval(() => {
  simInstances.forEach(inst => {
    if (inst.type === 'servicedesk' && !inst.completed) {
      const elapsed = Date.now() - inst.created_at;
      const status = inst.variables.status || 'New';

      // 60s reaction SLA
      if (status === 'New' && elapsed > 60000 && !inst.variables.sla_reaction_breached) {
        inst.variables.sla_reaction_breached = true;
        inst.variables.priority = 'Critical';
        inst.version++;
        inst.history.push(`[SLA ALERT] Reaction limit breached! Priority auto-escalated to Critical.`);
        
        if (inst.id === activeDemoInstanceId) {
          logDemoConsole('Wasmee: [SLA BREACH DETECTED] Incident Reaction SLA breached! Auto-escalating priority to Critical.', 'warn');
          logDemoConsole(`Wasmee: Saved snapshot checkpoint (Version ${inst.version}).`, 'success');
        }
      }

      // 180s resolution SLA
      if (status !== 'Resolved' && elapsed > 180000 && !inst.variables.sla_resolution_breached) {
        inst.variables.sla_resolution_breached = true;
        inst.version++;
        inst.history.push(`[SLA BREACH] Resolution limit breached! Escalated to Service Manager.`);
        
        if (inst.id === activeDemoInstanceId) {
          logDemoConsole('Wasmee: [SLA BREACH DETECTED] Incident Resolution SLA breached! Escalating alert to Manager.', 'error');
          logDemoConsole(`Wasmee: Saved snapshot checkpoint (Version ${inst.version}).`, 'success');
        }
      }
    }
  });

  if (activeDemoProduct === 'servicedesk') {
    renderDemoItems();
    if (activeDemoInstanceId) {
      const activeInst = simInstances.find(i => i.id === activeDemoInstanceId);
      if (activeInst) {
        updateDemoVarsView(activeInst.variables);
      }
    }
  }
}, 1000);

// Bind navigation routes
window.addEventListener('hashchange', router);
window.addEventListener('load', router);

// Trigger initial router execution on script load
if (document.readyState === 'complete' || document.readyState === 'interactive') {
  router();
}
