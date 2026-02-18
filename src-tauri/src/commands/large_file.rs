//! 大文件扫描命令

use tauri::{command, AppHandle};
use crate::modules::large_file_scanner::{
    self, ScanConfig, LargeFileScanProgress,
};
use crate::models::LargeFileAnalysisResult;

#[command]
pub async fn large_file_scan_start(
    app: AppHandle,
    options: ScanConfig,
) -> Result<String, String> {
    large_file_scanner::start_scan(app, options).await
}

#[command]
pub async fn large_file_scan_pause(scan_id: String) -> Result<(), String> {
    large_file_scanner::pause_scan(&scan_id).await
}

#[command]
pub async fn large_file_scan_resume(scan_id: String) -> Result<(), String> {
    large_file_scanner::resume_scan(&scan_id).await
}

#[command]
pub async fn large_file_scan_cancel(scan_id: String) -> Result<(), String> {
    large_file_scanner::cancel_scan(&scan_id).await
}

#[command]
pub async fn large_file_scan_get_progress(scan_id: String) -> Result<Option<LargeFileScanProgress>, String> {
    Ok(large_file_scanner::get_progress(&scan_id).await)
}

#[command]
pub async fn large_file_scan_get_result(scan_id: String) -> Result<Option<LargeFileAnalysisResult>, String> {
    Ok(large_file_scanner::get_result(&scan_id).await)
}

#[command]
pub async fn large_file_scan_clear(scan_id: String) -> Result<(), String> {
    large_file_scanner::clear_scan(&scan_id).await
}
