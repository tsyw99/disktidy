use serde::{Deserialize, Serialize};

use super::cleaner::{AnalysisType, CleanResult};

pub const EVENT_SCAN_PROGRESS: &str = "scan:progress";
pub const EVENT_SCAN_COMPLETE: &str = "scan:complete";
pub const EVENT_CLEAN_PROGRESS: &str = "clean:progress";
pub const EVENT_CLEAN_COMPLETE: &str = "clean:complete";
pub const EVENT_ERROR: &str = "app:error";
pub const EVENT_ANALYSIS_PROGRESS: &str = "analysis:progress";
pub const EVENT_ANALYSIS_COMPLETE: &str = "analysis:complete";
pub const EVENT_LARGE_FILE_PROGRESS: &str = "large_file:progress";
pub const EVENT_LARGE_FILE_COMPLETE: &str = "large_file:complete";
pub const EVENT_JUNK_FILE_PROGRESS: &str = "junk_file:progress";
pub const EVENT_JUNK_FILE_COMPLETE: &str = "junk_file:complete";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AppEvent {
    ScanProgress(ScanProgressEvent),
    ScanComplete(ScanCompleteEvent),
    AnalysisProgress(AnalysisProgressEvent),
    AnalysisComplete(AnalysisCompleteEvent),
    CleanProgress(CleanProgressEvent),
    CleanComplete(CleanCompleteEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanProgressEvent {
    pub scan_id: String,
    pub current_path: String,
    pub scanned_files: u64,
    pub scanned_size: u64,
    pub percent: f32,
    pub speed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanCompleteEvent {
    pub scan_id: String,
    pub total_files: u64,
    pub total_size: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisProgressEvent {
    pub analysis_id: String,
    pub analysis_type: AnalysisType,
    pub current_phase: String,
    pub processed_files: u64,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisCompleteEvent {
    pub analysis_id: String,
    pub analysis_type: AnalysisType,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanProgressEvent {
    pub clean_id: String,
    pub total_files: u64,
    pub cleaned_files: u64,
    pub current_file: String,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanCompleteEvent {
    pub clean_id: String,
    pub result: CleanResult,
}
