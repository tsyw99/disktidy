import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { 
  LargeFileAnalysisResult, 
  LargeFileScanProgress,
  LargeFileAnalyzerOptions,
} from '../types';
import { EVENT_LARGE_FILE_PROGRESS, EVENT_LARGE_FILE_COMPLETE } from '../types';

export const largeFileScanService = {
  start: (path: string, threshold: number, options?: Partial<LargeFileAnalyzerOptions>): Promise<string> =>
    invoke<string>('large_file_scan_start', { 
      path, 
      threshold,
      options: options ? {
        threshold,
        exclude_paths: options.exclude_paths ?? [],
        include_hidden: options.include_hidden ?? false,
        include_system: options.include_system ?? false,
      } : undefined,
    }),
  
  pause: (scanId: string): Promise<void> =>
    invoke<void>('large_file_scan_pause', { scanId }),
  
  resume: (scanId: string): Promise<void> =>
    invoke<void>('large_file_scan_resume', { scanId }),
  
  cancel: (scanId: string): Promise<void> =>
    invoke<void>('large_file_scan_cancel', { scanId }),
  
  getProgress: (scanId: string): Promise<LargeFileScanProgress | null> =>
    invoke<LargeFileScanProgress | null>('large_file_scan_progress', { scanId }),
  
  getResult: (scanId: string): Promise<LargeFileAnalysisResult | null> =>
    invoke<LargeFileAnalysisResult | null>('large_file_scan_result', { scanId }),
  
  clearResult: (scanId: string): Promise<void> =>
    invoke<void>('large_file_scan_clear', { scanId }),
  
  onProgress: (callback: (progress: LargeFileScanProgress) => void): Promise<UnlistenFn> =>
    listen<LargeFileScanProgress>(EVENT_LARGE_FILE_PROGRESS, (event) => callback(event.payload)),
  
  onComplete: (callback: (result: LargeFileAnalysisResult) => void): Promise<UnlistenFn> =>
    listen<LargeFileAnalysisResult>(EVENT_LARGE_FILE_COMPLETE, (event) => callback(event.payload)),
};
