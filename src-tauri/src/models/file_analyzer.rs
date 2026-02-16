use serde::{Deserialize, Serialize};

pub use super::cleaner::{
    AnalysisOptions, AnalysisProgress, AnalysisType, DuplicateAnalysisResult, DuplicateFile,
    DuplicateGroup, GarbageAnalysisResult, GarbageCategory, GarbageFile, LargeFile,
    LargeFileAnalysisResult, LargeFileAnalyzerOptions, RiskLevel,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum JunkCategory {
    TempFiles,
    Cache,
    Logs,
    BrowserCache,
    RecycleBin,
    Thumbnails,
    WindowsUpdate,
}

impl JunkCategory {
    pub fn display_name(&self) -> &str {
        match self {
            Self::TempFiles => "临时文件",
            Self::Cache => "缓存文件",
            Self::Logs => "日志文件",
            Self::BrowserCache => "浏览器缓存",
            Self::RecycleBin => "回收站",
            Self::Thumbnails => "缩略图缓存",
            Self::WindowsUpdate => "Windows更新缓存",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkFile {
    pub path: String,
    pub size: u64,
    pub category: JunkCategory,
    pub description: String,
    pub risk_level: JunkRiskLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JunkRiskLevel {
    Safe,
    Low,
    Medium,
    High,
}

impl JunkRiskLevel {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Safe => "安全",
            Self::Low => "低风险",
            Self::Medium => "中等风险",
            Self::High => "高风险",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkAnalysisResult {
    pub scan_id: String,
    pub total_files: u64,
    pub total_size: u64,
    pub categories: std::collections::HashMap<String, JunkCategoryStats>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkCategoryStats {
    pub category: JunkCategory,
    pub file_count: u64,
    pub total_size: u64,
    pub files: Vec<JunkFile>,
}
