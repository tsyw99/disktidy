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

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rayon::prelude::*;
use tauri::{AppHandle, Emitter};

use crate::modules::scanner_framework::{
    ControlAction, FileFilter, FileWalker, FilterOptions, ScanContext, ScanManager, ScanProgress,
    StandardFileFilter,
};

use crate::models::{ScanStatus, EVENT_JUNK_FILE_COMPLETE, EVENT_JUNK_FILE_PROGRESS};
use crate::modules::cleaner::safety::SafetyChecker;
use crate::modules::file_analyzer::{JunkFile, JunkFileType, JunkScanOptions, JunkScanResult};
use crate::utils::path::{PathUtils, SystemPaths};

// 进度更新间隔
const PROGRESS_UPDATE_INTERVAL: u64 = 50;

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
            current_phase: String::new(),
            status: ScanStatus::Idle,
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

/// 扫描阶段定义
#[derive(Debug, Clone, Copy, PartialEq)]
enum ScanPhase {
    TempFiles,
    EmptyFolders,
    InvalidShortcuts,
    OldLogs,
    OldInstallers,
    InvalidDownloads,
    SmallFiles,
    BrowserCache,
}

impl ScanPhase {
    fn name(&self) -> &'static str {
        match self {
            ScanPhase::TempFiles => "扫描临时文件",
            ScanPhase::EmptyFolders => "扫描空文件夹",
            ScanPhase::InvalidShortcuts => "扫描无效快捷方式",
            ScanPhase::OldLogs => "扫描过期日志",
            ScanPhase::OldInstallers => "扫描旧安装包",
            ScanPhase::InvalidDownloads => "扫描无效下载",
            ScanPhase::SmallFiles => "扫描零散小文件",
            ScanPhase::BrowserCache => "扫描浏览器缓存",
        }
    }
}

/// 全局扫描管理器
lazy_static::lazy_static! {
    static ref SCAN_MANAGER: ScanManager<JunkFileScanProgress, Vec<JunkScanResult>> = ScanManager::new();
}

/// 启动零碎文件扫描
pub async fn start_junk_file_scan(
    app: AppHandle,
    options: Option<JunkScanOptions>,
) -> Result<String, String> {
    let opts = options.unwrap_or_default();
    println!("[JunkFileScanner] Starting scan with options: {:?}", opts);
    let progress = JunkFileScanProgress::new("");

    SCAN_MANAGER
        .start_scan(app, progress, move |mut ctx| async move {
            perform_scan(&mut ctx, opts).await
        })
        .await
}

/// 执行扫描
async fn perform_scan(
    ctx: &mut ScanContext<JunkFileScanProgress>,
    options: JunkScanOptions,
) -> Result<Vec<JunkScanResult>, String> {
    let scanner = JunkFileScanner::new(options);
    let mut results = Vec::new();

    // 定义启用的扫描阶段
    let phases = scanner.get_enabled_phases();
    let total_phases = phases.len();

    for (idx, phase) in phases.iter().enumerate() {
        // 计算当前进度百分比
        let percent = if total_phases > 0 {
            ((idx as f32 / total_phases as f32) * 100.0).min(99.0)
        } else {
            0.0
        };

        // 更新阶段信息
        update_phase_progress(ctx, phase.name(), percent).await;

        // 检查控制信号
        match check_control_with_store(ctx).await {
            ControlAction::Cancel => break,
            ControlAction::Continue => {}
        }

        // 执行阶段扫描
        if let Some(result) = scanner.scan_phase(*phase, ctx).await {
            results.push(result);
        }
    }

    // 扫描完成，更新最终进度
    update_phase_progress(ctx, "扫描完成", 100.0).await;

    // 发送完成事件
    let _ = ctx.app.emit(EVENT_JUNK_FILE_COMPLETE, &results);

    Ok(results)
}

/// 更新阶段进度
async fn update_phase_progress(
    ctx: &ScanContext<JunkFileScanProgress>,
    phase_name: &str,
    percent: f32,
) {
    println!(
        "[JunkFileScanner] Phase progress: phase={}, percent={}",
        phase_name, percent
    );

    let progress = JunkFileScanProgress {
        scan_id: ctx.scan_id.clone(),
        current_phase: phase_name.to_string(),
        percent,
        status: ScanStatus::Scanning,
        ..ctx.progress.clone()
    };

    let _ = ctx.app.emit(EVENT_JUNK_FILE_PROGRESS, progress);
}

/// 检查控制信号
async fn check_control_with_store(ctx: &mut ScanContext<JunkFileScanProgress>) -> ControlAction {
    // 使用全局 SCAN_MANAGER 的状态
    if ctx.cancel_receiver.has_changed().unwrap_or(false) && *ctx.cancel_receiver.borrow() {
        return ControlAction::Cancel;
    }

    while *ctx.pause_receiver.borrow() {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        if *ctx.cancel_receiver.borrow() {
            return ControlAction::Cancel;
        }
    }

    ControlAction::Continue
}

/// 零碎文件扫描器
struct JunkFileScanner {
    options: JunkScanOptions,
    safety_checker: SafetyChecker,
    filter: StandardFileFilter,
}

impl JunkFileScanner {
    fn new(options: JunkScanOptions) -> Self {
        let filter_options = FilterOptions {
            include_hidden: options.include_hidden,
            include_system: options.include_system,
            exclude_paths: options.exclude_paths.clone(),
        };

        Self {
            options,
            safety_checker: SafetyChecker::new(),
            filter: StandardFileFilter::new(&filter_options),
        }
    }

    fn get_enabled_phases(&self) -> Vec<ScanPhase> {
        let mut phases = Vec::new();

        // 默认启用所有扫描阶段
        phases.push(ScanPhase::TempFiles);

        if self.options.include_empty_folders {
            phases.push(ScanPhase::EmptyFolders);
        }
        if self.options.include_invalid_shortcuts {
            phases.push(ScanPhase::InvalidShortcuts);
        }
        if self.options.include_old_logs {
            phases.push(ScanPhase::OldLogs);
        }
        if self.options.include_old_installers {
            phases.push(ScanPhase::OldInstallers);
        }
        if self.options.include_invalid_downloads {
            phases.push(ScanPhase::InvalidDownloads);
        }
        // 默认启用小文件扫描
        phases.push(ScanPhase::SmallFiles);
        phases.push(ScanPhase::BrowserCache);

        phases
    }

    async fn scan_phase(
        &self,
        phase: ScanPhase,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        match phase {
            ScanPhase::TempFiles => self.scan_temp_files(ctx).await,
            ScanPhase::EmptyFolders => self.scan_empty_folders(ctx).await,
            ScanPhase::InvalidShortcuts => self.scan_invalid_shortcuts(ctx).await,
            ScanPhase::OldLogs => self.scan_old_logs(ctx).await,
            ScanPhase::OldInstallers => self.scan_old_installers(ctx).await,
            ScanPhase::InvalidDownloads => self.scan_invalid_downloads(ctx).await,
            ScanPhase::SmallFiles => self.scan_small_files(ctx).await,
            ScanPhase::BrowserCache => self.scan_browser_cache(ctx).await,
        }
    }

    /// 扫描临时文件
    async fn scan_temp_files(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        // 获取所有临时目录
        let temp_paths = SystemPaths::get_temp_paths();

        for temp_path in &temp_paths {
            if !temp_path.exists() {
                continue;
            }

            // 收集临时文件
            let temp_files: Vec<_> = walker
                .walk_files(temp_path)
                .filter(|e| self.is_temp_file(e.path()))
                .collect();

            // 并行处理
            let junk_files: Vec<_> = temp_files
                .into_par_iter()
                .filter_map(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let size = metadata.len();
                        return self.create_junk_file(
                            e.path(),
                            size,
                            JunkFileType::OldLogs,
                            "系统临时文件",
                        );
                    }
                    None
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 100 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, temp_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::OldLogs,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    async fn scan_empty_folders(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let mut items = Vec::new();

        let scan_paths = self.get_scan_paths();
        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            // 使用 walkdir 直接遍历，不过滤根目录
            let empty_dirs: Vec<_> = walkdir::WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_dir())
                .filter(|e| self.filter.should_include(e)) // 应用过滤器
                .filter(|e| self.is_empty_folder(e.path()))
                .collect();

            // 并行处理
            let junk_files: Vec<_> = empty_dirs
                .into_par_iter()
                .filter_map(|e| {
                    self.create_junk_file(e.path(), 0, JunkFileType::EmptyFolders, "空文件夹")
                })
                .collect();

            items.extend(junk_files);

            // 更新进度
            if items.len() % 100 == 0 {
                self.emit_progress(ctx, scan_path, items.len() as u64, 0)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        Some(JunkScanResult {
            file_type: JunkFileType::EmptyFolders,
            total_size: 0,
            count: items.len() as u64,
            items,
        })
    }

    async fn scan_invalid_shortcuts(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        let scan_paths = self.get_scan_paths();
        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            // 收集所有快捷方式文件
            let shortcut_files: Vec<_> = walker
                .walk_files(scan_path)
                .filter(|e| self.is_shortcut_file(e.path()))
                .collect();

            // 并行处理检查有效性
            let junk_files: Vec<_> = shortcut_files
                .into_par_iter()
                .filter_map(|e| {
                    if self.is_invalid_shortcut(e.path()) {
                        let size = fs::metadata(e.path()).map(|m| m.len()).unwrap_or(0);
                        self.create_junk_file(
                            e.path(),
                            size,
                            JunkFileType::InvalidShortcuts,
                            "目标路径不存在的快捷方式",
                        )
                    } else {
                        None
                    }
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 100 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, scan_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::InvalidShortcuts,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    async fn scan_old_logs(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let cutoff_time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - (self.options.log_max_age_days as i64 * 24 * 60 * 60);

        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        let scan_paths = self.get_scan_paths();
        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            // 收集候选日志文件
            let log_files: Vec<_> = walker
                .walk_files(scan_path)
                .filter(|e| self.is_log_file(e.path()))
                .collect();

            // 并行处理检查过期
            let junk_files: Vec<_> = log_files
                .into_par_iter()
                .filter_map(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let modified_time = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        if modified_time < cutoff_time {
                            let size = metadata.len();
                            return self.create_junk_file(
                                e.path(),
                                size,
                                JunkFileType::OldLogs,
                                &format!(
                                    "超过 {} 天未修改的日志文件",
                                    self.options.log_max_age_days
                                ),
                            );
                        }
                    }
                    None
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 100 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, scan_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::OldLogs,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    async fn scan_old_installers(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let cutoff_time = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - (self.options.installer_max_age_days as i64 * 24 * 60 * 60);

        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        let scan_paths = self.get_scan_paths();
        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            // 收集候选安装包文件
            let installer_files: Vec<_> = walker
                .walk_files(scan_path)
                .filter(|e| self.is_installer_file(e.path()))
                .collect();

            // 并行处理检查过期
            let junk_files: Vec<_> = installer_files
                .into_par_iter()
                .filter_map(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let modified_time = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        if modified_time < cutoff_time {
                            let size = metadata.len();
                            return self.create_junk_file(
                                e.path(),
                                size,
                                JunkFileType::OldInstallers,
                                &format!(
                                    "超过 {} 天的安装包文件",
                                    self.options.installer_max_age_days
                                ),
                            );
                        }
                    }
                    None
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 100 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, scan_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::OldInstallers,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    async fn scan_invalid_downloads(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        // 主要扫描下载目录
        let download_paths = self.get_download_paths();

        for download_path in &download_paths {
            if !download_path.exists() {
                continue;
            }

            // 收集候选下载文件
            let download_files: Vec<_> = walker
                .walk_files(download_path)
                .filter(|e| self.is_invalid_download(e.path()))
                .collect();

            // 并行处理
            let junk_files: Vec<_> = download_files
                .into_par_iter()
                .filter_map(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let size = metadata.len();
                        return self.create_junk_file(
                            e.path(),
                            size,
                            JunkFileType::InvalidDownloads,
                            "未完成或损坏的下载文件",
                        );
                    }
                    None
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 100 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, download_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::InvalidDownloads,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    async fn scan_small_files(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        // 扫描下载目录和文档目录中的小文件
        let scan_paths = self.get_small_file_scan_paths();

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            // 收集所有小文件（小于 1MB）
            let small_files: Vec<_> = walker
                .walk_files(scan_path)
                .filter(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let size = metadata.len();
                        // 小文件定义：1KB - 1MB 之间的文件
                        size > 1024 && size <= 1024 * 1024
                    } else {
                        false
                    }
                })
                .collect();

            // 并行处理创建 JunkFile
            let junk_files: Vec<_> = small_files
                .into_par_iter()
                .filter_map(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let size = metadata.len();
                        return self.create_junk_file(
                            e.path(),
                            size,
                            JunkFileType::SmallFiles,
                            &format!("零散小文件 ({} KB)", size / 1024),
                        );
                    }
                    None
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 1000 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, scan_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::SmallFiles,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    /// 扫描浏览器缓存
    async fn scan_browser_cache(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
    ) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let walker = FileWalker::new(&self.filter);

        // 获取浏览器缓存路径
        let cache_paths = SystemPaths::get_browser_cache_paths();

        for cache_path in &cache_paths {
            if !cache_path.exists() {
                continue;
            }

            // 收集缓存文件
            let cache_files: Vec<_> = walker.walk_files(cache_path).collect();

            // 并行处理
            let junk_files: Vec<_> = cache_files
                .into_par_iter()
                .filter_map(|e| {
                    if let Ok(metadata) = fs::metadata(e.path()) {
                        let size = metadata.len();
                        return self.create_junk_file(
                            e.path(),
                            size,
                            JunkFileType::OldLogs,
                            "浏览器缓存文件",
                        );
                    }
                    None
                })
                .collect();

            items.extend(junk_files);

            if items.len() % 100 == 0 {
                let total_size = items.iter().map(|f| f.size).sum();
                self.emit_progress(ctx, cache_path, items.len() as u64, total_size)
                    .await;
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size = items.iter().map(|f| f.size).sum();
        Some(JunkScanResult {
            file_type: JunkFileType::OldLogs,
            total_size,
            count: items.len() as u64,
            items,
        })
    }

    async fn emit_progress(
        &self,
        ctx: &ScanContext<JunkFileScanProgress>,
        path: &Path,
        found: u64,
        size: u64,
    ) {
        let progress = JunkFileScanProgress {
            scan_id: ctx.scan_id.clone(),
            current_path: path.to_string_lossy().to_string(),
            found_files: found,
            scanned_size: size,
            status: ScanStatus::Scanning,
            ..ctx.progress.clone()
        };

        println!(
            "[JunkFileScanner] Emitting progress: scanId={}, currentPath={}, scannedFiles={}, foundFiles={}, scannedSize={}, status={:?}",
            progress.scan_id,
            progress.current_path,
            progress.scanned_files,
            progress.found_files,
            progress.scanned_size,
            progress.status
        );

        let _ = ctx.app.emit(EVENT_JUNK_FILE_PROGRESS, &progress);
    }

    fn get_scan_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self
            .options
            .scan_paths
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect();

        // 如果没有指定路径，扫描常用目录
        if paths.is_empty() {
            if let Some(home) = SystemPaths::home_dir() {
                paths.push(home);
            }
            // 添加下载目录
            if let Some(downloads) = SystemPaths::home_dir().map(|h| h.join("Downloads")) {
                if downloads.exists() {
                    paths.push(downloads);
                }
            }
            // 添加文档目录
            if let Some(documents) = SystemPaths::home_dir().map(|h| h.join("Documents")) {
                if documents.exists() {
                    paths.push(documents);
                }
            }
        }

        paths
    }

    fn get_small_file_scan_paths(&self) -> Vec<PathBuf> {
        // 优先使用用户指定的扫描路径
        let mut paths: Vec<PathBuf> = self
            .options
            .scan_paths
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect();

        // 如果没有指定路径，默认扫描下载目录
        if paths.is_empty() {
            if let Some(home) = SystemPaths::home_dir() {
                let downloads = home.join("Downloads");
                if downloads.exists() {
                    paths.push(downloads);
                }
            }
        }

        paths
    }

    fn get_download_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(home) = SystemPaths::home_dir() {
            paths.push(home.join("Downloads"));
        }

        paths.retain(|p| p.exists());
        paths
    }

    fn is_empty_folder(&self, path: &Path) -> bool {
        if !path.is_dir() {
            return false;
        }

        match fs::read_dir(path) {
            Ok(mut entries) => entries.next().is_none(),
            Err(_) => false,
        }
    }

    fn is_shortcut_file(&self, path: &Path) -> bool {
        PathUtils::get_extension(path)
            .map(|ext| ext.eq_ignore_ascii_case("lnk"))
            .unwrap_or(false)
    }

    fn is_invalid_shortcut(&self, path: &Path) -> bool {
        if !self.is_shortcut_file(path) {
            return false;
        }

        #[cfg(windows)]
        {
            if let Ok(target) = resolve_shortcut_target(path) {
                return !Path::new(&target).exists();
            }
        }

        false
    }

    fn is_log_file(&self, path: &Path) -> bool {
        let extension = PathUtils::get_extension(path);
        let name = PathUtils::get_filename(path)
            .unwrap_or_default()
            .to_lowercase();

        matches!(
            extension.as_deref(),
            Some("log") | Some("txt") | Some("old")
        ) || name.ends_with(".log")
            || name.ends_with(".txt")
            || name.contains("log")
    }

    fn is_temp_file(&self, path: &Path) -> bool {
        let extension = PathUtils::get_extension(path);
        let name = PathUtils::get_filename(path)
            .unwrap_or_default()
            .to_lowercase();

        matches!(
            extension.as_deref(),
            Some("tmp") | Some("temp") | Some("cache")
        ) || name.ends_with(".tmp")
            || name.ends_with(".temp")
            || name.ends_with(".cache")
    }

    fn is_installer_file(&self, path: &Path) -> bool {
        let extension = PathUtils::get_extension(path);
        let name = PathUtils::get_filename(path)
            .unwrap_or_default()
            .to_lowercase();

        matches!(
            extension.as_deref(),
            Some("exe") | Some("msi") | Some("dmg") | Some("pkg") | Some("deb") | Some("rpm")
        ) && (name.contains("setup")
            || name.contains("install")
            || name.contains("installer")
            || name.contains("uninstall"))
    }

    fn is_invalid_download(&self, path: &Path) -> bool {
        let name = PathUtils::get_filename(path)
            .unwrap_or_default()
            .to_lowercase();

        name.ends_with(".part")
            || name.ends_with(".crdownload")
            || name.ends_with(".download")
            || name.ends_with(".tmp")
            || name.ends_with(".temp")
            || name.starts_with("unconfirmed")
            || name.starts_with(".com.google.chrome")
    }

    fn create_junk_file(
        &self,
        path: &Path,
        size: u64,
        file_type: JunkFileType,
        description: &str,
    ) -> Option<JunkFile> {
        let metadata = fs::metadata(path).ok()?;
        let safety_result = self.safety_checker.check(path);

        Some(JunkFile {
            id: uuid::Uuid::new_v4().to_string(),
            path: path.to_string_lossy().to_string(),
            size,
            file_type: file_type.clone(),
            description: description.to_string(),
            modified_time: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            created_time: metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            safe_to_delete: safety_result.safe_to_delete,
            risk_level: format!("{:?}", safety_result.risk_level),
        })
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
