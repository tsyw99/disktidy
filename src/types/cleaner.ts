export type CleanMode = 'MoveToTrash' | 'Permanent';

export type CleanStatus = 'Pending' | 'Running' | 'Completed' | 'Failed' | 'Cancelled';

export type GarbageCategory = 'SystemTemp' | 'BrowserCache' | 'AppCache' | 'RecycleBin' | 'LogFile' | 'Other';

export type RiskLevel = 'Low' | 'Medium' | 'High' | 'Critical';

export interface CleanOptions {
  move_to_recycle_bin: boolean;
  secure_delete: boolean;
  secure_pass_count: number;
}

export interface CleanPreview {
  total_files: number;
  total_size: number;
  protected_files: ProtectedFile[];
  warnings: string[];
}

export interface ProtectedFile {
  path: string;
  reason: string;
}

export interface CleanProgress {
  total_files: number;
  cleaned_files: number;
  current_file: string;
  percent: number;
  speed: number;
}

export interface CleanResult {
  scan_id: string;
  total_files: number;
  cleaned_files: number;
  failed_files: number;
  skipped_files: number;
  total_size: number;
  cleaned_size: number;
  errors: CleanError[];
  duration_ms: number;
}

export interface CleanError {
  path: string;
  error_code: string;
  error_message: string;
}

export interface CleanReport {
  clean_id: string;
  start_time: number;
  end_time: number;
  total_files: number;
  total_size: number;
  success_count: number;
  failed_count: number;
  failed_files: FailedFileInfo[];
}

export interface FailedFileInfo {
  path: string;
  error: string;
}

export interface CleanReportData {
  clean_id: string;
  start_time: string;
  end_time: string;
  total_files: number;
  total_size: number;
  success_count: number;
  failed_count: number;
  failed_files: FailedFileInfo[];
  duration_seconds: number;
  average_speed: number;
}

export interface SafetyCheckResult {
  safe_to_delete: boolean;
  risk_level: string;
  reason: string | null;
}

export interface RecycleBinInfo {
  total_items: number;
  total_size: number;
}

export const DEFAULT_CLEAN_OPTIONS: CleanOptions = {
  move_to_recycle_bin: true,
  secure_delete: false,
  secure_pass_count: 3,
};
