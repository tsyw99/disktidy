export type ResidueType = 'leftover_folder' | 'registry_key' | 'cache_file' | 'config_file';

export interface ResidueItem {
  id: string;
  name: string;
  path: string;
  size: number;
  residue_type: ResidueType;
  app_name: string;
  description: string;
  last_modified: number;
  safe_to_delete: boolean;
  risk_level: string;
}

export interface ResidueScanResult {
  residue_type: ResidueType;
  items: ResidueItem[];
  total_size: number;
  count: number;
}

export interface ResidueScanProgress {
  percent: number;
  current_phase: string;
  current_path: string;
  scanned_count: number;
  found_count: number;
  elapsed_time: number;
}

export interface ResidueScanOptions {
  include_leftover_folders: boolean;
  include_registry_keys: boolean;
  include_cache_files: boolean;
  include_config_files: boolean;
  scan_all_drives: boolean;
  custom_scan_paths: string[];
}

export interface InstalledSoftware {
  name: string;
  publisher?: string;
  install_location?: string;
  uninstall_string?: string;
  version?: string;
  install_date?: string;
}

export interface DeleteResidueResult {
  deleted_count: number;
  deleted_size: number;
  failed_count: number;
  failed_items: FailedItem[];
}

export interface FailedItem {
  id: string;
  path: string;
  error: string;
}

export const RESIDUE_TYPE_NAMES: Record<ResidueType, string> = {
  leftover_folder: '遗留目录',
  registry_key: '注册表项',
  cache_file: '缓存文件',
  config_file: '配置文件',
};

export const RESIDUE_TYPE_DESCRIPTIONS: Record<ResidueType, string> = {
  leftover_folder: '已卸载软件的残留目录',
  registry_key: '已卸载软件的注册表残留项',
  cache_file: '已卸载软件的缓存文件',
  config_file: '已卸载软件的配置文件',
};
