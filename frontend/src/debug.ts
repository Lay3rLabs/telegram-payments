/**
 * Debug utilities for displaying errors on screen
 * (since Telegram browser doesn't have console)
 */

export function createDebugPanel() {
  const panel = document.createElement('div');
  panel.id = 'debug-panel';
  panel.style.cssText = `
    position: fixed;
    top: 10px;
    left: 10px;
    right: 10px;
    max-height: 300px;
    overflow-y: auto;
    background: rgba(0, 0, 0, 0.9);
    color: #0f0;
    padding: 10px;
    font-family: monospace;
    font-size: 11px;
    z-index: 99999;
    border: 2px solid #0f0;
    border-radius: 4px;
  `;
  document.body.appendChild(panel);
  return panel;
}

export function log(message: string, type: 'info' | 'error' | 'warn' = 'info') {
  console.log(`[${type}]`, message);

  let panel = document.getElementById('debug-panel');
  if (!panel) {
    panel = createDebugPanel();
  }

  const color = type === 'error' ? '#f00' : type === 'warn' ? '#ff0' : '#0f0';
  const timestamp = new Date().toLocaleTimeString();

  const line = document.createElement('div');
  line.style.cssText = `color: ${color}; margin-bottom: 4px;`;
  line.textContent = `[${timestamp}] ${message}`;
  panel.appendChild(line);

  // Auto scroll to bottom
  panel.scrollTop = panel.scrollHeight;
}

export function clearDebug() {
  const panel = document.getElementById('debug-panel');
  if (panel) {
    panel.remove();
  }
}

// Make it available globally
(window as any).debugLog = log;
(window as any).clearDebug = clearDebug;
