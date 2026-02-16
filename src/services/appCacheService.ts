import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import type {
  AppCacheScanProgress,
  AppCacheScanResult,
  AppCacheScanOptions,
} from '../types';
import { EVENT_APP_CACHE_PROGRESS, EVENT_APP_CACHE_COMPLETE } from '../types';

export const appCacheService = {
  startScan: (apps: string[], categories: string[], options?: Partial<AppCacheScanOptions>): Promise<string> =>
    invoke<string>('app_cache_scan_start', { 
      apps, 
      categories,
      incremental: options?.incremental ?? false,
      forceRescan: options?.forceRescan ?? false,
    }),

  pauseScan: (scanId: string): Promise<void> =>
    invoke<void>('app_cache_scan_pause', { scanId }),

  resumeScan: (scanId: string): Promise<void> =>
    invoke<void>('app_cache_scan_resume', { scanId }),

  cancelScan: (scanId: string): Promise<void> =>
    invoke<void>('app_cache_scan_cancel', { scanId }),

  getProgress: (scanId: string): Promise<AppCacheScanProgress | null> =>
    invoke<AppCacheScanProgress | null>('app_cache_scan_progress', { scanId }),

  getResult: (scanId: string): Promise<AppCacheScanResult | null> =>
    invoke<AppCacheScanResult | null>('app_cache_scan_result', { scanId }),

  clearResult: (scanId: string): Promise<void> =>
    invoke<void>('app_cache_scan_clear', { scanId }),

  onProgress: (callback: (progress: AppCacheScanProgress) => void): Promise<UnlistenFn> =>
    listen<AppCacheScanProgress>(EVENT_APP_CACHE_PROGRESS, (event) => callback(event.payload)),

  onComplete: (callback: (result: AppCacheScanResult) => void): Promise<UnlistenFn> =>
    listen<AppCacheScanResult>(EVENT_APP_CACHE_COMPLETE, (event) => callback(event.payload)),

  getConfig: (): Promise<Record<string, string>> =>
    invoke<Record<string, string>>('get_app_config'),

  setPath: (app: string, path: string): Promise<void> =>
    invoke<void>('set_app_path', { app, path }),

  removePath: (app: string): Promise<void> =>
    invoke<void>('remove_app_path', { app }),

  isConfigured: (app: string): Promise<boolean> =>
    invoke<boolean>('is_app_configured', { app }),

  getConfiguredApps: (): Promise<string[]> =>
    invoke<string[]>('get_all_configured_apps'),

  validatePath: (path: string): Promise<boolean> =>
    invoke<boolean>('validate_path', { path }),

  selectFolder: async (defaultPath?: string): Promise<string | null> => {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath,
      title: '选择文件夹',
    });
    return selected as string | null;
  },
};
