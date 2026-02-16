use crate::models::{FileInfo, FileCategory, ScanOptions, ScanProgress, ScanResult, ScanStatus, generate_scan_id, EVENT_SCAN_PROGRESS, EVENT_SCAN_COMPLETE, CategoryFilesResponse};
use crate::utils::path::SystemPaths;
use crate::utils::file_category::{FileCategoryRegistry, get_category_description};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::{RwLock, watch};
use walkdir::WalkDir;

struct FullFileCategory {
    name: String,
    display_name: String,
    files: Vec<FileInfo>,
    total_size: u64,
}

lazy_static::lazy_static! {
    static ref SCAN_PROGRESS: Arc<RwLock<HashMap<String, ScanProgress>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_RESULTS: Arc<RwLock<HashMap<String, ScanResult>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_FULL_CATEGORIES: Arc<RwLock<HashMap<String, HashMap<String, FullFileCategory>>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_CONTROLLERS: Arc<RwLock<HashMap<String, ScanController>>> = Arc::new(RwLock::new(HashMap::new()));
}

const PROGRESS_UPDATE_INTERVAL: u64 = 50;
const ESTIMATED_FILES_PER_PATH: u64 = 10000;

fn get_quick_scan_paths() -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    
    let temp_paths = SystemPaths::get_temp_paths();
    for p in temp_paths {
        if p.exists() {
            paths.push(p.to_string_lossy().to_string());
        }
    }
    
    let browser_cache_paths = SystemPaths::get_browser_cache_paths();
    for p in browser_cache_paths {
        if p.exists() {
            paths.push(p.to_string_lossy().to_string());
        }
    }
    
    if let Some(local_app_data) = SystemPaths::app_data_local() {
        let npm_cache = local_app_data.join("npm-cache");
        if npm_cache.exists() {
            paths.push(npm_cache.to_string_lossy().to_string());
        }
        let pip_cache = local_app_data.join("pip\\cache");
        if pip_cache.exists() {
            paths.push(pip_cache.to_string_lossy().to_string());
        }
    }
    
    paths
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

pub async fn start_scan(app: AppHandle, options: ScanOptions) -> Result<String, String> {
    let scan_id = generate_scan_id();
    
    let progress = ScanProgress::new(&scan_id);
    SCAN_PROGRESS.write().await.insert(scan_id.clone(), progress);
    
    let (controller, mut pause_receiver, mut cancel_receiver) = ScanController::new();
    SCAN_CONTROLLERS.write().await.insert(scan_id.clone(), controller);
    
    {
        let progress_map = SCAN_PROGRESS.read().await;
        if let Some(progress) = progress_map.get(&scan_id) {
            let _ = app.emit(EVENT_SCAN_PROGRESS, progress.clone());
        }
    }
    
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        if let Err(e) = perform_scan(&app_clone, &scan_id_clone, options, &mut pause_receiver, &mut cancel_receiver).await {
            let mut progress_map = SCAN_PROGRESS.write().await;
            if let Some(progress) = progress_map.get_mut(&scan_id_clone) {
                progress.status = ScanStatus::Error;
            }
            eprintln!("Scan error: {}", e);
        }
        
        SCAN_CONTROLLERS.write().await.remove(&scan_id_clone);
    });
    
    Ok(scan_id)
}

async fn perform_scan(
    app: &AppHandle,
    scan_id: &str, 
    options: ScanOptions,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
) -> Result<(), String> {
    let _start_time = chrono::Utc::now().timestamp_millis();
    let mut result = ScanResult::new(scan_id);
    
    {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Scanning;
            let _ = app.emit(EVENT_SCAN_PROGRESS, progress.clone());
        }
    }
    
    let category_registry = FileCategoryRegistry::new();
    let mut categories: HashMap<String, FileCategory> = HashMap::new();
    let mut total_files = 0u64;
    let mut total_size = 0u64;
    let start_instant = Instant::now();
    let mut last_update = 0u64;
    
    let scan_paths = if options.mode == "quick" {
        get_quick_scan_paths()
    } else {
        options.paths.clone()
    };
    
    let total_paths = scan_paths.len() as u64;
    let mut current_path_index = 0u64;
    
    for path_str in &scan_paths {
        let path = Path::new(path_str);
        if !path.exists() {
            current_path_index += 1;
            continue;
        }
        
        let walker = WalkDir::new(path)
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
                        if let Ok(metadata) = entry.metadata() {
                            let file_size = metadata.len();
                            let file_path = entry.path().display().to_string();
                            let file_name = entry.file_name().to_string_lossy().to_string();
                            let modified_time = metadata.modified()
                                .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64)
                                .unwrap_or(0);
                            
                            let category_name_opt = categorize_file_with_registry(&file_path, &category_registry);
                            
                            let category_name = match category_name_opt {
                                Some(name) => name,
                                None => continue,
                            };
                            
                            let file_info = FileInfo {
                                path: file_path.clone(),
                                name: file_name,
                                size: file_size,
                                modified_time,
                                category: category_name.clone(),
                            };
                            
                            let category = categories.entry(category_name.clone()).or_insert_with(|| {
                                FileCategory {
                                    name: category_name.clone(),
                                    display_name: get_category_display_name_with_desc(&category_name),
                                    description: get_category_description(&category_name),
                                    file_count: 0,
                                    total_size: 0,
                                    files: Vec::new(),
                                    has_more: false,
                                }
                            });
                            
                            category.file_count += 1;
                            category.total_size += file_size;
                            category.files.push(file_info);
                            
                            total_files += 1;
                            total_size += file_size;
                            
                            if total_files - last_update >= PROGRESS_UPDATE_INTERVAL {
                                last_update = total_files;
                                let mut progress_map = SCAN_PROGRESS.write().await;
                                if let Some(progress) = progress_map.get_mut(scan_id) {
                                    progress.current_path = entry.path().display().to_string();
                                    progress.scanned_files = total_files;
                                    progress.scanned_size = total_size;
                                    let elapsed = start_instant.elapsed().as_secs_f64();
                                    progress.speed = if elapsed > 0.0 {
                                        total_size as f64 / elapsed
                                    } else {
                                        0.0
                                    };
                                    progress.status = ScanStatus::Scanning;
                                    
                                    let path_progress = if total_paths > 0 {
                                        (current_path_index as f32 / total_paths as f32) * 100.0
                                    } else {
                                        0.0
                                    };
                                    let file_progress_factor = 0.3;
                                    let estimated_files = ESTIMATED_FILES_PER_PATH * total_paths;
                                    let file_progress = if estimated_files > 0 {
                                        (total_files as f32 / estimated_files as f32).min(1.0) * 100.0
                                    } else {
                                        0.0
                                    };
                                    progress.percent = path_progress * (1.0 - file_progress_factor) + file_progress * file_progress_factor;
                                    progress.percent = progress.percent.min(99.0);
                                    
                                    let _ = app.emit(EVENT_SCAN_PROGRESS, progress.clone());
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
        
        current_path_index += 1;
    }
    
    result.end_time = chrono::Utc::now().timestamp_millis();
    result.total_files = total_files;
    result.total_size = total_size;
    
    let full_categories: HashMap<String, FullFileCategory> = categories
        .into_iter()
        .map(|(name, cat)| {
            (name.clone(), FullFileCategory {
                name: cat.name,
                display_name: cat.display_name,
                files: cat.files,
                total_size: cat.total_size,
            })
        })
        .collect();
    
    let final_categories: Vec<crate::models::FileCategory> = full_categories
        .values()
        .map(|cat| {
            let total = cat.files.len();
            let has_more = total > crate::models::MAX_FILES_PER_CATEGORY;
            let files: Vec<FileInfo> = if has_more {
                cat.files.iter().take(crate::models::MAX_FILES_PER_CATEGORY).cloned().collect()
            } else {
                cat.files.clone()
            };
            crate::models::FileCategory {
                name: cat.name.clone(),
                display_name: cat.display_name.clone(),
                description: get_category_description(&cat.name),
                file_count: total as u64,
                total_size: cat.total_size,
                files,
                has_more,
            }
        })
        .collect();
    
    result.categories = final_categories;
    result.status = ScanStatus::Completed;
    result.duration = (result.end_time - result.start_time) as u64;
    
    {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Completed;
            progress.percent = 100.0;
            
            let _ = app.emit(EVENT_SCAN_PROGRESS, progress.clone());
        }
    }
    
    SCAN_RESULTS.write().await.insert(scan_id.to_string(), result.clone());
    SCAN_FULL_CATEGORIES.write().await.insert(scan_id.to_string(), full_categories);
    
    let _ = app.emit(EVENT_SCAN_COMPLETE, &result);
    
    Ok(())
}

pub async fn get_scan_progress(scan_id: &str) -> Option<ScanProgress> {
    let progress_map = SCAN_PROGRESS.read().await;
    progress_map.get(scan_id).cloned()
}

pub async fn get_scan_result(scan_id: &str) -> Option<ScanResult> {
    let results_map = SCAN_RESULTS.read().await;
    results_map.get(scan_id).cloned()
}

pub async fn get_category_files(scan_id: &str, category_name: &str, offset: u64, limit: u64) -> Option<CategoryFilesResponse> {
    let full_categories = SCAN_FULL_CATEGORIES.read().await;
    let categories = full_categories.get(scan_id)?;
    let category = categories.get(category_name)?;
    
    let total = category.files.len() as u64;
    let start = offset as usize;
    let end = std::cmp::min(start + limit as usize, category.files.len());
    
    let files: Vec<FileInfo> = if start < category.files.len() {
        category.files[start..end].to_vec()
    } else {
        Vec::new()
    };
    
    let has_more = end < category.files.len();
    
    Some(CategoryFilesResponse {
        files,
        total,
        has_more,
    })
}

pub async fn pause_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.pause();
        
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Paused;
        }
        Ok(())
    } else {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Paused;
        }
        Ok(())
    }
}

pub async fn resume_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.resume();
        
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Scanning;
        }
        Ok(())
    } else {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = ScanStatus::Scanning;
        }
        Ok(())
    }
}

pub async fn cancel_scan(scan_id: &str) -> Result<(), String> {
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

fn categorize_file_with_registry(path: &str, registry: &FileCategoryRegistry) -> Option<String> {
    let path_obj = Path::new(path);
    registry.categorize_file(path_obj).map(|c| c.name.clone())
}

fn get_category_display_name_with_desc(name: &str) -> String {
    crate::utils::file_category::get_category_display_name(name)
}

pub async fn delete_scanned_files(scan_id: &str, move_to_trash: bool) -> Result<crate::models::CleanResult, String> {
    let full_categories = SCAN_FULL_CATEGORIES.read().await;
    let categories = full_categories.get(scan_id)
        .ok_or_else(|| "Scan result not found".to_string())?;
    
    let files: Vec<std::path::PathBuf> = categories
        .values()
        .flat_map(|c| c.files.iter().map(|f| std::path::PathBuf::from(&f.path)))
        .collect();
    drop(full_categories);
    
    let options = crate::models::CleanOptions {
        move_to_recycle_bin: move_to_trash,
        secure_delete: false,
        secure_pass_count: 3,
    };
    
    let executor = crate::modules::cleaner::CleanerExecutor::with_options(options);
    let clean_result = executor.clean(files).await
        .map_err(|e| e.to_string())?;
    
    SCAN_RESULTS.write().await.remove(scan_id);
    SCAN_PROGRESS.write().await.remove(scan_id);
    SCAN_FULL_CATEGORIES.write().await.remove(scan_id);
    
    Ok(clean_result)
}

pub async fn delete_selected_files(scan_id: &str, file_paths: Vec<String>, move_to_trash: bool) -> Result<crate::models::CleanResult, String> {
    let files: Vec<std::path::PathBuf> = file_paths
        .iter()
        .map(|p| std::path::PathBuf::from(p))
        .collect();
    
    let options = crate::models::CleanOptions {
        move_to_recycle_bin: move_to_trash,
        secure_delete: false,
        secure_pass_count: 3,
    };
    
    let executor = crate::modules::cleaner::CleanerExecutor::with_options(options);
    let clean_result = executor.clean(files).await
        .map_err(|e| e.to_string())?;
    
    if clean_result.cleaned_files > 0 {
        let cleaned_paths: std::collections::HashSet<String> = file_paths.into_iter().collect();
        
        {
            let mut full_categories = SCAN_FULL_CATEGORIES.write().await;
            if let Some(categories) = full_categories.get_mut(scan_id) {
                for category in categories.values_mut() {
                    category.files.retain(|f| !cleaned_paths.contains(&f.path));
                    category.total_size = category.files.iter().map(|f| f.size).sum();
                }
                categories.retain(|_, c| !c.files.is_empty());
            }
        }
        
        let mut results_map = SCAN_RESULTS.write().await;
        if let Some(result) = results_map.get_mut(scan_id) {
            for category in &mut result.categories {
                category.files.retain(|f| !cleaned_paths.contains(&f.path));
                category.file_count = category.files.len() as u64;
                category.total_size = category.files.iter().map(|f| f.size).sum();
                category.has_more = category.file_count > crate::models::MAX_FILES_PER_CATEGORY as u64;
            }
            result.categories.retain(|c| c.file_count > 0);
            result.total_files = result.categories.iter().map(|c| c.file_count).sum();
            result.total_size = result.categories.iter().map(|c| c.total_size).sum();
        }
    }
    
    Ok(clean_result)
}

pub async fn clear_scan_result(scan_id: &str) -> Result<(), String> {
    SCAN_RESULTS.write().await.remove(scan_id);
    SCAN_PROGRESS.write().await.remove(scan_id);
    SCAN_FULL_CATEGORIES.write().await.remove(scan_id);
    Ok(())
}
