export type JunkCategory =
  | 'TempFiles'
  | 'Cache'
  | 'Logs'
  | 'BrowserCache'
  | 'RecycleBin'
  | 'Thumbnails'
  | 'WindowsUpdate';

export type JunkRiskLevel = 'safe' | 'low' | 'medium' | 'high';

export type AnalysisType = 'Garbage' | 'LargeFile' | 'Duplicate';

export interface JunkFile {
  path: string;
  size: number;
  category: JunkCategory;
  description: string;
  risk_level: JunkRiskLevel;
}

export interface JunkCategoryStats {
  category: JunkCategory;
  file_count: number;
  total_size: number;
  files: JunkFile[];
}

export interface JunkAnalysisResult {
  scan_id: string;
  total_files: number;
  total_size: number;
  categories: Record<string, JunkCategoryStats>;
  duration_ms: number;
}

export interface GarbageDetectorOptions {
  include_system_temp: boolean;
  include_browser_cache: boolean;
  include_app_cache: boolean;
  include_recycle_bin: boolean;
  include_log_files: boolean;
  min_file_age_days: number;
  max_files_per_category?: number;
}

export interface GarbageCategoryInfo {
  category: string;
  display_name: string;
  description: string;
  icon: string;
}

export interface LargeFile {
  path: string;
  name: string;
  size: number;
  modified_time: number;
  accessed_time: number;
  created_time: number;
  file_type: string;
  extension: string;
}

export interface LargeFileDetails {
  file: LargeFile;
  owner: string | null;
  is_readonly: boolean;
  is_hidden: boolean;
  is_system: boolean;
  attributes: number;
}

export interface LargeFileAnalyzerOptions {
  threshold: number;
  exclude_paths: string[];
  include_hidden: boolean;
  include_system: boolean;
}

export interface LargeFileStats {
  total_count: number;
  total_size: number;
  average_size: number;
  largest_file: LargeFile | null;
  type_distribution: TypeStats[];
}

export interface TypeStats {
  file_type: string;
  count: number;
  total_size: number;
  percentage: number;
}

export interface LargeFileAnalysisWithStats {
  result: LargeFileAnalysisResult;
  stats: LargeFileStats;
}

export interface LargeFileAnalysisResult {
  scan_id: string;
  total_files: number;
  total_size: number;
  files: LargeFile[];
  threshold: number;
  duration_ms: number;
}

export interface DuplicateGroup {
  hash: string;
  size: number;
  files: DuplicateFile[];
  wasted_space: number;
  file_type: string;
}

export interface DuplicateFile {
  path: string;
  name: string;
  modified_time: number;
  is_original: boolean;
}

export interface DuplicateDetectorOptions {
  min_size: number;
  max_size?: number;
  include_hidden: boolean;
  use_cache: boolean;
}

export interface DuplicateSuggestions {
  files_to_delete: string[];
  original_file: string | null;
}

export interface DuplicateAnalysisResult {
  scan_id: string;
  total_groups: number;
  total_files: number;
  total_size: number;
  wasted_space: number;
  groups: DuplicateGroup[];
  duration_ms: number;
}

export interface GarbageFile {
  path: string;
  size: number;
  category: string;
  safe_to_delete: boolean;
  risk_level: string;
  modified_time: number;
  accessed_time: number;
}

export interface GarbageAnalysisResult {
  scan_id: string;
  total_files: number;
  total_size: number;
  categories: Record<string, GarbageCategoryStats>;
  high_risk_count: number;
  duration_ms: number;
}

export interface GarbageCategoryStats {
  category: string;
  file_count: number;
  total_size: number;
  files: GarbageFile[];
}

export interface AnalysisProgress {
  analysis_type: AnalysisType;
  current_phase: string;
  processed_files: number;
  total_files: number;
  percent: number;
  current_path: string;
}

export type JunkFileType = 
  | 'empty_folders'
  | 'invalid_shortcuts'
  | 'old_logs'
  | 'old_installers'
  | 'invalid_downloads'
  | 'small_files'
  | 'orphaned_files';

export interface JunkFileItem {
  id: string;
  path: string;
  size: number;
  file_type: JunkFileType;
  description: string;
  modified_time: number;
  created_time: number;
  safe_to_delete: boolean;
  risk_level: string;
}

export interface JunkScanResult {
  file_type: JunkFileType;
  items: JunkFileItem[];
  total_size: number;
  count: number;
}

export interface JunkScanOptions {
  scan_paths: string[];
  include_empty_folders: boolean;
  include_invalid_shortcuts: boolean;
  include_old_logs: boolean;
  include_old_installers: boolean;
  include_invalid_downloads: boolean;
  include_small_files: boolean;
  small_file_max_size: number;
  log_max_age_days: number;
  installer_max_age_days: number;
  exclude_paths: string[];
  include_hidden: boolean;
  include_system: boolean;
}

export interface JunkTypeInfo {
  type_name: string;
  display_name: string;
  description: string;
  icon: string;
}

export const DEFAULT_JUNK_SCAN_OPTIONS: JunkScanOptions = {
  scan_paths: [],
  include_empty_folders: true,
  include_invalid_shortcuts: true,
  include_old_logs: true,
  include_old_installers: true,
  include_invalid_downloads: true,
  include_small_files: false,
  small_file_max_size: 1024 * 100,
  log_max_age_days: 30,
  installer_max_age_days: 90,
  exclude_paths: [],
  include_hidden: false,
  include_system: false,
};

export interface LargeFileFilter {
  minSize: number;
  unit: 'MB' | 'GB';
  fileTypes?: string[];
}

export interface LargeFileScanProgress {
  scan_id: string;
  current_path: string;
  scanned_files: number;
  found_files: number;
  scanned_size: number;
  total_size: number;
  percent: number;
  speed: number;
  status: 'idle' | 'scanning' | 'paused' | 'completed' | 'error';
}

export const EVENT_LARGE_FILE_PROGRESS = 'large_file:progress';
export const EVENT_LARGE_FILE_COMPLETE = 'large_file:complete';

export const DEFAULT_GARBAGE_DETECTOR_OPTIONS: GarbageDetectorOptions = {
  include_system_temp: true,
  include_browser_cache: true,
  include_app_cache: true,
  include_recycle_bin: true,
  include_log_files: true,
  min_file_age_days: 0,
};

export const DEFAULT_LARGE_FILE_OPTIONS: LargeFileAnalyzerOptions = {
  threshold: 100 * 1024 * 1024,
  exclude_paths: [],
  include_hidden: false,
  include_system: false,
};

export const DEFAULT_DUPLICATE_OPTIONS: DuplicateDetectorOptions = {
  min_size: 1024 * 1024,
  include_hidden: false,
  use_cache: true,
};
