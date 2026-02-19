//! 优化的磁盘扫描模块（使用 scanner_framework）
//!
//! 性能优化：
//! - 使用并行处理加速文件分类
//! - 使用统一的文件过滤器
//! - 批量处理减少锁竞争

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use rayon::prelude::*;
use tauri::{AppHandle, Emitter};
use walkdir::DirEntry;

use crate::models::{
    FileInfo, FileCategory, ScanOptions, ScanProgress, ScanResult, ScanStatus,
    CategoryFilesResponse, EVENT_SCAN_PROGRESS, EVENT_SCAN_COMPLETE,
};
use crate::modules::scanner_framework::{
    FileWalker, FilterOptions, ScanContext, ScanManager, ScanProgress as ScanProgressTrait,
    StandardFileFilter,
};
use crate::utils::file_category::{FileCategoryRegistry, get_category_description, get_category_display_name};

const PROGRESS_UPDATE_INTERVAL: u64 = 50;

/// 完整文件分类（内部使用）
#[derive(Debug, Clone)]
struct FullFileCategory {
    name: String,
    display_name: String,
    files: Vec<FileInfo>,
    total_size: u64,
}

impl ScanProgressTrait for ScanProgress {
    fn set_status(&mut self, status: ScanStatus) {
        self.status = status;
    }

    fn event_name() -> &'static str {
        EVENT_SCAN_PROGRESS
    }
}

lazy_static::lazy_static! {
    static ref SCAN_MANAGER: ScanManager<ScanProgress, ScanResult> = ScanManager::new();
    static ref SCAN_FULL_CATEGORIES: std::sync::Arc<tokio::sync::RwLock<HashMap<String, HashMap<String, FullFileCategory>>>> = 
        std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new()));
}

fn get_quick_scan_paths() -> Vec<String> {
    use crate::utils::path::SystemPaths;
    
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

/// 启动磁盘扫描
pub async fn start_scan(app: AppHandle, options: ScanOptions) -> Result<String, String> {
    let scan_id = crate::models::generate_scan_id();
    let progress = ScanProgress::new(&scan_id);

    SCAN_MANAGER
        .start_scan_with_id(app, scan_id.clone(), progress, move |mut ctx| async move {
            perform_scan(&mut ctx, options).await
        })
        .await
}

/// 执行扫描
async fn perform_scan(
    ctx: &mut ScanContext<ScanProgress>,
    options: ScanOptions,
) -> Result<ScanResult, String> {
    let mut result = ScanResult::new(&ctx.scan_id);
    let category_registry = FileCategoryRegistry::new();

    let filter_options = FilterOptions {
        include_hidden: options.include_hidden,
        include_system: options.include_system,
        exclude_paths: options.exclude_paths.clone(),
    };
    let filter = StandardFileFilter::new(&filter_options);
    let walker = FileWalker::new(&filter);

    let scan_paths = if options.mode == "quick" {
        get_quick_scan_paths()
    } else {
        options.paths.clone()
    };

    let total_paths = scan_paths.len() as u64;
    let mut current_path_index = 0u64;
    let start_instant = Instant::now();
    let mut last_scanned_update = 0u64;
    let mut scanned_all_files = 0u64;
    let mut scanned_all_size = 0u64;
    let mut categorized_files_count = 0u64;
    let mut categorized_size = 0u64;

    let mut full_categories: HashMap<String, FullFileCategory> = HashMap::new();

    for path_str in &scan_paths {
        if *ctx.cancel_receiver.borrow() {
            result.status = ScanStatus::Cancelled;
            return Ok(result);
        }

        while *ctx.pause_receiver.borrow() {
            let progress = ScanProgress {
                scan_id: ctx.scan_id.clone(),
                current_path: path_str.clone(),
                scanned_files: scanned_all_files,
                scanned_size: scanned_all_size,
                total_files: categorized_files_count,
                total_size: categorized_size,
                percent: 0.0,
                speed: 0.0,
                status: ScanStatus::Paused,
            };
            let _ = ctx.app.emit(EVENT_SCAN_PROGRESS, &progress);
            
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            if *ctx.cancel_receiver.borrow() {
                result.status = ScanStatus::Cancelled;
                return Ok(result);
            }
        }

        let path = Path::new(path_str);
        if !path.exists() {
            current_path_index += 1;
            continue;
        }

        let files: Vec<DirEntry> = walker.walk_files(path).collect();
        let files_count = files.len() as u64;
        
        let path_total_size: u64 = files
            .iter()
            .filter_map(|entry| entry.metadata().ok().map(|m| m.len()))
            .sum();
        scanned_all_size += path_total_size;

        let categorized_files: Vec<(String, FileInfo)> = files
            .into_par_iter()
            .filter_map(|entry| {
                if let Ok(metadata) = entry.metadata() {
                    let file_path = entry.path().display().to_string();
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    let file_size = metadata.len();
                    let modified_time = metadata
                        .modified()
                        .map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as i64)
                        .unwrap_or(0);

                    if let Some(category_name) = categorize_file_with_registry(&file_path, &category_registry) {
                        let file_info = FileInfo {
                            path: file_path,
                            name: file_name,
                            size: file_size,
                            modified_time,
                            category: category_name.clone(),
                        };
                        return Some((category_name, file_info));
                    }
                }
                None
            })
            .collect();

        for (category_name, file_info) in categorized_files {
            let file_size = file_info.size;
            let category = full_categories.entry(category_name.clone()).or_insert_with(|| {
                FullFileCategory {
                    name: category_name.clone(),
                    display_name: get_category_display_name(&category_name),
                    files: Vec::new(),
                    total_size: 0,
                }
            });

            category.total_size += file_size;
            category.files.push(file_info);
            categorized_files_count += 1;
            categorized_size += file_size;
        }

        scanned_all_files += files_count;
        
        if scanned_all_files - last_scanned_update >= PROGRESS_UPDATE_INTERVAL || current_path_index == total_paths - 1 {
            last_scanned_update = scanned_all_files;
            
            let path_progress = if total_paths > 0 {
                ((current_path_index + 1) as f32 / total_paths as f32) * 100.0
            } else {
                0.0
            };
            
            let elapsed = start_instant.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                scanned_all_size as f64 / elapsed
            } else {
                0.0
            };

            let progress = ScanProgress {
                scan_id: ctx.scan_id.clone(),
                current_path: path_str.clone(),
                scanned_files: scanned_all_files,
                scanned_size: scanned_all_size,
                total_files: categorized_files_count,
                total_size: categorized_size,
                percent: path_progress.min(99.0),
                speed,
                status: ScanStatus::Scanning,
            };

            let _ = ctx.app.emit(EVENT_SCAN_PROGRESS, &progress);
        }

        current_path_index += 1;
    }

    result.end_time = chrono::Utc::now().timestamp_millis();
    result.total_files = categorized_files_count;
    result.total_size = categorized_size;
    result.duration = (result.end_time - result.start_time) as u64;
    result.status = ScanStatus::Completed;

    // 转换为最终分类
    let final_categories: Vec<FileCategory> = full_categories
        .values()
        .map(|cat| {
            let total = cat.files.len();
            let has_more = total > crate::models::MAX_FILES_PER_CATEGORY;
            let files: Vec<FileInfo> = if has_more {
                cat.files.iter().take(crate::models::MAX_FILES_PER_CATEGORY).cloned().collect()
            } else {
                cat.files.clone()
            };
            FileCategory {
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

    // 存储完整分类
    SCAN_FULL_CATEGORIES.write().await.insert(ctx.scan_id.clone(), full_categories);

    // 发送完成事件
    let _ = ctx.app.emit(EVENT_SCAN_COMPLETE, &result);

    Ok(result)
}

fn categorize_file_with_registry(path: &str, registry: &FileCategoryRegistry) -> Option<String> {
    let path_obj = Path::new(path);
    registry.categorize_file(path_obj).map(|c| c.name.clone())
}

// 公共 API 函数
pub async fn get_scan_progress(scan_id: &str) -> Option<ScanProgress> {
    SCAN_MANAGER.get_progress(scan_id).await
}

pub async fn get_scan_result(scan_id: &str) -> Option<ScanResult> {
    SCAN_MANAGER.get_result(scan_id).await
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
    SCAN_MANAGER.pause_scan(scan_id).await
}

pub async fn resume_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.resume_scan(scan_id).await
}

pub async fn cancel_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.cancel_scan(scan_id).await
}

pub async fn delete_scanned_files(scan_id: &str, move_to_trash: bool) -> Result<crate::models::CleanResult, String> {
    let full_categories = SCAN_FULL_CATEGORIES.read().await;
    let categories = full_categories.get(scan_id)
        .ok_or_else(|| "Scan result not found".to_string())?;
    
    let files: Vec<PathBuf> = categories
        .values()
        .flat_map(|c| c.files.iter().map(|f| PathBuf::from(&f.path)))
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
    
    SCAN_MANAGER.clear_scan(scan_id).await.ok();
    SCAN_FULL_CATEGORIES.write().await.remove(scan_id);
    
    Ok(clean_result)
}

pub async fn delete_selected_files(scan_id: &str, file_paths: Vec<String>, move_to_trash: bool) -> Result<crate::models::CleanResult, String> {
    let files: Vec<PathBuf> = file_paths
        .iter()
        .map(|p| PathBuf::from(p))
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
        
        let result = SCAN_MANAGER.get_result(scan_id).await;
        if let Some(mut result) = result {
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
    SCAN_MANAGER.clear_scan(scan_id).await?;
    SCAN_FULL_CATEGORIES.write().await.remove(scan_id);
    Ok(())
}
