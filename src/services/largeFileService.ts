/**
 * 大文件扫描服务
 * 
 * 使用 Tauri 事件系统接收实时进度更新
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  LargeFileScanOptions,
  LargeFileScanProgress,
  LargeFileScanResult,
} from '../types/largeFile';

const EVENT_LARGE_FILE_PROGRESS = 'large-file:progress';
const EVENT_LARGE_FILE_COMPLETE = 'large-file:complete';

export class LargeFileService {
  private progressUnlistener: UnlistenFn | null = null;
  private completeUnlistener: UnlistenFn | null = null;

  async startScan(options: LargeFileScanOptions): Promise<string> {
    return invoke<string>('large_file_scan_start', { options });
  }

  async pauseScan(scanId: string): Promise<void> {
    return invoke<void>('large_file_scan_pause', { scanId });
  }

  async resumeScan(scanId: string): Promise<void> {
    return invoke<void>('large_file_scan_resume', { scanId });
  }

  async cancelScan(scanId: string): Promise<void> {
    return invoke<void>('large_file_scan_cancel', { scanId });
  }

  async getProgress(scanId: string): Promise<LargeFileScanProgress | null> {
    return invoke<LargeFileScanProgress | null>('large_file_scan_get_progress', { scanId });
  }

  async getResult(scanId: string): Promise<LargeFileScanResult | null> {
    return invoke<LargeFileScanResult | null>('large_file_scan_get_result', { scanId });
  }

  async clearResult(scanId: string): Promise<void> {
    return invoke<void>('large_file_scan_clear', { scanId });
  }

  async onProgress(callback: (progress: LargeFileScanProgress) => void): Promise<void> {
    if (this.progressUnlistener) {
      this.progressUnlistener();
    }
    
    this.progressUnlistener = await listen<LargeFileScanProgress>(
      EVENT_LARGE_FILE_PROGRESS,
      (event) => callback(event.payload)
    );
  }

  async onComplete(callback: (result: LargeFileScanResult) => void): Promise<void> {
    if (this.completeUnlistener) {
      this.completeUnlistener();
    }
    
    this.completeUnlistener = await listen<LargeFileScanResult>(
      EVENT_LARGE_FILE_COMPLETE,
      (event) => callback(event.payload)
    );
  }

  unsubscribe(): void {
    if (this.progressUnlistener) {
      this.progressUnlistener();
      this.progressUnlistener = null;
    }
    if (this.completeUnlistener) {
      this.completeUnlistener();
      this.completeUnlistener = null;
    }
  }
}

export const largeFileService = new LargeFileService();
