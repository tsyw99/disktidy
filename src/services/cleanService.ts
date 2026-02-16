import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  CleanOptions,
  CleanPreview,
  CleanProgress,
  CleanResult,
  CleanReportData,
  SafetyCheckResult,
  RecycleBinInfo,
  GarbageFile,
  DuplicateGroup,
} from '../types';

export const EVENT_CLEAN_PROGRESS = 'clean:progress';
export const EVENT_CLEAN_COMPLETE = 'clean:complete';

export const cleanService = {
  preview: (files: string[]): Promise<CleanPreview> =>
    invoke<CleanPreview>('clean_preview', { files }),

  execute: (files: string[], options?: CleanOptions): Promise<string> =>
    invoke<string>('clean_files', { files, options }),

  cleanGarbage: (garbageFiles: GarbageFile[], options?: CleanOptions): Promise<string> =>
    invoke<string>('clean_garbage_files', { garbageFiles, options }),

  cleanDuplicates: (duplicateGroups: DuplicateGroup[], keepOriginals: boolean, options?: CleanOptions): Promise<string> =>
    invoke<string>('clean_duplicates', { duplicateGroups, keepOriginals, options }),

  cancel: (cleanId: string): Promise<void> =>
    invoke<void>('clean_cancel', { cleanId }),

  getStatus: (cleanId: string): Promise<string> =>
    invoke<string>('clean_status', { cleanId }),

  checkFileSafety: (path: string): Promise<SafetyCheckResult> =>
    invoke<SafetyCheckResult>('check_file_safety', { path }),

  generateReport: (result: CleanResult): Promise<CleanReportData> =>
    invoke<CleanReportData>('generate_clean_report', { result }),

  exportReportJson: (report: CleanReportData): Promise<string> =>
    invoke<string>('export_report_json', { report }),

  exportReportHtml: (report: CleanReportData): Promise<string> =>
    invoke<string>('export_report_html', { report }),

  emptyRecycleBin: (): Promise<void> =>
    invoke<void>('empty_recycle_bin'),

  getRecycleBinInfo: (): Promise<RecycleBinInfo> =>
    invoke<RecycleBinInfo>('get_recycle_bin_info'),

  onProgress: (callback: (progress: CleanProgress) => void): Promise<UnlistenFn> =>
    listen<CleanProgress>(EVENT_CLEAN_PROGRESS, (event) => callback(event.payload)),

  onComplete: (callback: (result: CleanResult) => void): Promise<UnlistenFn> =>
    listen<CleanResult>(EVENT_CLEAN_COMPLETE, (event) => callback(event.payload)),
};
