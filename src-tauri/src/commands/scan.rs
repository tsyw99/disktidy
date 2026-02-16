use crate::models::{ScanOptions, ScanProgress, ScanResult, CleanResult, CategoryFilesResponse};
use crate::modules;
use tauri::{command, AppHandle};

#[command]
pub async fn disk_scan_start(app: AppHandle, options: ScanOptions) -> Result<String, String> {
    modules::start_scan(app, options).await
}

#[command]
pub async fn disk_scan_pause(scan_id: String) -> Result<(), String> {
    modules::pause_scan(&scan_id).await
}

#[command]
pub async fn disk_scan_resume(scan_id: String) -> Result<(), String> {
    modules::resume_scan(&scan_id).await
}

#[command]
pub async fn disk_scan_cancel(scan_id: String) -> Result<(), String> {
    modules::cancel_scan(&scan_id).await
}

#[command]
pub async fn disk_scan_progress(scan_id: String) -> Result<Option<ScanProgress>, String> {
    Ok(modules::get_scan_progress(&scan_id).await)
}

#[command]
pub async fn disk_scan_result(scan_id: String) -> Result<Option<ScanResult>, String> {
    Ok(modules::get_scan_result(&scan_id).await)
}

#[command]
pub async fn disk_scan_category_files(scan_id: String, category_name: String, offset: u64, limit: u64) -> Result<Option<CategoryFilesResponse>, String> {
    Ok(modules::get_category_files(&scan_id, &category_name, offset, limit).await)
}

#[command]
pub async fn disk_scan_delete_files(scan_id: String, move_to_trash: bool) -> Result<CleanResult, String> {
    modules::delete_scanned_files(&scan_id, move_to_trash).await
}

#[command]
pub async fn disk_scan_delete_selected(scan_id: String, file_paths: Vec<String>, move_to_trash: bool) -> Result<CleanResult, String> {
    modules::delete_selected_files(&scan_id, file_paths, move_to_trash).await
}

#[command]
pub async fn disk_scan_clear_result(scan_id: String) -> Result<(), String> {
    modules::clear_scan_result(&scan_id).await
}
