use crate::modules::{
    AppCacheScanOptions, AppCacheScanProgress, AppCacheScanResult,
    start_app_cache_scan, get_app_cache_progress, get_app_cache_result,
    pause_app_cache_scan, resume_app_cache_scan, cancel_app_cache_scan,
    clear_app_cache_result,
};
use tauri::{command, AppHandle};

#[command]
pub async fn app_cache_scan_start(
    app: AppHandle,
    apps: Vec<String>,
    categories: Vec<String>,
    incremental: Option<bool>,
    force_rescan: Option<bool>,
) -> Result<String, String> {
    let options = AppCacheScanOptions { 
        apps, 
        categories,
        incremental: incremental.unwrap_or(false),
        force_rescan: force_rescan.unwrap_or(false),
    };
    start_app_cache_scan(app, options).await
}

#[command]
pub async fn app_cache_scan_pause(scan_id: String) -> Result<(), String> {
    pause_app_cache_scan(&scan_id).await
}

#[command]
pub async fn app_cache_scan_resume(scan_id: String) -> Result<(), String> {
    resume_app_cache_scan(&scan_id).await
}

#[command]
pub async fn app_cache_scan_cancel(scan_id: String) -> Result<(), String> {
    cancel_app_cache_scan(&scan_id).await
}

#[command]
pub async fn app_cache_scan_progress(scan_id: String) -> Result<Option<AppCacheScanProgress>, String> {
    Ok(get_app_cache_progress(&scan_id).await)
}

#[command]
pub async fn app_cache_scan_result(scan_id: String) -> Result<Option<AppCacheScanResult>, String> {
    Ok(get_app_cache_result(&scan_id).await)
}

#[command]
pub async fn app_cache_scan_clear(scan_id: String) -> Result<(), String> {
    clear_app_cache_result(&scan_id).await
}
