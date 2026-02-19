//! 统一的文件扫描框架
//!
//! 提供可复用的文件扫描基础设施，包括：
//! - 扫描控制（暂停/继续/取消）
//! - 进度跟踪和事件发送
//! - 文件遍历和过滤
//! - 并发管理

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, RwLock};
use walkdir::{DirEntry, WalkDir};

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use crate::models::{generate_scan_id, ScanStatus};
use crate::utils::path::SystemPaths;

/// 扫描控制句柄，用于控制扫描过程
#[derive(Debug)]
pub struct ScanController {
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

    pub fn pause(&self) {
        let _ = self.pause_sender.send(true);
    }

    pub fn resume(&self) {
        let _ = self.pause_sender.send(false);
    }

    pub fn cancel(&self) {
        let _ = self.cancel_sender.send(true);
    }
}

/// 扫描上下文，包含扫描过程中的共享状态
pub struct ScanContext<P: Clone + Send + 'static> {
    pub scan_id: String,
    pub app: AppHandle,
    pub pause_receiver: watch::Receiver<bool>,
    pub cancel_receiver: watch::Receiver<bool>,
    pub progress: P,
    pub start_instant: Instant,
    last_update: std::sync::atomic::AtomicU64,
}

impl<P: Clone + Send + 'static + serde::Serialize> ScanContext<P> {
    fn new(
        scan_id: String,
        app: AppHandle,
        pause_receiver: watch::Receiver<bool>,
        cancel_receiver: watch::Receiver<bool>,
        progress: P,
    ) -> Self {
        Self {
            scan_id,
            app,
            pause_receiver,
            cancel_receiver,
            progress,
            start_instant: Instant::now(),
            last_update: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// 检查是否需要暂停或取消
    pub async fn check_control(
        &mut self,
        progress_store: &Arc<RwLock<HashMap<String, P>>>,
    ) -> ControlAction
    where
        P: ScanProgress,
    {
        if *self.cancel_receiver.borrow() {
            return ControlAction::Cancel;
        }

        while *self.pause_receiver.borrow() {
            {
                let mut store = progress_store.write().await;
                if let Some(p) = store.get_mut(&self.scan_id) {
                    p.set_status(ScanStatus::Paused);
                    let _ = self.app.emit(P::event_name(), p.clone());
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if *self.cancel_receiver.borrow() {
                return ControlAction::Cancel;
            }
        }

        ControlAction::Continue
    }

    /// 更新进度（带节流）
    pub async fn update_progress<F>(
        &self,
        update_fn: F,
        progress_store: &Arc<RwLock<HashMap<String, P>>>,
        interval: u64,
    ) where
        F: FnOnce(&mut P),
        P: ScanProgress + serde::Serialize + Clone,
    {
        let current = self.last_update.load(std::sync::atomic::Ordering::Relaxed);
        if current % interval != 0 {
            self.last_update
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            return;
        }

        let mut store = progress_store.write().await;
        if let Some(progress) = store.get_mut(&self.scan_id) {
            update_fn(progress);
            progress.set_status(ScanStatus::Scanning);
            let _ = self.app.emit(P::event_name(), progress.clone());
        }
    }

    pub fn increment_counter(&self) {
        self.last_update
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlAction {
    Continue,
    Cancel,
}

/// 扫描进度 trait，定义进度类型的通用接口
pub trait ScanProgress: Clone + Send + Sync + serde::Serialize {
    fn set_status(&mut self, status: ScanStatus);
    fn event_name() -> &'static str;
}

/// 文件过滤器 trait
pub trait FileFilter: Send + Sync {
    fn should_include(&self, entry: &DirEntry) -> bool;
}

/// 标准文件过滤器
pub struct StandardFileFilter {
    pub include_hidden: bool,
    pub include_system: bool,
    pub exclude_paths: Vec<String>,
    pub protected_paths: Vec<PathBuf>,
}

impl StandardFileFilter {
    pub fn new(options: &FilterOptions) -> Self {
        Self {
            include_hidden: options.include_hidden,
            include_system: options.include_system,
            exclude_paths: options.exclude_paths.clone(),
            protected_paths: SystemPaths::get_protected_paths(),
        }
    }
}

impl FileFilter for StandardFileFilter {
    fn should_include(&self, entry: &DirEntry) -> bool {
        let path = entry.path();
        let path_str = path.to_string_lossy();

        // 检查排除路径
        for exclude in &self.exclude_paths {
            if path_str.to_lowercase().starts_with(&exclude.to_lowercase()) {
                return false;
            }
        }

        // 检查受保护路径
        for protected in &self.protected_paths {
            if path.starts_with(protected) {
                return false;
            }
        }

        // 对于目录：只检查排除路径和受保护路径，不检查隐藏/系统属性
        // 因为 filter_entry 对目录返回 false 会跳过整个目录树
        if entry.file_type().is_dir() {
            return true;
        }

        // 对于文件：检查隐藏文件和系统文件
        #[cfg(windows)]
        {
            if let Ok(metadata) = entry.metadata() {
                let attrs = metadata.file_attributes();

                if !self.include_hidden && (attrs & 0x2) != 0 {
                    return false;
                }

                if !self.include_system && (attrs & 0x4) != 0 {
                    return false;
                }
            }
        }

        #[cfg(not(windows))]
        {
            if !self.include_hidden {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') {
                        return false;
                    }
                }
            }
        }

        true
    }
}

/// 过滤选项
#[derive(Debug, Clone)]
pub struct FilterOptions {
    pub include_hidden: bool,
    pub include_system: bool,
    pub exclude_paths: Vec<String>,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            include_system: false,
            exclude_paths: vec![],
        }
    }
}

/// 扫描管理器，管理所有扫描任务
pub struct ScanManager<P: ScanProgress + Clone + Send + 'static, R: Clone + Send + 'static> {
    progress_store: Arc<RwLock<HashMap<String, P>>>,
    result_store: Arc<RwLock<HashMap<String, R>>>,
    controllers: Arc<RwLock<HashMap<String, ScanController>>>,
}

impl<
        P: ScanProgress + Clone + Send + 'static + serde::Serialize,
        R: Clone + Send + Sync + 'static,
    > ScanManager<P, R>
{
    pub fn new() -> Self {
        Self {
            progress_store: Arc::new(RwLock::new(HashMap::new())),
            result_store: Arc::new(RwLock::new(HashMap::new())),
            controllers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 启动新的扫描任务
    pub async fn start_scan<F, Fut>(
        &self,
        app: AppHandle,
        initial_progress: P,
        scan_fn: F,
    ) -> Result<String, String>
    where
        F: FnOnce(ScanContext<P>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, String>> + Send + 'static,
    {
        let scan_id = generate_scan_id();
        self.start_scan_with_id(app, scan_id, initial_progress, scan_fn)
            .await
    }

    /// 使用指定的 scan_id 启动扫描任务
    pub async fn start_scan_with_id<F, Fut>(
        &self,
        app: AppHandle,
        scan_id: String,
        initial_progress: P,
        scan_fn: F,
    ) -> Result<String, String>
    where
        F: FnOnce(ScanContext<P>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<R, String>> + Send + 'static,
    {
        // 初始化进度
        self.progress_store
            .write()
            .await
            .insert(scan_id.clone(), initial_progress.clone());

        // 创建控制器
        let (controller, pause_receiver, cancel_receiver) = ScanController::new();
        self.controllers
            .write()
            .await
            .insert(scan_id.clone(), controller);

        // 发送初始进度
        let _ = app.emit(P::event_name(), &initial_progress);

        // 创建上下文
        let context = ScanContext::new(
            scan_id.clone(),
            app.clone(),
            pause_receiver,
            cancel_receiver,
            initial_progress,
        );

        let progress_store = self.progress_store.clone();
        let result_store = self.result_store.clone();
        let controllers = self.controllers.clone();
        let scan_id_clone = scan_id.clone();

        // 启动扫描任务
        tokio::spawn(async move {
            match scan_fn(context).await {
                Ok(result) => {
                    // 存储结果
                    result_store
                        .write()
                        .await
                        .insert(scan_id_clone.clone(), result);
                }
                Err(e) => {
                    eprintln!("Scan error for {}: {}", scan_id_clone, e);
                    let mut store = progress_store.write().await;
                    if let Some(progress) = store.get_mut(&scan_id_clone) {
                        progress.set_status(ScanStatus::Error);
                        let _ = app.emit(P::event_name(), progress.clone());
                    }
                }
            }

            // 清理控制器
            controllers.write().await.remove(&scan_id_clone);
        });

        Ok(scan_id)
    }

    pub async fn pause_scan(&self, scan_id: &str) -> Result<(), String> {
        let controllers = self.controllers.read().await;
        if let Some(controller) = controllers.get(scan_id) {
            controller.pause();
        }

        let mut store = self.progress_store.write().await;
        if let Some(progress) = store.get_mut(scan_id) {
            progress.set_status(ScanStatus::Paused);
        }
        Ok(())
    }

    pub async fn resume_scan(&self, scan_id: &str) -> Result<(), String> {
        let controllers = self.controllers.read().await;
        if let Some(controller) = controllers.get(scan_id) {
            controller.resume();
        }

        let mut store = self.progress_store.write().await;
        if let Some(progress) = store.get_mut(scan_id) {
            progress.set_status(ScanStatus::Scanning);
        }
        Ok(())
    }

    pub async fn cancel_scan(&self, scan_id: &str) -> Result<(), String> {
        let controllers = self.controllers.read().await;
        if let Some(controller) = controllers.get(scan_id) {
            controller.cancel();
        }

        let mut store = self.progress_store.write().await;
        if let Some(progress) = store.get_mut(scan_id) {
            progress.set_status(ScanStatus::Idle);
        }
        Ok(())
    }

    pub async fn get_progress(&self, scan_id: &str) -> Option<P> {
        self.progress_store.read().await.get(scan_id).cloned()
    }

    pub async fn get_result(&self, scan_id: &str) -> Option<R> {
        self.result_store.read().await.get(scan_id).cloned()
    }

    pub async fn clear_scan(&self, scan_id: &str) -> Result<(), String> {
        self.progress_store.write().await.remove(scan_id);
        self.result_store.write().await.remove(scan_id);
        Ok(())
    }

    /// 获取进度存储的引用（用于 check_control）
    pub fn get_progress_store(&self) -> Arc<RwLock<HashMap<String, P>>> {
        self.progress_store.clone()
    }
}

impl<
        P: ScanProgress + Clone + Send + 'static + serde::Serialize,
        R: Clone + Send + Sync + 'static,
    > Default for ScanManager<P, R>
{
    fn default() -> Self {
        Self::new()
    }
}

/// 文件遍历器，提供高效的文件遍历
pub struct FileWalker<'a> {
    filter: &'a dyn FileFilter,
    follow_links: bool,
}

impl<'a> FileWalker<'a> {
    pub fn new(filter: &'a dyn FileFilter) -> Self {
        Self {
            filter,
            follow_links: false,
        }
    }

    pub fn follow_links(mut self, follow: bool) -> Self {
        self.follow_links = follow;
        self
    }

    pub fn walk(&self, path: &Path) -> impl Iterator<Item = Result<DirEntry, walkdir::Error>> + '_ {
        WalkDir::new(path)
            .follow_links(self.follow_links)
            .into_iter()
            .filter_entry(|e| self.filter.should_include(e))
    }

    pub fn walk_files(&self, path: &Path) -> impl Iterator<Item = DirEntry> + '_ {
        self.walk(path)
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
    }

    pub fn walk_dirs(&self, path: &Path) -> impl Iterator<Item = DirEntry> + '_ {
        self.walk(path)
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
    }
}

/// 并行文件处理器
pub struct ParallelFileProcessor;

impl ParallelFileProcessor {
    /// 使用 rayon 并行处理文件列表
    pub fn process_files<T, F>(files: Vec<PathBuf>, processor: F) -> Vec<T>
    where
        F: Fn(&Path) -> Option<T> + Sync,
        T: Send,
    {
        use rayon::prelude::*;

        files
            .par_iter()
            .filter_map(|path| processor(path))
            .collect()
    }
}

/// 扫描统计信息
#[derive(Debug, Clone, Default)]
pub struct ScanStatistics {
    pub files_scanned: u64,
    pub files_found: u64,
    pub total_size: u64,
    pub errors: u64,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

impl ScanStatistics {
    pub fn new() -> Self {
        Self {
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            ..Default::default()
        }
    }

    pub fn finish(&mut self) {
        self.end_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
        );
    }

    pub fn duration_ms(&self) -> u64 {
        match self.end_time {
            Some(end) => end - self.start_time,
            None => {
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
                    - self.start_time
            }
        }
    }

    pub fn speed_bytes_per_sec(&self) -> f64 {
        let duration_sec = self.duration_ms() as f64 / 1000.0;
        if duration_sec > 0.0 {
            self.total_size as f64 / duration_sec
        } else {
            0.0
        }
    }
}

/// 错误处理工具
pub mod error_handling {
    use super::*;

    /// 检查错误是否是权限错误（可以忽略）
    pub fn is_permission_error(error: &walkdir::Error) -> bool {
        let error_str = error.to_string().to_lowercase();
        error_str.contains("拒绝访问")
            || error_str.contains("access is denied")
            || error_str.contains("permission denied")
            || error_str.contains("not found")
            || error_str.contains("找不到")
    }

    /// 检查错误是否是可恢复的（可以继续扫描）
    pub fn is_recoverable_error(error: &walkdir::Error) -> bool {
        is_permission_error(error)
            || error.io_error().map(|e| e.kind()).map_or(false, |kind| {
                matches!(
                    kind,
                    std::io::ErrorKind::NotFound
                        | std::io::ErrorKind::PermissionDenied
                        | std::io::ErrorKind::Other
                )
            })
    }

    /// 安全的目录遍历，自动跳过权限错误
    pub fn safe_walk_dir(
        path: &Path,
    ) -> impl Iterator<Item = Result<DirEntry, walkdir::Error>> + '_ {
        WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter(|e| {
                if let Err(ref err) = e {
                    !is_permission_error(err)
                } else {
                    true
                }
            })
    }
}
