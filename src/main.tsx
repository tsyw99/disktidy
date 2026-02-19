import React from 'react';
import ReactDOM from 'react-dom/client';
import App from './App';
import './styles/index.css';

async function showWindow() {
  try {
    const { getCurrentWindow } = await import('@tauri-apps/api/window');
    const mainWindow = getCurrentWindow();
    await mainWindow.show();
  } catch {
    // Not running in Tauri environment
  }
}

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

showWindow();
