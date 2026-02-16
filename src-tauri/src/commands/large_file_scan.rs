use crate::models::{LargeFileAnalysisResult, LargeFileAnalyzerOptions};
use crate::modules::large_file_scan::LargeFileScanProgress;
use tauri::{command, AppHandle};

#[command]
pub async fn large_file_scan_start(
    app: AppHandle, 
    path: String, 
    threshold: u64,
    options: Option<LargeFileAnalyzerOptions>,
) -> Result<String, String> {
    crate::modules::start_large_file_scan(app, path, threshold, options).await
}

#[command]
pub async fn large_file_scan_pause(scan_id: String) -> Result<(), String> {
    crate::modules::pause_large_file_scan(&scan_id).await
}

#[command]
pub async fn large_file_scan_resume(scan_id: String) -> Result<(), String> {
    crate::modules::resume_large_file_scan(&scan_id).await
}

#[command]
pub async fn large_file_scan_cancel(scan_id: String) -> Result<(), String> {
    crate::modules::cancel_large_file_scan(&scan_id).await
}

#[command]
pub async fn large_file_scan_progress(scan_id: String) -> Result<Option<LargeFileScanProgress>, String> {
    Ok(crate::modules::get_large_file_scan_progress(&scan_id).await)
}

#[command]
pub async fn large_file_scan_result(scan_id: String) -> Result<Option<LargeFileAnalysisResult>, String> {
    Ok(crate::modules::get_large_file_scan_result(&scan_id).await)
}

#[command]
pub async fn large_file_scan_clear(scan_id: String) -> Result<(), String> {
    crate::modules::clear_large_file_scan_result(&scan_id).await
}
