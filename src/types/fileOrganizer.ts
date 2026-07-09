// 文件智能整理相关类型

export interface CategoryRule {
  id: string;
  name: string;
  description: string;
  target_folder: string;
  extensions: string[];
  min_size?: number;
  max_size?: number;
  date_range_days?: number;
  keywords: string[];
  enabled: boolean;
}

export interface ScannedFile {
  path: string;
  name: string;
  extension: string;
  size_bytes: number;
  modified_time: number;
  is_hidden: boolean;
  category: string;
}

export interface OrganizeScanResult {
  scan_id: string;
  root_path: string;
  files: ScannedFile[];
  total_files: number;
  total_size_bytes: number;
  scan_duration_ms: number;
}

export interface OrganizeOperation {
  source_path: string;
  dest_path: string;
  file_name: string;
  file_size_bytes: number;
  action: 'Move' | 'Copy' | 'Skip';
  category: string;
  reason: string;
}

export interface PreviewStats {
  files_to_move: number;
  files_to_copy: number;
  files_to_skip: number;
  total_categories: number;
  size_to_move_bytes: number;
}

export interface OrganizePreview {
  preview_id: string;
  root_path: string;
  operations: OrganizeOperation[];
  total_files: number;
  total_size_bytes: number;
  categories: Record<string, string[]>;
  summary_stats: PreviewStats;
}

export interface OrganizeResult {
  success: boolean;
  moved_count: number;
  copied_count: number;
  skipped_count: number;
  failed_count: number;
  errors: string[];
}

export interface ContentGroup {
  group_id: string;
  group_name: string;
  files: ScannedFile[];
  total_size: number;
  similarity_score: number;
  representative_keywords: string[];
}

export interface CategoryRuleStats {
  rule_id: string;
  rule_name: string;
  target_folder: string;
  file_count: number;
  total_size: number;
}

export interface ClassificationResult {
  rule_groups: Record<string, ScannedFile[]>;
  content_groups: ContentGroup[];
  unclassified: ScannedFile[];
  rule_stats: CategoryRuleStats[];
}
