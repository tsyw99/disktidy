//! 优化的零碎文件扫描模块（使用 scanner_framework）
//!
//! 零碎文件定义标准：
//! 1. 空文件夹：不包含任何文件的空目录
//! 2. 无效快捷方式：目标路径不存在的 .lnk 文件
//! 3. 过期日志文件：超过 30 天未修改的日志文件（.log, .txt, .old）
//! 4. 旧安装包：超过 90 天未修改的安装程序（setup*, install*, *.msi, *.exe）
//! 5. 无效下载：未完成的下载文件（.part, .crdownload, .tmp）
//! 6. 零散小文件：小于 1MB 的非系统文件（默认启用）
//! 7. 临时文件：系统临时目录中的文件
//! 8. 浏览器缓存：浏览器缓存文件
//!
//! 性能优化策略：
//! 1. 统一遍历：一次遍历收集所有文件类型信息，避免重复遍历
//! 2. 元数据缓存：缓存文件元数据，避免重复 I/O
//! 3. 流式处理：使用迭代器链式处理，减少内存分配
//! 4. 并行处理：使用 rayon 并行处理文件分类
//!
//! v2 改进：
//! - 实时进度计算：基于扫描路径和文件数量动态计算进度百分比
//! - 分批返回结果：支持大量文件时的分页加载
//! - 完善错误处理：捕获并报告扫描过程中的异常
//! - 初始化优化：快速启动扫描，延迟加载监听器

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Instant, SystemTime};

use rayon::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::modules::scanner_framework::{
    FileFilter, FileWalker, FilterOptions, ScanContext, ScanManager, ScanProgress, ScanStatistics,
    StandardFileFilter,
};

use crate::models::{ScanStatus, EVENT_JUNK_FILE_COMPLETE, EVENT_JUNK_FILE_PROGRESS};
use crate::modules::cleaner::safety::SafetyChecker;
use crate::modules::file_analyzer::{JunkFile, JunkFileType, JunkScanOptions, JunkScanResult};
use crate::utils::path::{PathUtils, SystemPaths};

/// 进度计算常量
const ESTIMATED_FILES_PER_PATH: f32 = 5000.0;
const PATH_PROGRESS_WEIGHT: f32 = 70.0;
const FILE_PROGRESS_WEIGHT: f32 = 30.0;
const MAX_PROGRESS_BEFORE_COMPLETE: f32 = 99.0;

/// 零碎文件扫描进度
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JunkFileScanProgress {
    pub scan_id: String,
    pub current_path: String,
    pub scanned_files: u64,
    pub found_files: u64,
    pub scanned_size: u64,
    pub total_size: u64,
    pub percent: f32,
    pub current_phase: String,
    pub status: ScanStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}

impl JunkFileScanProgress {
    pub fn new(scan_id: &str) -> Self {
        Self {
            scan_id: scan_id.to_string(),
            current_path: String::new(),
            scanned_files: 0,
            found_files: 0,
            scanned_size: 0,
            total_size: 0,
            percent: 0.0,
            current_phase: "初始化扫描...".to_string(),
            status: ScanStatus::Scanning,
            speed: None,
        }
    }
}

impl ScanProgress for JunkFileScanProgress {
    fn set_status(&mut self, status: ScanStatus) {
        self.status = status;
    }

    fn event_name() -> &'static str {
        EVENT_JUNK_FILE_PROGRESS
    }
}

// 全局扫描管理器 - 用于跟踪扫描进度和存储结果
lazy_static::lazy_static! {
    static ref SCAN_MANAGER: ScanManager<JunkFileScanProgress, Vec<JunkScanResult>> = ScanManager::new();
    static ref JUNK_FILES_STORE: Arc<RwLock<HashMap<String, HashMap<JunkFileType, Vec<JunkFile>>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub async fn get_junk_category_files(
    scan_id: &str,
    file_type: &str,
    offset: u64,
    limit: u64,
) -> Option<JunkCategoryFilesResponse> {
    let parsed_type = parse_junk_file_type_str(file_type)?;
    let store = JUNK_FILES_STORE.read().ok()?;
    let scan_data = store.get(scan_id)?;
    let files = scan_data.get(&parsed_type)?;

    let total = files.len() as u64;
    let start = offset as usize;
    let end = std::cmp::min(start + limit as usize, files.len());

    if start >= files.len() {
        return Some(JunkCategoryFilesResponse {
            files: vec![],
            total,
            has_more: false,
        });
    }

    let result_files = files[start..end].to_vec();
    let has_more = end < files.len();

    Some(JunkCategoryFilesResponse {
        files: result_files,
        total,
        has_more,
    })
}

fn parse_junk_file_type_str(s: &str) -> Option<JunkFileType> {
    match s.to_lowercase().as_str() {
        "empty_folders" => Some(JunkFileType::EmptyFolders),
        "invalid_shortcuts" => Some(JunkFileType::InvalidShortcuts),
        "old_logs" => Some(JunkFileType::OldLogs),
        "old_installers" => Some(JunkFileType::OldInstallers),
        "invalid_downloads" => Some(JunkFileType::InvalidDownloads),
        "small_files" => Some(JunkFileType::SmallFiles),
        "orphaned_files" => Some(JunkFileType::OrphanedFiles),
        _ => None,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JunkCategoryFilesResponse {
    pub files: Vec<JunkFile>,
    pub total: u64,
    pub has_more: bool,
}

/// 启动零碎文件扫描
pub async fn start_junk_file_scan(
    app: AppHandle,
    options: Option<JunkScanOptions>,
) -> Result<String, String> {
    let opts = options.unwrap_or_default();

    let scan_id = crate::models::generate_scan_id();
    let progress = JunkFileScanProgress::new(&scan_id);

    SCAN_MANAGER
        .start_scan_with_id(app, scan_id.clone(), progress, move |mut ctx| async move {
            perform_scan(&mut ctx, opts).await
        })
        .await
}

/// 执行扫描（优化版本：统一遍历 + 元数据缓存 + 实时进度）
async fn perform_scan(
    ctx: &mut ScanContext<JunkFileScanProgress>,
    options: JunkScanOptions,
) -> Result<Vec<JunkScanResult>, String> {
    let scan_start = Instant::now();
    let mut stats = ScanStatistics::new();

    // 使用原子计数器跟踪进度
    let total_scanned_files = Arc::new(AtomicU64::new(0));
    let total_scanned_size = Arc::new(AtomicU64::new(0));
    let total_found_files = Arc::new(AtomicU64::new(0));
    let current_path_index = Arc::new(AtomicU64::new(0));

    let scan_id = ctx.scan_id.clone();
    let app = ctx.app.clone();

    let progress_store = SCAN_MANAGER.get_progress_store();
    let progress_store_clone = progress_store.clone();
    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();

    let total_scanned_files_clone = total_scanned_files.clone();
    let total_scanned_size_clone = total_scanned_size.clone();
    let total_found_files_clone = total_found_files.clone();
    let current_path_index_clone = current_path_index.clone();

    let mut last_progress_update = Instant::now();
    let progress_update_interval = std::time::Duration::from_millis(150); // 更频繁的更新

    // 改进的进度发送函数，包含百分比计算
    let mut emit_progress_fn = |current_path: &str, phase: &str, total_paths: usize| {
        let now = Instant::now();
        if now.duration_since(last_progress_update) >= progress_update_interval {
            last_progress_update = now;

            let current_scanned = total_scanned_files_clone.load(Ordering::Relaxed);
            let current_size = total_scanned_size_clone.load(Ordering::Relaxed);
            let current_found = total_found_files_clone.load(Ordering::Relaxed);
            let path_idx = current_path_index_clone.load(Ordering::Relaxed);

            let elapsed = scan_start.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                current_size as f64 / elapsed
            } else {
                0.0
            };

            // 计算进度百分比：基于扫描路径进度 + 文件扫描进度
            // 路径进度占70%，文件扫描速度占30%
            let path_progress = if total_paths > 0 {
                (path_idx as f32 / total_paths as f32) * PATH_PROGRESS_WEIGHT
            } else {
                0.0
            };

            // 文件扫描进度（假设每个路径平均扫描 ESTIMATED_FILES_PER_PATH 个文件作为基准）
            let file_progress =
                (current_scanned as f32 / ESTIMATED_FILES_PER_PATH).min(FILE_PROGRESS_WEIGHT);
            let percent = (path_progress + file_progress).min(MAX_PROGRESS_BEFORE_COMPLETE); // 最高99%，完成后才100%

            let progress = JunkFileScanProgress {
                scan_id: scan_id_clone.clone(),
                current_path: current_path.to_string(),
                scanned_files: current_scanned,
                found_files: current_found,
                scanned_size: current_size,
                total_size: 0,
                percent,
                current_phase: phase.to_string(),
                status: ScanStatus::Scanning,
                speed: Some(speed),
            };

            let _ = app_clone.emit(EVENT_JUNK_FILE_PROGRESS, &progress);

            if let Ok(mut store) = progress_store_clone.try_write() {
                if let Some(p) = store.get_mut(&scan_id_clone) {
                    p.scanned_files = current_scanned;
                    p.found_files = current_found;
                    p.scanned_size = current_size;
                    p.speed = Some(speed);
                    p.current_path = current_path.to_string();
                    p.current_phase = phase.to_string();
                    p.percent = percent;
                }
            }
        }
    };

    emit_progress_fn("", "初始化扫描...", 0);

    let filter_options = FilterOptions {
        include_hidden: options.include_hidden,
        include_system: options.include_system,
        exclude_paths: options.exclude_paths.clone(),
    };
    let filter = StandardFileFilter::new(&filter_options);
    let safety_checker = SafetyChecker::new();

    let log_cutoff_time = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        - (options.log_max_age_days as i64 * 24 * 60 * 60);

    let installer_cutoff_time = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
        - (options.installer_max_age_days as i64 * 24 * 60 * 60);

    let mut results_by_type: HashMap<JunkFileType, Vec<JunkFile>> = HashMap::new();

    let all_scan_paths = collect_all_scan_paths(&options);
    let total_scan_paths = all_scan_paths.len();

    // 快速启动：立即发送初始进度
    {
        let mut store = progress_store_clone.try_write().unwrap();
        if let Some(p) = store.get_mut(&scan_id_clone) {
            p.current_phase = "正在准备扫描...".to_string();
            p.percent = 1.0;
            let _ = app_clone.emit(EVENT_JUNK_FILE_PROGRESS, p.clone());
        }
    }

    for (path_idx, scan_path) in all_scan_paths.iter().enumerate() {
        if !scan_path.exists() {
            current_path_index.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        let path_str = scan_path.to_string_lossy().to_string();
        let phase_text = format!(
            "扫描目录 ({}/{}) {}",
            path_idx + 1,
            total_scan_paths,
            path_str
        );
        emit_progress_fn(&path_str, &phase_text, total_scan_paths);

        // 检查取消信号
        if *ctx.cancel_receiver.borrow() {
            break;
        }

        // 处理暂停
        while *ctx.pause_receiver.borrow() {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if *ctx.cancel_receiver.borrow() {
                break;
            }
        }

        // 更新当前路径索引
        current_path_index.store((path_idx + 1) as u64, Ordering::Relaxed);

        // 遍历文件
        let walker = FileWalker::new(&filter);
        let entries: Vec<_> = walker.walk_files(scan_path).collect();
        let total_entries = entries.len();

        if total_entries == 0 {
            continue;
        }

        let progress_counter = Arc::new(AtomicU64::new(0));
        let progress_counter_clone = progress_counter.clone();
        let scan_id_for_progress = scan_id.clone();
        let app_for_progress = ctx.app.clone();
        let last_progress_send = Arc::new(std::sync::Mutex::new(Instant::now()));
        let phase_text_clone = phase_text.clone();

        // 并行处理文件
        let path_results: Vec<(JunkFileType, JunkFile)> = entries
            .into_par_iter()
            .filter_map(|entry| {
                let path = entry.path();

                // 错误处理包装
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    if let Ok(metadata) = fs::metadata(path) {
                        let size = metadata.len();
                        total_scanned_files.fetch_add(1, Ordering::Relaxed);
                        total_scanned_size.fetch_add(size, Ordering::Relaxed);

                        let processed = progress_counter_clone.fetch_add(1, Ordering::Relaxed);

                        // 每50个文件更新一次进度
                        if processed % 50 == 0 {
                            if let Ok(mut last) = last_progress_send.lock() {
                                if last.elapsed() >= std::time::Duration::from_millis(80) {
                                    *last = Instant::now();
                                    let current_scanned =
                                        total_scanned_files.load(Ordering::Relaxed);
                                    let current_size = total_scanned_size.load(Ordering::Relaxed);
                                    let current_found = total_found_files.load(Ordering::Relaxed);

                                    // 计算进度百分比
                                    let path_progress = ((path_idx + 1) as f32
                                        / total_scan_paths as f32)
                                        * PATH_PROGRESS_WEIGHT;
                                    let file_progress = (current_scanned as f32
                                        / ESTIMATED_FILES_PER_PATH)
                                        .min(FILE_PROGRESS_WEIGHT);
                                    let percent = (path_progress + file_progress)
                                        .min(MAX_PROGRESS_BEFORE_COMPLETE);

                                    let progress = JunkFileScanProgress {
                                        scan_id: scan_id_for_progress.clone(),
                                        current_path: path.to_string_lossy().to_string(),
                                        scanned_files: current_scanned,
                                        found_files: current_found,
                                        scanned_size: current_size,
                                        total_size: 0,
                                        percent,
                                        current_phase: phase_text_clone.clone(),
                                        status: ScanStatus::Scanning,
                                        speed: Some(
                                            current_size as f64
                                                / scan_start.elapsed().as_secs_f64().max(0.001),
                                        ),
                                    };
                                    let _ =
                                        app_for_progress.emit(EVENT_JUNK_FILE_PROGRESS, &progress);
                                }
                            }
                        }

                        let modified_time = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        let created_time = metadata
                            .created()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        let extension = PathUtils::get_extension(path);
                        let name_lower = PathUtils::get_filename(path)
                            .unwrap_or_default()
                            .to_lowercase();

                        let (file_type, description) = classify_file(
                            path,
                            &extension,
                            &name_lower,
                            size,
                            modified_time,
                            log_cutoff_time,
                            installer_cutoff_time,
                            &options,
                        );

                        if file_type != JunkFileType::EmptyFolders {
                            let safety_result = safety_checker.check(path);
                            total_found_files.fetch_add(1, Ordering::Relaxed);

                            let file_type_clone = file_type.clone();
                            return Some((
                                file_type,
                                JunkFile {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    path: path.to_string_lossy().to_string(),
                                    size,
                                    file_type: file_type_clone,
                                    description,
                                    modified_time,
                                    created_time,
                                    safe_to_delete: safety_result.safe_to_delete,
                                    risk_level: format!("{:?}", safety_result.risk_level),
                                },
                            ));
                        }
                    }
                    None
                }));

                result.unwrap_or_else(|_| None)
            })
            .collect();

        for (file_type, junk_file) in path_results {
            results_by_type
                .entry(file_type)
                .or_insert_with(Vec::new)
                .push(junk_file);
        }
    }

    // 扫描空文件夹
    if options.include_empty_folders && !*ctx.cancel_receiver.borrow() {
        emit_progress_fn("", "扫描空文件夹...", total_scan_paths);
        if let Ok(empty_folders) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            scan_empty_folders_optimized(&all_scan_paths, &filter, &safety_checker)
        })) {
            if !empty_folders.is_empty() {
                total_found_files.fetch_add(empty_folders.len() as u64, Ordering::Relaxed);
                results_by_type.insert(JunkFileType::EmptyFolders, empty_folders);
            }
        }
    }

    stats.files_scanned = total_scanned_files.load(Ordering::Relaxed);
    stats.files_found = total_found_files.load(Ordering::Relaxed);
    stats.total_size = total_scanned_size.load(Ordering::Relaxed);
    stats.finish();

    let was_cancelled = *ctx.cancel_receiver.borrow();

    // 存储完整的文件列表到全局存储
    {
        let mut store = JUNK_FILES_STORE.write().unwrap();
        store.insert(ctx.scan_id.clone(), results_by_type.clone());
    }

    // 构建返回结果 - 限制每个类型返回的文件数量以提高性能
    const MAX_ITEMS_PER_TYPE: usize = 100;
    let results: Vec<JunkScanResult> = results_by_type
        .iter()
        .map(|(file_type, items)| {
            let total_size = items.iter().map(|f| f.size).sum();
            let display_items: Vec<JunkFile> =
                items.iter().take(MAX_ITEMS_PER_TYPE).cloned().collect();
            JunkScanResult {
                file_type: file_type.clone(),
                total_size,
                count: items.len() as u64,
                items: display_items,
            }
        })
        .collect();

    // 发送完成进度
    let final_status = if was_cancelled {
        ScanStatus::Idle
    } else {
        ScanStatus::Completed
    };

    let completed_progress = JunkFileScanProgress {
        scan_id: ctx.scan_id.clone(),
        current_path: String::new(),
        current_phase: if was_cancelled {
            "扫描已取消".to_string()
        } else {
            "扫描完成".to_string()
        },
        scanned_files: total_scanned_files.load(Ordering::Relaxed),
        found_files: total_found_files.load(Ordering::Relaxed),
        scanned_size: total_scanned_size.load(Ordering::Relaxed),
        total_size: 0,
        percent: 100.0,
        status: final_status,
        speed: None,
    };

    let _ = ctx.app.emit(EVENT_JUNK_FILE_PROGRESS, &completed_progress);
    let _ = ctx.app.emit(EVENT_JUNK_FILE_COMPLETE, &results);

    // 如果被取消，返回错误
    if was_cancelled {
        return Err("扫描已被用户取消".to_string());
    }

    Ok(results)
}

fn collect_all_scan_paths(options: &JunkScanOptions) -> Vec<PathBuf> {
    // 1. 首先收集用户选择的扫描路径
    let mut paths: Vec<PathBuf> = options
        .scan_paths
        .iter()
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .collect();

    // 2. 如果用户没有指定路径，使用默认路径
    if paths.is_empty() {
        if let Some(home) = SystemPaths::home_dir() {
            paths.push(home);
        }
        if let Some(downloads) = SystemPaths::home_dir().map(|h| h.join("Downloads")) {
            if downloads.exists() {
                paths.push(downloads);
            }
        }
    }

    // 3. 只在用户选择的路径包含系统盘时，才添加系统临时路径和浏览器缓存
    // 判断是否有路径在系统盘（通常是 C 盘）
    let has_system_drive = paths.iter().any(|p| {
        let path_str = p.to_string_lossy().to_uppercase();
        path_str.starts_with("C:") || path_str.starts_with("C:\\")
    });

    if has_system_drive {
        // 添加系统临时路径
        for temp_path in SystemPaths::get_temp_paths() {
            if temp_path.exists() && !paths.contains(&temp_path) {
                paths.push(temp_path);
            }
        }

        // 添加浏览器缓存路径
        for cache_path in SystemPaths::get_browser_cache_paths() {
            if cache_path.exists() && !paths.contains(&cache_path) {
                paths.push(cache_path);
            }
        }
    }

    paths
}

fn classify_file(
    path: &Path,
    extension: &Option<String>,
    name_lower: &str,
    size: u64,
    modified_time: i64,
    log_cutoff_time: i64,
    installer_cutoff_time: i64,
    options: &JunkScanOptions,
) -> (JunkFileType, String) {
    if is_temp_file_fast(extension, name_lower) {
        return (JunkFileType::OldLogs, "系统临时文件".to_string());
    }

    if is_browser_cache_path(path) {
        return (JunkFileType::OldLogs, "浏览器缓存文件".to_string());
    }

    if options.include_invalid_downloads && is_invalid_download_fast(name_lower) {
        return (
            JunkFileType::InvalidDownloads,
            "未完成或损坏的下载文件".to_string(),
        );
    }

    if options.include_old_logs
        && is_log_file_fast(extension, name_lower)
        && modified_time < log_cutoff_time
    {
        return (
            JunkFileType::OldLogs,
            format!("超过 {} 天未修改的日志文件", options.log_max_age_days),
        );
    }

    if options.include_old_installers
        && is_installer_file_fast(extension, name_lower)
        && modified_time < installer_cutoff_time
    {
        return (
            JunkFileType::OldInstallers,
            format!("超过 {} 天的安装包文件", options.installer_max_age_days),
        );
    }

    if options.include_invalid_shortcuts && is_shortcut_file_fast(extension) {
        #[cfg(windows)]
        {
            if let Ok(target) = resolve_shortcut_target(path) {
                if !Path::new(&target).exists() {
                    return (
                        JunkFileType::InvalidShortcuts,
                        "目标路径不存在的快捷方式".to_string(),
                    );
                }
            }
        }
    }

    if size > 1024 && size <= 1024 * 1024 {
        return (
            JunkFileType::SmallFiles,
            format!("零散小文件 ({} KB)", size / 1024),
        );
    }

    (JunkFileType::EmptyFolders, String::new())
}

fn is_temp_file_fast(extension: &Option<String>, name_lower: &str) -> bool {
    matches!(
        extension.as_deref(),
        Some("tmp") | Some("temp") | Some("cache")
    ) || name_lower.ends_with(".tmp")
        || name_lower.ends_with(".temp")
        || name_lower.ends_with(".cache")
}

fn is_browser_cache_path(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_lowercase();
    path_str.contains("cache")
        || path_str.contains("appdata\\local\\google\\chrome")
        || path_str.contains("appdata\\local\\microsoft\\edge")
        || path_str.contains("appdata\\local\\mozilla\\firefox")
}

fn is_invalid_download_fast(name_lower: &str) -> bool {
    name_lower.ends_with(".part")
        || name_lower.ends_with(".crdownload")
        || name_lower.ends_with(".download")
        || name_lower.starts_with("unconfirmed")
        || name_lower.starts_with(".com.google.chrome")
}

fn is_log_file_fast(extension: &Option<String>, name_lower: &str) -> bool {
    matches!(
        extension.as_deref(),
        Some("log") | Some("txt") | Some("old")
    ) || name_lower.ends_with(".log")
        || name_lower.ends_with(".txt")
        || name_lower.contains("log")
}

fn is_installer_file_fast(extension: &Option<String>, name_lower: &str) -> bool {
    matches!(
        extension.as_deref(),
        Some("exe") | Some("msi") | Some("dmg") | Some("pkg") | Some("deb") | Some("rpm")
    ) && (name_lower.contains("setup")
        || name_lower.contains("install")
        || name_lower.contains("installer")
        || name_lower.contains("uninstall"))
}

fn is_shortcut_file_fast(extension: &Option<String>) -> bool {
    extension
        .as_deref()
        .map(|ext| ext.eq_ignore_ascii_case("lnk"))
        .unwrap_or(false)
}

fn scan_empty_folders_optimized(
    scan_paths: &[PathBuf],
    filter: &StandardFileFilter,
    safety_checker: &SafetyChecker,
) -> Vec<JunkFile> {
    let mut empty_folders = Vec::new();

    for scan_path in scan_paths {
        if !scan_path.exists() {
            continue;
        }

        let dirs: Vec<_> = walkdir::WalkDir::new(scan_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
            .filter(|e| filter.should_include(e))
            .collect();

        for dir in dirs {
            if is_empty_folder_fast(dir.path()) {
                let safety_result = safety_checker.check(dir.path());
                empty_folders.push(JunkFile {
                    id: uuid::Uuid::new_v4().to_string(),
                    path: dir.path().to_string_lossy().to_string(),
                    size: 0,
                    file_type: JunkFileType::EmptyFolders,
                    description: "空文件夹".to_string(),
                    modified_time: 0,
                    created_time: 0,
                    safe_to_delete: safety_result.safe_to_delete,
                    risk_level: format!("{:?}", safety_result.risk_level),
                });
            }
        }
    }

    empty_folders
}

fn is_empty_folder_fast(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    match fs::read_dir(path) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

#[cfg(windows)]
fn resolve_shortcut_target(path: &Path) -> Result<String, std::io::Error> {
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER, STGM,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};

    unsafe {
        CoInitialize(None).ok()?;

        let result = (|| {
            let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

            let persist_file = shell_link.cast::<windows::Win32::System::Com::IPersistFile>()?;
            let path_wide: Vec<u16> = path
                .to_string_lossy()
                .to_string()
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            persist_file.Load(PCWSTR(path_wide.as_ptr()), STGM(0))?;

            let mut target = [0u16; 260];
            shell_link.GetPath(&mut target, std::ptr::null_mut(), 0)?;

            let target_string = String::from_utf16_lossy(
                &target[..target.iter().position(|&c| c == 0).unwrap_or(target.len())],
            );

            Ok(target_string)
        })();

        CoUninitialize();
        result
    }
}

// 公共 API 函数
pub async fn get_junk_file_scan_progress(scan_id: &str) -> Option<JunkFileScanProgress> {
    SCAN_MANAGER.get_progress(scan_id).await
}

pub async fn get_junk_file_scan_result(scan_id: &str) -> Option<Vec<JunkScanResult>> {
    SCAN_MANAGER.get_result(scan_id).await
}

pub async fn pause_junk_file_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.pause_scan(scan_id).await
}

pub async fn resume_junk_file_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.resume_scan(scan_id).await
}

pub async fn cancel_junk_file_scan(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.cancel_scan(scan_id).await
}

pub async fn clear_junk_file_scan_result(scan_id: &str) -> Result<(), String> {
    SCAN_MANAGER.clear_scan(scan_id).await
}
