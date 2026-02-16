use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CleanMode {
    MoveToTrash,
    Permanent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CleanStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GarbageCategory {
    SystemTemp,
    BrowserCache,
    AppCache,
    RecycleBin,
    LogFile,
    Other,
}

impl GarbageCategory {
    pub fn display_name(&self) -> &str {
        match self {
            Self::SystemTemp => "系统临时文件",
            Self::BrowserCache => "浏览器缓存",
            Self::AppCache => "应用程序缓存",
            Self::RecycleBin => "回收站",
            Self::LogFile => "日志文件",
            Self::Other => "其他",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    pub fn display_name(&self) -> &str {
        match self {
            Self::Low => "低风险",
            Self::Medium => "中等风险",
            Self::High => "高风险",
            Self::Critical => "极高风险",
        }
    }

    pub fn color(&self) -> &str {
        match self {
            Self::Low => "#52c41a",
            Self::Medium => "#faad14",
            Self::High => "#ff4d4f",
            Self::Critical => "#a8071a",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageFile {
    pub path: String,
    pub size: u64,
    pub category: GarbageCategory,
    pub safe_to_delete: bool,
    pub risk_level: RiskLevel,
    pub modified_time: i64,
    pub accessed_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFile {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified_time: i64,
    pub accessed_time: i64,
    pub created_time: i64,
    pub file_type: String,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileDetails {
    pub file: LargeFile,
    pub owner: Option<String>,
    pub is_readonly: bool,
    pub is_hidden: bool,
    pub is_system: bool,
    pub attributes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileAnalyzerOptions {
    pub threshold: u64,
    pub exclude_paths: Vec<String>,
    pub include_hidden: bool,
    pub include_system: bool,
}

impl Default for LargeFileAnalyzerOptions {
    fn default() -> Self {
        Self {
            threshold: 100 * 1024 * 1024,
            exclude_paths: vec![],
            include_hidden: false,
            include_system: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<DuplicateFile>,
    pub wasted_space: u64,
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateFile {
    pub path: String,
    pub name: String,
    pub modified_time: i64,
    pub is_original: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectorOptions {
    pub min_size: u64,
    pub max_size: Option<u64>,
    pub include_hidden: bool,
    pub use_cache: bool,
}

impl Default for DuplicateDetectorOptions {
    fn default() -> Self {
        Self {
            min_size: 1024 * 1024,
            max_size: None,
            include_hidden: false,
            use_cache: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOptions {
    pub move_to_recycle_bin: bool,
    pub secure_delete: bool,
    pub secure_pass_count: u8,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            move_to_recycle_bin: true,
            secure_delete: false,
            secure_pass_count: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub scan_id: String,
    pub total_files: u64,
    pub cleaned_files: u64,
    pub failed_files: u64,
    pub skipped_files: u64,
    pub total_size: u64,
    pub cleaned_size: u64,
    pub errors: Vec<CleanError>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanError {
    pub path: String,
    pub error_code: String,
    pub error_message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanProgress {
    pub total_files: u64,
    pub cleaned_files: u64,
    pub current_file: String,
    pub percent: f32,
    pub speed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanPreview {
    pub total_files: u64,
    pub total_size: u64,
    pub protected_files: Vec<ProtectedFile>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtectedFile {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanReport {
    pub clean_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub success_count: u64,
    pub failed_count: u64,
    pub failed_files: Vec<FailedFileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedFileInfo {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageAnalysisResult {
    pub scan_id: String,
    pub total_files: u64,
    pub total_size: u64,
    pub categories: HashMap<String, CategoryStats>,
    pub high_risk_count: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryStats {
    pub category: String,
    pub file_count: u64,
    pub total_size: u64,
    pub files: Vec<GarbageFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileAnalysisResult {
    pub scan_id: String,
    pub total_files: u64,
    pub total_size: u64,
    pub files: Vec<LargeFile>,
    pub threshold: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateAnalysisResult {
    pub scan_id: String,
    pub total_groups: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub wasted_space: u64,
    pub groups: Vec<DuplicateGroup>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    pub paths: Vec<String>,
    pub include_hidden: bool,
    pub include_system: bool,
    pub max_files: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgress {
    pub analysis_type: AnalysisType,
    pub current_phase: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub percent: f32,
    pub current_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisType {
    Garbage,
    LargeFile,
    Duplicate,
}
