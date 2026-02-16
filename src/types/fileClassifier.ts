export interface FileTypeStats {
  category: string;
  display_name: string;
  count: number;
  total_size: number;
  percentage: number;
  extensions: Record<string, ExtensionStats>;
}

export interface ExtensionStats {
  extension: string;
  count: number;
  total_size: number;
}

export interface FileBriefInfo {
  path: string;
  name: string;
  size: number;
  extension: string;
  category: string;
  modified_time: number;
}

export interface FileClassificationResult {
  scan_id: string;
  path: string;
  total_files: number;
  total_size: number;
  total_folders: number;
  categories: FileTypeStats[];
  duration_ms: number;
  largest_files: FileBriefInfo[];
  cancelled?: boolean;
}

export interface FileClassificationOptions {
  max_depth?: number;
  include_hidden: boolean;
  include_system: boolean;
  exclude_paths: string[];
  max_files?: number;
  top_n_categories?: number;
}

export const DEFAULT_CLASSIFICATION_OPTIONS: FileClassificationOptions = {
  max_depth: 5,
  include_hidden: false,
  include_system: false,
  exclude_paths: [
    'Windows',
    'Program Files',
    'Program Files (x86)',
    'ProgramData',
    '$Recycle.Bin',
    'System Volume Information',
  ],
  max_files: 100000,
  top_n_categories: 15,
};
