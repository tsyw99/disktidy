use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScanMode {
    Quick,
    Full,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanOptions {
    pub paths: Vec<String>,
    pub mode: String,
    pub include_hidden: bool,
    pub include_system: bool,
    pub exclude_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgress {
    pub scan_id: String,
    pub current_path: String,
    pub scanned_files: u64,
    pub scanned_size: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub percent: f32,
    pub speed: f64,
    pub status: ScanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum ScanStatus {
    Idle,
    Scanning,
    Paused,
    Completed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub start_time: i64,
    pub end_time: i64,
    pub total_files: u64,
    pub total_size: u64,
    pub total_folders: u64,
    pub categories: Vec<FileCategory>,
    pub status: ScanStatus,
    pub duration: u64,
}

pub const MAX_FILES_PER_CATEGORY: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCategory {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub file_count: u64,
    pub total_size: u64,
    pub files: Vec<FileInfo>,
    #[serde(default)]
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub modified_time: i64,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFilesRequest {
    pub scan_id: String,
    pub category_name: String,
    pub offset: u64,
    pub limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryFilesResponse {
    pub files: Vec<FileInfo>,
    pub total: u64,
    pub has_more: bool,
}

impl ScanProgress {
    pub fn new(scan_id: &str) -> Self {
        Self {
            scan_id: scan_id.to_string(),
            current_path: String::new(),
            scanned_files: 0,
            scanned_size: 0,
            total_files: 0,
            total_size: 0,
            percent: 0.0,
            speed: 0.0,
            status: ScanStatus::Idle,
        }
    }
}

impl ScanResult {
    pub fn new(scan_id: &str) -> Self {
        Self {
            scan_id: scan_id.to_string(),
            start_time: chrono::Utc::now().timestamp_millis(),
            end_time: 0,
            total_files: 0,
            total_size: 0,
            total_folders: 0,
            categories: Vec::new(),
            status: ScanStatus::Idle,
            duration: 0,
        }
    }
}

pub fn generate_scan_id() -> String {
    Uuid::new_v4().to_string()
}
