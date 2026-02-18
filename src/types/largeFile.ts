/**
 * 大文件管理类型定义
 */

export interface LargeFile {
  path: string;
  name: string;
  size: number;
  modified_time: number;
  accessed_time: number;
  created_time: number;
  extension: string;
  file_type: string;
}

export interface LargeFileScanOptions {
  path: string;
  minSizeBytes: number;
  excludePaths: string[];
  includeHidden: boolean;
  includeSystem: boolean;
}

export interface LargeFileScanProgress {
  scanId: string;
  currentPath: string;
  scannedFiles: number;
  foundFiles: number;
  scannedSize: number;
  totalSize: number;
  percent: number;
  speed: number;
  status: 'idle' | 'scanning' | 'paused' | 'completed' | 'error';
}

export interface LargeFileScanResult {
  scan_id: string;
  files: LargeFile[];
  total_files: number;
  total_size: number;
  duration_ms: number;
  threshold: number;
}

export interface LargeFileFilter {
  minSize: number;
  unit: 'MB' | 'GB';
  fileTypes?: string[];
}

export const DEFAULT_LARGE_FILE_FILTER: LargeFileFilter = {
  minSize: 500,
  unit: 'MB',
  fileTypes: [],
};
