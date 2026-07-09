import { invoke } from '@tauri-apps/api/core';
import type {
  OrganizeScanResult,
  OrganizePreview,
  OrganizeResult,
  CategoryRule,
  ClassificationResult,
} from '../types/fileOrganizer';

export const fileOrganizerService = {
  /** 扫描目录 */
  scan: (
    path: string,
    includeHidden = false,
    maxDepth?: number,
  ): Promise<OrganizeScanResult> =>
    invoke<OrganizeScanResult>('organizer_scan', {
      request: { path, include_hidden: includeHidden, max_depth: maxDepth },
    }),

  /** 预览整理方案 */
  preview: (
    path: string,
    includeHidden = false,
    maxDepth?: number,
    customRules?: CategoryRule[],
  ): Promise<OrganizePreview> =>
    invoke<OrganizePreview>('organizer_preview', {
      request: {
        path,
        include_hidden: includeHidden,
        max_depth: maxDepth,
        custom_rules: customRules,
      },
    }),

  /** 执行整理 */
  execute: (
    path: string,
    includeHidden = false,
    maxDepth?: number,
    customRules?: CategoryRule[],
    confirmedOperations?: number[],
    dryRun = false,
  ): Promise<OrganizeResult> =>
    invoke<OrganizeResult>('organizer_execute', {
      request: {
        path,
        include_hidden: includeHidden,
        max_depth: maxDepth,
        custom_rules: customRules,
        confirmed_operations: confirmedOperations,
        dry_run: dryRun,
      },
    }),

  /** 获取默认分类规则 */
  getDefaultRules: (): Promise<CategoryRule[]> =>
    invoke<CategoryRule[]>('organizer_default_rules'),

  /** 使用自定义规则分类 */
  classify: (
    path: string,
    rules: CategoryRule[],
    includeHidden = false,
    maxDepth?: number,
  ): Promise<ClassificationResult> =>
    invoke<ClassificationResult>('organizer_classify', {
      request: {
        path,
        include_hidden: includeHidden,
        max_depth: maxDepth,
        rules,
      },
    }),
};
