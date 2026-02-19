import { useState, useEffect } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, Square, X, Maximize2 } from 'lucide-react';

export default function TitleBar() {
  const [isMaximized, setIsMaximized] = useState(false);
  const appWindow = getCurrentWindow();

  useEffect(() => {
    const checkMaximized = async () => {
      const maximized = await appWindow.isMaximized();
      setIsMaximized(maximized);
    };
    checkMaximized();

    const unlisten = appWindow.onResized(() => {
      checkMaximized();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, [appWindow]);

  const handleMinimize = () => {
    appWindow.minimize();
  };

  const handleMaximize = () => {
    if (isMaximized) {
      appWindow.unmaximize();
    } else {
      appWindow.maximize();
    }
  };

  const handleClose = () => {
    appWindow.close();
  };

  return (
    <div
      data-tauri-drag-region
      className="h-10 flex items-center justify-between bg-[var(--bg-secondary)] border-b border-[var(--border-color)] select-none"
    >
      <div data-tauri-drag-region className="flex items-center px-4 flex-1">
        <span className="text-sm font-medium text-[var(--text-primary)]">
          DiskTidy - 磁盘清理工具
        </span>
      </div>

      <div className="flex items-center">
        <button
          onClick={handleMinimize}
          className="w-12 h-10 flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
          title="最小化"
        >
          <Minus className="w-4 h-4" />
        </button>
        <button
          onClick={handleMaximize}
          className="w-12 h-10 flex items-center justify-center text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-tertiary)] transition-colors"
          title={isMaximized ? "还原" : "最大化"}
        >
          {isMaximized ? (
            <Square className="w-3.5 h-3.5" />
          ) : (
            <Maximize2 className="w-3.5 h-3.5" />
          )}
        </button>
        <button
          onClick={handleClose}
          className="w-12 h-10 flex items-center justify-center text-[var(--text-secondary)] hover:text-white hover:bg-red-500 transition-colors"
          title="关闭"
        >
          <X className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
}
