import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { 
  ScanProgress, 
  ScanResult, 
  ScanOptions,
  CleanResult,
  CategoryFilesResponse
} from '../types';
import { EVENT_SCAN_PROGRESS, EVENT_SCAN_COMPLETE } from '../types';

export const scanService = {
  start: (paths: string[], mode: string, options: Partial<ScanOptions> = {}): Promise<string> =>
    invoke<string>('disk_scan_start', {
      options: {
        paths,
        mode,
        include_hidden: options.include_hidden ?? false,
        include_system: options.include_system ?? false,
        exclude_paths: options.exclude_paths ?? [],
      }
    }),
  
  pause: (scanId: string): Promise<void> =>
    invoke<void>('disk_scan_pause', { scanId }),
  
  resume: (scanId: string): Promise<void> =>
    invoke<void>('disk_scan_resume', { scanId }),
  
  cancel: (scanId: string): Promise<void> =>
    invoke<void>('disk_scan_cancel', { scanId }),
  
  getProgress: (scanId: string): Promise<ScanProgress | null> =>
    invoke<ScanProgress | null>('disk_scan_progress', { scanId }),
  
  getResult: (scanId: string): Promise<ScanResult | null> =>
    invoke<ScanResult | null>('disk_scan_result', { scanId }),
  
  getCategoryFiles: (scanId: string, categoryName: string, offset: number, limit: number): Promise<CategoryFilesResponse | null> =>
    invoke<CategoryFilesResponse | null>('disk_scan_category_files', { scanId, categoryName, offset, limit }),
  
  deleteFiles: (scanId: string, moveToTrash: boolean = true): Promise<CleanResult> =>
    invoke<CleanResult>('disk_scan_delete_files', { scanId, moveToTrash }),
  
  deleteSelectedFiles: (scanId: string, filePaths: string[], moveToTrash: boolean = true): Promise<CleanResult> =>
    invoke<CleanResult>('disk_scan_delete_selected', { scanId, filePaths, moveToTrash }),
  
  clearResult: (scanId: string): Promise<void> =>
    invoke<void>('disk_scan_clear_result', { scanId }),
  
  onProgress: (callback: (progress: ScanProgress) => void): Promise<UnlistenFn> =>
    listen<ScanProgress>(EVENT_SCAN_PROGRESS, (event) => callback(event.payload)),
  
  onComplete: (callback: (result: ScanResult) => void): Promise<UnlistenFn> =>
    listen<ScanResult>(EVENT_SCAN_COMPLETE, (event) => callback(event.payload)),
};
