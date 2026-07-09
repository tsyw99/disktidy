// 文件内容分析相关类型

export interface KeywordItem {
  word: string;
  count: number;
  score: number;
}

export interface StructureSection {
  level: number;
  title: string;
  line_start: number;
  line_end: number;
  char_count: number;
}

export interface ContentAnalysis {
  file_path: string;
  file_name: string;
  file_format: string;
  file_size_bytes: number;
  total_chars: number;
  total_lines: number;
  total_words: number;
  unique_words: number;
  avg_word_length: number;
  paragraphs: number;
  sentences: number;
  reading_time_minutes: number;
  top_keywords: KeywordItem[];
  structure: StructureSection[];
  language_hint: string;
  content_density: number;
}

export interface AnalysisReportSummary {
  report_id: string;
  files_analyzed: number;
  total_size_bytes: number;
  analyses: ContentAnalysis[];
}

export interface SimilarityResponse {
  similarity: number;
  file_a_name: string;
  file_b_name: string;
}

export interface SimilarityPair {
  file_a: string;
  file_b: string;
  similarity: number;
}

export interface BatchSimilarityResponse {
  pairs: SimilarityPair[];
}
