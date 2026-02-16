export const APP_NAME = 'DiskTidy';
export const APP_VERSION = '1.0.0';

export const SCAN_MODES = {
  QUICK: 'quick',
  DEEP: 'deep',
  CUSTOM: 'custom',
} as const;

export const FILE_CATEGORIES = {
  SYSTEM_TEMP: 'system_temp',
  BROWSER_CACHE: 'browser_cache',
  APP_CACHE: 'app_cache',
  LOG_FILES: 'log_files',
  RECYCLE_BIN: 'recycle_bin',
  LARGE_FILES: 'large_files',
  DUPLICATES: 'duplicates',
} as const;

export const CATEGORY_DISPLAY_NAMES: Record<string, string> = {
  system_temp: '系统临时文件',
  browser_cache: '浏览器缓存',
  app_cache: '应用程序缓存',
  log_files: '日志文件',
  recycle_bin: '回收站',
  large_files: '大文件',
  duplicates: '重复文件',
};

export const PROTECTED_PATHS = [
  'C:\\Windows',
  'C:\\Program Files',
  'C:\\Program Files (x86)',
  'C:\\ProgramData',
];

export const DEFAULT_EXCLUDE_PATHS = [
  'C:\\$Recycle.Bin',
  'C:\\System Volume Information',
];
