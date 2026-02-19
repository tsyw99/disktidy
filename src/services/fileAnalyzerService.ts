import { invoke } from '@tauri-apps/api/core';
import type {
  GarbageDetectorOptions,
  GarbageCategoryInfo,
  GarbageAnalysisResult,
  LargeFileAnalyzerOptions,
  LargeFileAnalysisResult,
  LargeFileDetails,
  LargeFileAnalysisWithStats,
  DuplicateDetectorOptions,
  DuplicateAnalysisResult,
  DuplicateGroup,
  DuplicateSuggestions,
  JunkScanOptions,
  JunkScanResult,
  JunkTypeInfo,
  JunkCategoryFilesResponse,
} from '../types';

export const fileAnalyzerService = {
  analyzeGarbage: (options?: GarbageDetectorOptions): Promise<GarbageAnalysisResult> =>
    invoke<GarbageAnalysisResult>('analyze_garbage_files', { options }),

  analyzeGarbageByCategory: (category: string): Promise<GarbageAnalysisResult> =>
    invoke<GarbageAnalysisResult>('analyze_garbage_by_category', { category }),

  getGarbageCategories: (): Promise<GarbageCategoryInfo[]> =>
    invoke<GarbageCategoryInfo[]>('get_garbage_categories'),

  analyzeLargeFiles: (path: string, threshold?: number, options?: LargeFileAnalyzerOptions): Promise<LargeFileAnalysisResult> =>
    invoke<LargeFileAnalysisResult>('analyze_large_files', { path, threshold, options }),

  analyzeLargeFilesWithStats: (path: string, threshold?: number): Promise<LargeFileAnalysisWithStats> =>
    invoke<LargeFileAnalysisWithStats>('analyze_large_files_with_stats', { path, threshold }),

  getLargeFileDetails: (path: string): Promise<LargeFileDetails | null> =>
    invoke<LargeFileDetails | null>('get_large_file_details', { path }),

  findDuplicates: (paths: string[], minSize?: number, options?: DuplicateDetectorOptions): Promise<DuplicateAnalysisResult> =>
    invoke<DuplicateAnalysisResult>('find_duplicate_files', { paths, minSize, options }),

  getDuplicateSuggestions: (group: DuplicateGroup): Promise<DuplicateSuggestions> =>
    invoke<DuplicateSuggestions>('get_duplicate_suggestions', { group }),

  scanJunkFiles: (options?: JunkScanOptions): Promise<JunkScanResult[]> =>
    invoke<JunkScanResult[]>('scan_junk_files', { options }),

  scanJunkByType: (fileType: string, options?: JunkScanOptions): Promise<JunkScanResult | null> =>
    invoke<JunkScanResult | null>('scan_junk_by_type', { fileType, options }),

  getJunkFileTypes: (): Promise<JunkTypeInfo[]> =>
    invoke<JunkTypeInfo[]>('get_junk_file_types'),

  getJunkCategoryFiles: (
    scanId: string,
    fileType: string,
    offset: number,
    limit: number
  ): Promise<JunkCategoryFilesResponse | null> =>
    invoke<JunkCategoryFilesResponse | null>('junk_file_category_files', {
      scanId,
      fileType,
      offset,
      limit,
    }),
};
