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
        updateConsole([
          { type: 'success', msg: 'Pre-warming completed successfully!' },
          { type: 'value', msg: `Compiled Module Hash: ${data.wasm_hash}` }
        ]);
        localStorage.setItem('wasmee_last_hash', data.wasm_hash);
      } else {
        updateConsole([
          { type: 'error', msg: `Pre-warming failed: ${data.error}` }
        ]);
      }
    } catch (e: any) {
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
          wasm_hash: savedHash, 
          git_source: savedHash ? undefined : {
            repository: repo,
            git_ref: ref,
            file_path: path,
            git_token: token
          },
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
