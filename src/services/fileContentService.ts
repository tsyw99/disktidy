import { invoke } from '@tauri-apps/api/core';
import type {
  AnalysisReportSummary,
  SimilarityResponse,
  BatchSimilarityResponse,
} from '../types/fileContent';

export const fileContentService = {
  /** 分析指定文件内容 */
  analyzeFiles: (paths: string[], generateHtml = false): Promise<AnalysisReportSummary> =>
    invoke<AnalysisReportSummary>('content_analyze_files', {
      request: { paths, generate_html: generateHtml },
    }),

  /** 生成HTML可视化报告 */
  generateHtmlReport: (paths: string[]): Promise<string> =>
    invoke<string>('content_generate_html_report', {
      request: { paths, generate_html: true },
    }),

  /** 比较两个文件的内容相似度 */
  compareSimilarity: (pathA: string, pathB: string): Promise<SimilarityResponse> =>
    invoke<SimilarityResponse>('content_compare_similarity', {
      request: { path_a: pathA, path_b: pathB },
    }),

  /** 批量相似度分析 */
  batchSimilarity: (paths: string[]): Promise<BatchSimilarityResponse> =>
    invoke<BatchSimilarityResponse>('content_batch_similarity', {
      request: { paths },
    }),
};
