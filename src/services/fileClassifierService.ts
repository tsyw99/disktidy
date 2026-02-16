import { invoke } from '@tauri-apps/api/core';
import type {
  FileClassificationResult,
  FileClassificationOptions,
} from '../types';

export const fileClassifierService = {
  classifyFiles: (path: string, options?: FileClassificationOptions): Promise<FileClassificationResult> =>
    invoke<FileClassificationResult>('classify_files', { path, options }),

  classifyDisk: (disk: string): Promise<FileClassificationResult> =>
    invoke<FileClassificationResult>('classify_disk', { disk }),

  startClassifyFiles: (path: string, options?: FileClassificationOptions): Promise<FileClassificationResult> =>
    invoke<FileClassificationResult>('start_classify_files', { path, options }),

  cancelClassifyFiles: (): Promise<boolean> =>
    invoke<boolean>('cancel_classify_files'),
};
