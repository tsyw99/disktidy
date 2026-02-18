import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  JunkScanResult,
  JunkScanOptions,
  JunkFileScanProgress,
} from '../types';
import { EVENT_JUNK_FILE_PROGRESS, EVENT_JUNK_FILE_COMPLETE } from '../types';

export const junkFileScanService = {
  start: (options?: Partial<JunkScanOptions>): Promise<string> =>
    invoke<string>('junk_file_scan_start', {
      options: options ? {
        scanPaths: options.scan_paths ?? [],
        includeEmptyFolders: options.include_empty_folders ?? true,
        includeInvalidShortcuts: options.include_invalid_shortcuts ?? true,
        includeOldLogs: options.include_old_logs ?? true,
        includeOldInstallers: options.include_old_installers ?? true,
        includeInvalidDownloads: options.include_invalid_downloads ?? true,
        includeSmallFiles: options.include_small_files ?? false,
        smallFileMaxSize: options.small_file_max_size ?? 1024 * 100,
        logMaxAgeDays: options.log_max_age_days ?? 30,
        installerMaxAgeDays: options.installer_max_age_days ?? 90,
        excludePaths: options.exclude_paths ?? [],
        includeHidden: options.include_hidden ?? false,
        includeSystem: options.include_system ?? false,
      } : undefined,
    }),

  pause: (scanId: string): Promise<void> =>
    invoke<void>('junk_file_scan_pause', { scanId }),

  resume: (scanId: string): Promise<void> =>
    invoke<void>('junk_file_scan_resume', { scanId }),

  cancel: (scanId: string): Promise<void> =>
    invoke<void>('junk_file_scan_cancel', { scanId }),

  getProgress: (scanId: string): Promise<JunkFileScanProgress | null> =>
    invoke<JunkFileScanProgress | null>('junk_file_scan_progress', { scanId }),

  getResult: (scanId: string): Promise<JunkScanResult[] | null> =>
    invoke<JunkScanResult[] | null>('junk_file_scan_result', { scanId }),

  clearResult: (scanId: string): Promise<void> =>
    invoke<void>('junk_file_scan_clear', { scanId }),

  onProgress: (callback: (progress: JunkFileScanProgress) => void): Promise<UnlistenFn> =>
    listen<JunkFileScanProgress>(EVENT_JUNK_FILE_PROGRESS, (event) => callback(event.payload)),

  onComplete: (callback: (result: JunkScanResult[]) => void): Promise<UnlistenFn> =>
    listen<JunkScanResult[]>(EVENT_JUNK_FILE_COMPLETE, (event) => callback(event.payload)),
};
