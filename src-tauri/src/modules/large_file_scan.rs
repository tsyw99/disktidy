use crate::models::{
    LargeFile, LargeFileAnalysisResult, LargeFileAnalyzerOptions, 
    ScanStatus, generate_scan_id, EVENT_LARGE_FILE_PROGRESS, EVENT_LARGE_FILE_COMPLETE,
};
use crate::utils::path::{PathUtils, SystemPaths};
use crate::utils::file_type::get_file_type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::{RwLock, watch};
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

const PROGRESS_UPDATE_INTERVAL: u64 = 100;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
            status: ScanStatus::Idle,
        }
    }
}

struct ScanController {
    pause_sender: watch::Sender<bool>,
    cancel_sender: watch::Sender<bool>,
}

impl ScanController {
    fn new() -> (Self, watch::Receiver<bool>, watch::Receiver<bool>) {
        let (pause_tx, pause_rx) = watch::channel(false);
        let (cancel_tx, cancel_rx) = watch::channel(false);
        (
            Self {
                pause_sender: pause_tx,
                cancel_sender: cancel_tx,
            },
            pause_rx,
            cancel_rx,
        )
    }

    fn pause(&self) {
        let _ = self.pause_sender.send(true);
    }

    fn resume(&self) {
        let _ = self.pause_sender.send(false);
    }

    fn cancel(&self) {
        let _ = self.cancel_sender.send(true);
    }
}

lazy_static::lazy_static! {
    static ref SCAN_PROGRESS: Arc<RwLock<HashMap<String, LargeFileScanProgress>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_RESULTS: Arc<RwLock<HashMap<String, LargeFileAnalysisResult>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_CONTROLLERS: Arc<RwLock<HashMap<String, ScanController>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub async fn start_large_file_scan(
    app: AppHandle, 
    path: String, 
    threshold: u64,
    options: Option<LargeFileAnalyzerOptions>,
) -> Result<String, String> {
    let scan_id = generate_scan_id();
    
    let progress = LargeFileScanProgress::new(&scan_id);
    SCAN_PROGRESS.write().await.insert(scan_id.clone(), progress);
    
    let (controller, mut pause_receiver, mut cancel_receiver) = ScanController::new();
    SCAN_CONTROLLERS.write().await.insert(scan_id.clone(), controller);
    
    {
        let progress_map = SCAN_PROGRESS.read().await;
        if let Some(progress) = progress_map.get(&scan_id) {
            let _ = app.emit(EVENT_LARGE_FILE_PROGRESS, progress.clone());
        }
    }
    
    let opts = options.unwrap_or_else(|| LargeFileAnalyzerOptions {
        threshold,
        exclude_paths: vec![],
        include_hidden: false,
        include_system: false,
    });
    
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        if let Err(e) = perform_large_file_scan(
            &app_clone, 
            &scan_id_clone, 
            path, 
            opts, 
            &mut pause_receiver, 
            &mut cancel_receiver
        ).await {
            let mut progress_map = SCAN_PROGRESS.write().await;
            if let Some(progress) = progress_map.get_mut(&scan_id_clone) {
                progress.status = ScanStatus::Error;
            }
            eprintln!("Large file scan error: {}", e);
        }
        
        SCAN_CONTROLLERS.write().await.remove(&scan_id_clone);
    });
    
    Ok(scan_id)
}

async fn perform_large_file_scan(
    app: &AppHandle,
    scan_id: &str,
    path: String,
    options: LargeFileAnalyzerOptions,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
) -> Result<(), String> {
    let start_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    let start_instant = Instant::now();
    
    {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Scanning;
            let _ = app.emit(EVENT_LARGE_FILE_PROGRESS, progress.clone());
        }
    }
    
    let scan_path = PathBuf::from(&path);
    if !scan_path.exists() {
        return Err(format!("Path does not exist: {}", path));
    }
    
    let protected_paths = SystemPaths::get_protected_paths();
    let mut large_files: Vec<LargeFile> = Vec::new();
    let mut scanned_count = 0u64;
    let mut last_update = 0u64;
    
    let walker = WalkDir::new(&scan_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if !options.include_hidden {
                if let Some(name) = e.file_name().to_str() {
                    if name.starts_with('.') {
                        return false;
                    }
                }
            }
            true
        });
    
    for entry in walker {
        if *cancel_receiver.borrow() {
            let mut progress_map = SCAN_PROGRESS.write().await;
            if let Some(progress) = progress_map.get_mut(scan_id) {
                progress.status = ScanStatus::Idle;
            }
            return Ok(());
        }
        
        while *pause_receiver.borrow() {
            {
                let mut progress_map = SCAN_PROGRESS.write().await;
                if let Some(progress) = progress_map.get_mut(scan_id) {
                    progress.status = ScanStatus::Paused;
                    let _ = app.emit(EVENT_LARGE_FILE_PROGRESS, progress.clone());
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            if *cancel_receiver.borrow() {
                let mut progress_map = SCAN_PROGRESS.write().await;
                if let Some(progress) = progress_map.get_mut(scan_id) {
                    progress.status = ScanStatus::Idle;
                }
                return Ok(());
            }
        }
        
        match entry {
            Ok(entry) => {
                if entry.file_type().is_file() {
                    let file_path = entry.path();
                    
                    if should_skip(file_path, &options, &protected_paths) {
                        continue;
                    }
                    
                    scanned_count += 1;
                    
                    if let Ok(metadata) = fs::metadata(file_path) {
                        let file_size = metadata.len();
                        
                        if file_size >= options.threshold {
                            if let Some(large_file) = create_large_file(file_path, &metadata) {
                                large_files.push(large_file);
                            }
                        }
                        
                        if scanned_count - last_update >= PROGRESS_UPDATE_INTERVAL {
                            last_update = scanned_count;
                            let elapsed = start_instant.elapsed().as_secs_f64();
                            let found_size: u64 = large_files.iter().map(|f| f.size).sum();
                            
                            let mut progress_map = SCAN_PROGRESS.write().await;
                            if let Some(progress) = progress_map.get_mut(scan_id) {
                                progress.current_path = file_path.to_string_lossy().to_string();
                                progress.scanned_files = scanned_count;
                                progress.found_files = large_files.len() as u64;
                                progress.scanned_size = found_size;
                                progress.speed = if elapsed > 0.0 {
                                    found_size as f64 / elapsed
                                } else {
                                    0.0
                                };
                                progress.status = ScanStatus::Scanning;
                                progress.percent = (scanned_count as f32 / 100000.0).min(0.99) * 100.0;
                                
                                let _ = app.emit(EVENT_LARGE_FILE_PROGRESS, progress.clone());
                            }
                        }
                    }
                }
            }
            Err(e) => {
                if !e.to_string().contains("拒绝访问") && !e.to_string().contains("Access is denied") {
                    eprintln!("Error walking directory: {}", e);
                }
            }
        }
    }
    
    large_files.sort_by(|a, b| b.size.cmp(&a.size));
    
    let end_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;
    
    let total_size: u64 = large_files.iter().map(|f| f.size).sum();
    
    let result = LargeFileAnalysisResult {
        scan_id: scan_id.to_string(),
        total_files: large_files.len() as u64,
        total_size,
        files: large_files,
        threshold: options.threshold,
        duration_ms: end_time - start_time,
    };
    
    {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Completed;
            progress.percent = 100.0;
            progress.found_files = result.total_files;
            progress.total_size = result.total_size;
            
            let _ = app.emit(EVENT_LARGE_FILE_PROGRESS, progress.clone());
        }
    }
    
    SCAN_RESULTS.write().await.insert(scan_id.to_string(), result.clone());
    
    let _ = app.emit(EVENT_LARGE_FILE_COMPLETE, &result);
    
    Ok(())
}

fn should_skip(path: &Path, options: &LargeFileAnalyzerOptions, protected_paths: &[PathBuf]) -> bool {
    let path_str = path.to_string_lossy();
    
    for exclude in &options.exclude_paths {
        if path_str.to_lowercase().starts_with(&exclude.to_lowercase()) {
            return true;
        }
    }
    
    for protected in protected_paths {
        if path.starts_with(protected) {
            return true;
        }
    }
    
    if !options.include_hidden {
        if let Ok(metadata) = fs::metadata(path) {
            let attrs = metadata.file_attributes();
            if attrs & 0x2 != 0 {
                return true;
            }
        }
    }
    
    if !options.include_system {
        if let Ok(metadata) = fs::metadata(path) {
            let attrs = metadata.file_attributes();
            if attrs & 0x4 != 0 {
                return true;
            }
        }
    }
    
    false
}

fn create_large_file(path: &Path, metadata: &fs::Metadata) -> Option<LargeFile> {
    let name = PathUtils::get_filename(path)?;
    let extension = PathUtils::get_extension(path).unwrap_or_default();
    let file_type = get_file_type(&extension);
    
    Some(LargeFile {
        path: path.to_string_lossy().to_string(),
        name,
        size: metadata.len(),
        modified_time: metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        accessed_time: metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        created_time: metadata
            .created()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0),
        file_type,
        extension,
    })
}

pub async fn get_large_file_scan_progress(scan_id: &str) -> Option<LargeFileScanProgress> {
    let progress_map = SCAN_PROGRESS.read().await;
    progress_map.get(scan_id).cloned()
}

pub async fn get_large_file_scan_result(scan_id: &str) -> Option<LargeFileAnalysisResult> {
    let results_map = SCAN_RESULTS.read().await;
    results_map.get(scan_id).cloned()
}

pub async fn pause_large_file_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.pause();
    }
    
    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.status = ScanStatus::Paused;
    }
    Ok(())
}

pub async fn resume_large_file_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.resume();
    }
    
    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.status = ScanStatus::Scanning;
    }
    Ok(())
}

pub async fn cancel_large_file_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.cancel();
    }
    
    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.status = ScanStatus::Idle;
    }
    Ok(())
}

pub async fn clear_large_file_scan_result(scan_id: &str) -> Result<(), String> {
    SCAN_RESULTS.write().await.remove(scan_id);
    SCAN_PROGRESS.write().await.remove(scan_id);
    Ok(())
}
