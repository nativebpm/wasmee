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
  go: 'main.go'
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
