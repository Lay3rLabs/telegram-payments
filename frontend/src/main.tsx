import './polyfills';
import { log } from './debug';

log('🚀 Starting app...');

// Add global error handler FIRST
window.addEventListener('error', (event) => {
  const msg = `Error: ${event.error?.message || event.message}\n${event.error?.stack || ''}`;
  log(msg, 'error');
  console.error('Global error:', event.error);

  // Try to show in Telegram
  try {
    if ((window as any).Telegram?.WebApp?.showAlert) {
      (window as any).Telegram.WebApp.showAlert(msg.substring(0, 200));
    }
  } catch (e) {
    console.error('Failed to show error:', e);
  }
});

window.addEventListener('unhandledrejection', (event) => {
  const msg = `Promise Error: ${event.reason?.message || event.reason}\n${event.reason?.stack || ''}`;
  log(msg, 'error');
  console.error('Unhandled promise rejection:', event.reason);

  // Try to show in Telegram
  try {
    if ((window as any).Telegram?.WebApp?.showAlert) {
      (window as any).Telegram.WebApp.showAlert(msg.substring(0, 200));
    }
  } catch (e) {
    console.error('Failed to show error:', e);
  }
});

log('✅ Error handlers registered');

try {
  log('📦 Importing React...');
  const { StrictMode } = await import('react');
  const { createRoot } = await import('react-dom/client');

  log('📦 Importing components...');
  const { default: App } = await import('./App.tsx');
  const { ErrorBoundary } = await import('./components/ErrorBoundary');

  log('📦 Importing styles...');
  await import('./index.css');

  log('🔍 Finding root element...');
  const rootElement = document.getElementById('root');
  if (!rootElement) {
    throw new Error('Root element not found!');
  }
  log('✅ Root element found');

  log('⚛️ Creating React root...');
  const root = createRoot(rootElement);

  log('⚛️ Rendering app...');
  root.render(
    <StrictMode>
      <ErrorBoundary>
        <App />
      </ErrorBoundary>
    </StrictMode>
  );

  log('✅ App rendered successfully!');

  // Keep debug panel visible for 10 seconds to see wallet initialization
  setTimeout(() => {
    const panel = document.getElementById('debug-panel');
    if (panel && !panel.textContent?.includes('error') && !panel.textContent?.includes('❌')) {
      panel.style.display = 'none';
    }
  }, 10000);

} catch (error: any) {
  log(`❌ FATAL ERROR: ${error.message}\n${error.stack}`, 'error');

  // Show fallback UI
  const root = document.getElementById('root');
  if (root) {
    root.innerHTML = `
      <div style="padding: 20px; background: #fee; color: #c00; font-family: monospace;">
        <h1>Failed to start app</h1>
        <pre>${error.message}\n\n${error.stack}</pre>
        <button onclick="location.reload()" style="margin-top: 10px; padding: 10px 20px; font-size: 16px;">
          Reload
        </button>
      </div>
    `;
  }
}
