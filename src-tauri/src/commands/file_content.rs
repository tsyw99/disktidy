use serde::{Deserialize, Serialize};

use crate::modules::file_content_analyzer::{
    FileContentReader, ContentAnalyzer, ReportGenerator, ContentAnalysis,
};

/// 分析报告摘要（返回给前端，不含HTML中大的全文内容）
#[derive(Debug, Clone, Serialize)]
pub struct AnalysisReportSummary {
    pub report_id: String,
    pub files_analyzed: usize,
    pub total_size_bytes: u64,
    pub analyses: Vec<ContentAnalysis>,
}

/// 单文件分析请求
#[derive(Debug, Deserialize)]
pub struct AnalyzeFilesRequest {
    pub paths: Vec<String>,
    pub generate_html: Option<bool>,
}

/// 读取并分析文件内容，生成分析报告
#[tauri::command]
pub async fn content_analyze_files(
    request: AnalyzeFilesRequest,
) -> Result<AnalysisReportSummary, String> {
    let mut analyses = Vec::new();
    let mut errors = Vec::new();

    for path in &request.paths {
        match FileContentReader::read(path) {
            Ok(content) => {
                let analysis = ContentAnalyzer::analyze(&content);
                analyses.push(analysis);
            }
            Err(e) => {
                errors.push(format!("{}: {}", path, e));
            }
        }
    }

    if analyses.is_empty() && !errors.is_empty() {
        return Err(errors.join("; "));
    }

    let report = ReportGenerator::generate(analyses);

    Ok(AnalysisReportSummary {
        report_id: report.report_id,
        files_analyzed: report.files_analyzed,
        total_size_bytes: report.total_size_bytes,
        analyses: report.analyses,
    })
}

/// 生成HTML可视化报告
#[tauri::command]
pub async fn content_generate_html_report(
    request: AnalyzeFilesRequest,
) -> Result<String, String> {
    let mut analyses = Vec::new();

    for path in &request.paths {
        match FileContentReader::read(path) {
            Ok(content) => {
                let analysis = ContentAnalyzer::analyze(&content);
                analyses.push(analysis);
            }
            Err(e) => {
                return Err(format!("读取失败 {}: {}", path, e));
            }
        }
    }

    if analyses.is_empty() {
        return Err("没有成功读取任何文件".to_string());
    }

    let report = ReportGenerator::generate(analyses);
    Ok(report.html_report)
}

/// 计算两个文件的相似度
#[derive(Debug, Deserialize)]
pub struct SimilarityRequest {
    pub path_a: String,
    pub path_b: String,
}

#[derive(Debug, Serialize)]
pub struct SimilarityResponse {
    pub similarity: f64,
    pub file_a_name: String,
    pub file_b_name: String,
}

#[tauri::command]
pub async fn content_compare_similarity(
    request: SimilarityRequest,
) -> Result<SimilarityResponse, String> {
    let content_a = FileContentReader::read(&request.path_a)
        .map_err(|e| format!("读取失败: {}", e))?;
    let content_b = FileContentReader::read(&request.path_b)
        .map_err(|e| format!("读取失败: {}", e))?;

    let analysis_a = ContentAnalyzer::analyze(&content_a);
    let analysis_b = ContentAnalyzer::analyze(&content_b);

    let similarity = ContentAnalyzer::similarity(&analysis_a, &analysis_b);

    Ok(SimilarityResponse {
        similarity,
        file_a_name: analysis_a.file_name,
        file_b_name: analysis_b.file_name,
    })
}

/// 批量相似度分析
#[derive(Debug, Deserialize)]
pub struct BatchSimilarityRequest {
    pub paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchSimilarityResponse {
    pub pairs: Vec<SimilarityPair>,
}

#[derive(Debug, Serialize)]
pub struct SimilarityPair {
    pub file_a: String,
    pub file_b: String,
    pub similarity: f64,
}

#[tauri::command]
pub async fn content_batch_similarity(
    request: BatchSimilarityRequest,
) -> Result<BatchSimilarityResponse, String> {
    let mut analyses = Vec::new();
    for path in &request.paths {
        match FileContentReader::read(path) {
            Ok(content) => {
                let analysis = ContentAnalyzer::analyze(&content);
                analyses.push(analysis);
            }
            Err(_) => continue,
        }
    }

    let mut pairs = Vec::new();
    for i in 0..analyses.len() {
        for j in (i + 1)..analyses.len() {
            let similarity = ContentAnalyzer::similarity(&analyses[i], &analyses[j]);
            if similarity > 0.3 {
                pairs.push(SimilarityPair {
                    file_a: analyses[i].file_name.clone(),
                    file_b: analyses[j].file_name.clone(),
                    similarity,
                });
            }
        }
    }

    pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));

    Ok(BatchSimilarityResponse { pairs })
}
