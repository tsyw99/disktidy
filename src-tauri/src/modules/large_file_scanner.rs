//! 大文件扫描模块
//!
//! 优化要点：
//! - 流式处理：避免一次性加载所有文件到内存
//! - 实时进度：通过 Tauri 事件发送进度更新
//! - 暂停支持：支持暂停/恢复/取消操作

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use rayon::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::models::{LargeFile, LargeFileAnalysisResult, ScanStatus};
use crate::modules::scanner_framework::{
    ControlAction, FileWalker, FilterOptions, ScanContext, ScanManager,
    ScanProgress as ScanProgressTrait, StandardFileFilter,
};

pub const EVENT_LARGE_FILE_PROGRESS: &str = "large-file:progress";
pub const EVENT_LARGE_FILE_COMPLETE: &str = "large-file:complete";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    pub path: String,
    pub min_size_bytes: u64,
    pub exclude_paths: Vec<String>,
    pub include_hidden: bool,
    pub include_system: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            min_size_bytes: 500 * 1024 * 1024,
            exclude_paths: vec![],
            include_hidden: false,
            include_system: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeFileScanProgress {
    pub scan_id: String,
    pub current_path: String,
    pub scanned_files: u64,
    pub found_files: u64,
    pub scanned_size: u64,
    pub total_size: u64,
    pub percent: f32,
    pub speed: f64,
    pub status: ScanStatus,
}

impl LargeFileScanProgress {
    pub fn new(scan_id: &str) -> Self {
        Self {
            scan_id: scan_id.to_string(),
            current_path: String::new(),
            scanned_files: 0,
            found_files: 0,
            scanned_size: 0,
            total_size: 0,
            percent: 0.0,
            speed: 0.0,
            status: ScanStatus::Scanning,
        }
    }
}

impl ScanProgressTrait for LargeFileScanProgress {
    fn set_status(&mut self, status: ScanStatus) {
        self.status = status;
    }

    fn event_name() -> &'static str {
        EVENT_LARGE_FILE_PROGRESS
    }
}

lazy_static::lazy_static! {
    static ref SCAN_MANAGER: ScanManager<LargeFileScanProgress, LargeFileAnalysisResult> = ScanManager::new();
}

pub async fn start_scan(app: AppHandle, config: ScanConfig) -> Result<String, String> {
    let scan_id = crate::models::generate_scan_id();
    let progress = LargeFileScanProgress::new(&scan_id);
    SCAN_MANAGER
        .start_scan_with_id(app, scan_id, progress, move |mut ctx| async move {
            perform_scan(&mut ctx, config).await
        })
        .await
}

pub async fn pause_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.pause_scan(scan_id).await
}

pub async fn resume_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.resume_scan(scan_id).await
}

pub async fn cancel_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.cancel_scan(scan_id).await
}

pub async fn get_progress(scan_id: &str) -> Option<LargeFileScanProgress> {
    SCAN_MANAGER.get_progress(scan_id).await
}

pub async fn get_result(scan_id: &str) -> Option<LargeFileAnalysisResult> {
    SCAN_MANAGER.get_result(scan_id).await
}

pub async fn clear_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.clear_scan(scan_id).await
}

pub fn init_scanner() {}

async fn perform_scan(
    ctx: &mut ScanContext<LargeFileScanProgress>,
    config: ScanConfig,
) -> Result<LargeFileAnalysisResult, String> {
    let scan_path: PathBuf = config.path.clone().into();
    if !scan_path.exists() {
        return Err(format!("Path does not exist: {}", config.path));
    }

    let start_instant = Instant::now();
    let min_size = config.min_size_bytes;

    let large_files: Arc<tokio::sync::RwLock<Vec<LargeFile>>> =
        Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let scanned_count = Arc::new(AtomicU64::new(0));
    let found_count = Arc::new(AtomicU64::new(0));
    let scanned_size = Arc::new(AtomicU64::new(0));
    let is_paused = Arc::new(AtomicBool::new(false));
    let is_cancelled = Arc::new(AtomicBool::new(false));

    let scan_id = ctx.scan_id.clone();
    let app = ctx.app.clone();

    let large_files_clone = large_files.clone();
    let scanned_count_clone = scanned_count.clone();
    let found_count_clone = found_count.clone();
    let scanned_size_clone = scanned_size.clone();
    let is_paused_clone = is_paused.clone();
    let is_cancelled_clone = is_cancelled.clone();

    let progress_store = SCAN_MANAGER.get_progress_store();
    let progress_store_clone = progress_store.clone();
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();

    let filter_options = FilterOptions {
        include_hidden: config.include_hidden,
        include_system: config.include_system,
        exclude_paths: config.exclude_paths.clone(),
    };

    let handle = tokio::task::spawn_blocking(move || {
        let filter = StandardFileFilter::new(&filter_options);
        let walker = FileWalker::new(&filter);
        let mut last_update = std::time::Instant::now();
        let update_interval = std::time::Duration::from_millis(200);

        walker.walk(&scan_path).for_each(|entry_result| {
            if is_cancelled_clone.load(Ordering::Relaxed) {
                return;
            }

            while is_paused_clone.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if is_cancelled_clone.load(Ordering::Relaxed) {
                    return;
                }
            }

            let entry = match entry_result {
                Ok(e) => e,
                Err(_) => {
                    return;
                }
            };

            if !entry.file_type().is_file() {
                return;
            }

            scanned_count_clone.fetch_add(1, Ordering::Relaxed);

            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len();
                scanned_size_clone.fetch_add(file_size, Ordering::Relaxed);

                if file_size >= min_size {
                    found_count_clone.fetch_add(1, Ordering::Relaxed);

                    if let Some(large_file) = create_large_file(entry.path(), &metadata) {
                        if let Ok(mut files) = large_files_clone.try_write() {
                            files.push(large_file);
                        }
                    }
                }
            }

            let now = std::time::Instant::now();
            if now.duration_since(last_update) >= update_interval {
                last_update = now;

                let current_scanned = scanned_count_clone.load(Ordering::Relaxed);
                let current_found = found_count_clone.load(Ordering::Relaxed);
                let current_size = scanned_size_clone.load(Ordering::Relaxed);
                let current_path_str = entry.path().display().to_string();

                let elapsed = start_instant.elapsed().as_secs_f64();
                let speed = if elapsed > 0.0 {
                    current_size as f64 / elapsed
                } else {
                    0.0
                };

                let progress = LargeFileScanProgress {
                    scan_id: scan_id_clone.clone(),
                    current_path: current_path_str.clone(),
                    scanned_files: current_scanned,
                    found_files: current_found,
                    scanned_size: current_size,
                    total_size: 0,
                    percent: 0.0,
                    speed,
                    status: ScanStatus::Scanning,
                };

                let _ = app_clone.emit(EVENT_LARGE_FILE_PROGRESS, &progress);

                if let Ok(mut store) = progress_store_clone.try_write() {
                    if let Some(p) = store.get_mut(&scan_id_clone) {
                        p.scanned_files = current_scanned;
                        p.found_files = current_found;
                        p.scanned_size = current_size;
                        p.speed = speed;
                        p.current_path = current_path_str;
                    }
                }
            }
        });
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        match ctx.check_control(&SCAN_MANAGER.get_progress_store()).await {
            ControlAction::Cancel => {
                is_cancelled.store(true, Ordering::Relaxed);
                handle.await.ok();
                return Err("Scan cancelled".to_string());
            }
            ControlAction::Continue => {}
        }

        let paused = {
            let store = SCAN_MANAGER.get_progress_store();
            let store = store.read().await;
            store
                .get(&scan_id)
                .map(|p| p.status == ScanStatus::Paused)
                .unwrap_or(false)
        };
        is_paused.store(paused, Ordering::Relaxed);

        if handle.is_finished() {
            break;
        }
    }

    handle.await.ok();

    let mut final_files = large_files.write().await;
    final_files.par_sort_by(|a, b| b.size.cmp(&a.size));

    let total_size: u64 = final_files.iter().map(|f| f.size).sum();
    let found_count = final_files.len() as u64;
    let duration_ms = start_instant.elapsed().as_millis() as u64;

    let result = LargeFileAnalysisResult {
        scan_id: ctx.scan_id.clone(),
        total_files: found_count,
        total_size,
        files: final_files.clone(),
        threshold: config.min_size_bytes,
        duration_ms,
    };

    let _ = ctx.app.emit(EVENT_LARGE_FILE_COMPLETE, &result);

    Ok(result)
}

fn create_large_file(path: &std::path::Path, metadata: &std::fs::Metadata) -> Option<LargeFile> {
    let name = path.file_name()?.to_string_lossy().to_string();
    let extension = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let modified_time = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let accessed_time = metadata
        .accessed()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let created_time = metadata
        .created()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    Some(LargeFile {
        path: path.display().to_string(),
        name,
        size: metadata.len(),
        modified_time,
        accessed_time,
        created_time,
        extension,
        file_type: String::new(),
    })
}
