import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, FolderOpen, Check, AlertCircle, Clock } from 'lucide-react';
import { appCacheService } from '../../services/appCacheService';
import type { AppType } from '../../types';
import { APP_DISPLAY_NAMES, APP_PATH_HINTS, APP_ENABLED_STATUS } from '../../types';

interface PathConfigModalProps {
  visible: boolean;
  onClose: () => void;
  onConfigChange: () => void;
}

const APPS: AppType[] = ['wechat', 'qq', 'dingtalk', 'wework'];

export function PathConfigModal({ visible, onClose, onConfigChange }: PathConfigModalProps) {
  const [paths, setPaths] = useState<Record<string, string>>({});
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  useEffect(() => {
    if (visible) {
      loadConfig();
    }
  }, [visible]);

  const loadConfig = async () => {
    try {
      const config = await appCacheService.getConfig();
      setPaths(config || {});
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  };

  const handleSelectFolder = async (app: AppType) => {
    if (!APP_ENABLED_STATUS[app]) return;
    
    try {
      const selected = await appCacheService.selectFolder(paths[app]);
      if (selected) {
        setPaths(prev => ({ ...prev, [app]: selected }));
      }
    } catch (e) {
      console.error('Failed to select folder:', e);
    }
  };

  const handleClearPath = (app: AppType) => {
    if (!APP_ENABLED_STATUS[app]) return;
    
    setPaths(prev => {
      const newPaths = { ...prev };
      delete newPaths[app];
      return newPaths;
    });
  };

  const handleSave = async () => {
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      for (const [app, path] of Object.entries(paths)) {
        if (path && APP_ENABLED_STATUS[app as AppType]) {
          await appCacheService.setPath(app, path);
        }
      }
      setSuccess('配置已保存');
      onConfigChange();
      setTimeout(() => {
        onClose();
      }, 1000);
    } catch (e) {
      setError(e instanceof Error ? e.message : '保存失败');
    } finally {
      setLoading(false);
    }
  };

  if (!visible) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
        onClick={onClose}
      >
        <motion.div
          initial={{ scale: 0.95, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.95, opacity: 0 }}
          className="bg-[var(--bg-primary)] rounded-xl shadow-2xl w-full max-w-lg mx-4 overflow-hidden"
          onClick={e => e.stopPropagation()}
        >
          <div className="flex items-center justify-between p-4 border-b border-[var(--border-color)]">
            <h2 className="text-lg font-semibold text-[var(--text-primary)]">应用路径配置</h2>
            <button
              onClick={onClose}
              className="p-1 rounded-lg hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              <X className="w-5 h-5 text-[var(--text-secondary)]" />
            </button>
          </div>

          <div className="p-4 space-y-4">
            <p className="text-sm text-[var(--text-secondary)]">
              请为应用设置文件存储路径。目前仅支持微信，其他应用正在开发中。
            </p>

            {APPS.map(app => {
              const isEnabled = APP_ENABLED_STATUS[app];
              
              return (
                <div 
                  key={app} 
                  className={`space-y-2 ${!isEnabled ? 'opacity-50' : ''}`}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <label className={`text-sm font-medium ${isEnabled ? 'text-[var(--text-primary)]' : 'text-[var(--text-tertiary)]'}`}>
                        {APP_DISPLAY_NAMES[app]}
                      </label>
                      {!isEnabled && (
                        <span className="flex items-center gap-1 px-2 py-0.5 rounded-full bg-amber-500/10 text-amber-500 text-xs">
                          <Clock className="w-3 h-3" />
                          开发中
                        </span>
                      )}
                    </div>
                    {isEnabled && paths[app] && (
                      <button
                        onClick={() => handleClearPath(app)}
                        className="text-xs text-red-500 hover:text-red-600"
                      >
                        清除
                      </button>
                    )}
                  </div>
                  <div className="flex gap-2">
                    <div
                      className={`flex-1 px-3 py-2 rounded-lg border border-[var(--border-color)] bg-[var(--bg-secondary)] text-sm truncate transition-colors ${
                        isEnabled 
                          ? 'text-[var(--text-primary)] cursor-pointer hover:border-[var(--color-primary)]/50' 
                          : 'text-[var(--text-tertiary)] cursor-not-allowed'
                      }`}
                      onClick={() => handleSelectFolder(app)}
                      title={isEnabled ? (paths[app] || '点击选择文件夹') : '此功能开发中'}
                    >
                      {paths[app] || (isEnabled ? '点击选择文件夹...' : '开发中...')}
                    </div>
                    <button
                      onClick={() => handleSelectFolder(app)}
                      disabled={!isEnabled}
                      className={`px-3 py-2 rounded-lg border border-[var(--border-color)] transition-colors ${
                        isEnabled 
                          ? 'hover:border-[var(--color-primary)]/50' 
                          : 'cursor-not-allowed opacity-50'
                      }`}
                    >
                      <FolderOpen className={`w-4 h-4 ${isEnabled ? 'text-[var(--text-secondary)]' : 'text-[var(--text-tertiary)]'}`} />
                    </button>
                  </div>
                  <p className="text-xs text-[var(--text-tertiary)]">{APP_PATH_HINTS[app]}</p>
                </div>
              );
            })}

            {error && (
              <div className="flex items-center gap-2 p-3 rounded-lg bg-red-500/10 border border-red-500/20">
                <AlertCircle className="w-4 h-4 text-red-500 flex-shrink-0" />
                <p className="text-sm text-red-500">{error}</p>
              </div>
            )}

            {success && (
              <div className="flex items-center gap-2 p-3 rounded-lg bg-green-500/10 border border-green-500/20">
                <Check className="w-4 h-4 text-green-500 flex-shrink-0" />
                <p className="text-sm text-green-500">{success}</p>
              </div>
            )}
          </div>

          <div className="flex justify-end gap-3 p-4 border-t border-[var(--border-color)]">
            <button
              onClick={onClose}
              className="px-4 py-2 rounded-lg border border-[var(--border-color)] text-sm font-medium text-[var(--text-secondary)] hover:bg-[var(--bg-tertiary)] transition-colors"
            >
              取消
            </button>
            <button
              onClick={handleSave}
              disabled={loading}
              className="px-4 py-2 rounded-lg bg-[var(--color-primary)] text-white text-sm font-medium hover:opacity-90 disabled:opacity-50 transition-opacity"
            >
              {loading ? '保存中...' : '保存配置'}
            </button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}
