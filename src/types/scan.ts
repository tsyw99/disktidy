export type ScanMode = 'quick' | 'deep' | 'custom';

export interface ScanOptions {
  paths: string[];
  mode: string;
  include_hidden: boolean;
  include_system: boolean;
}

export type ScanStatus = 'idle' | 'scanning' | 'paused' | 'completed' | 'cancelled' | 'failed' | 'error';

export interface ScanProgress {
  scan_id: string;
  status: ScanStatus;
  current_path: string;
  scanned_files: number;
  scanned_size: number;
  total_files: number;
  total_size: number;
  percent: number;
  speed: number;
}

export interface ScanResult {
  scan_id: string;
  start_time: number;
  end_time: number;
  total_files: number;
  total_size: number;
  total_folders: number;
  categories: FileCategory[];
  status: ScanStatus;
  duration: number;
}

export interface FileCategory {
  name: string;
  display_name: string;
  description: string;
  file_count: number;
  total_size: number;
  files: FileInfo[];
  has_more: boolean;
}

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  modified_time: number;
  category: string;
}

export interface CategoryFilesResponse {
  files: FileInfo[];
  total: number;
  has_more: boolean;
}

export const EVENT_SCAN_PROGRESS = 'scan:progress';
export const EVENT_SCAN_COMPLETE = 'scan:complete';
