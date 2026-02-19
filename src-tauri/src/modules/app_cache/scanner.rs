use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Instant, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use crate::utils::WeChatDatDecoder;
use crate::utils::{
    get_app_paths_config, get_cache_dir, should_skip_file, AppPathResolver, ResolvedAppPath,
    ScanCache,
};

pub const EVENT_APP_CACHE_PROGRESS: &str = "app_cache:progress";
pub const EVENT_APP_CACHE_COMPLETE: &str = "app_cache:complete";

pub const IMAGE_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "bmp", "webp"];
pub const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mov", "avi", "mkv", "wmv", "flv"];
pub const DOC_EXTENSIONS: &[&str] = &[
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "zip", "rar", "7z",
];
pub const INSTALL_EXTENSIONS: &[&str] = &["apk", "ipa", "exe", "dmg"];
pub const VOICE_EXTENSIONS: &[&str] = &["amr", "mp3", "wav"];
pub const EMOJI_EXTENSIONS: &[&str] = &["gif", "png", "jpg"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AppType {
    Wechat,
    Dingtalk,
    Qq,
    Wework,
}

impl AppType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "wechat" => Some(Self::Wechat),
            "dingtalk" => Some(Self::Dingtalk),
            "qq" => Some(Self::Qq),
            "wework" => Some(Self::Wework),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Wechat => "wechat",
            Self::Dingtalk => "dingtalk",
            Self::Qq => "qq",
            Self::Wework => "wework",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CleanCategory {
    ChatImages,
    VideoFiles,
    DocumentFiles,
    InstallPackages,
    CacheData,
    VoiceFiles,
    EmojiCache,
    TempFiles,
    ThumbCache,
}

impl CleanCategory {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "chat_images" => Some(Self::ChatImages),
            "video_files" => Some(Self::VideoFiles),
            "document_files" => Some(Self::DocumentFiles),
            "install_packages" => Some(Self::InstallPackages),
            "cache_data" => Some(Self::CacheData),
            "voice_files" => Some(Self::VoiceFiles),
            "emoji_cache" => Some(Self::EmojiCache),
            "temp_files" => Some(Self::TempFiles),
            "thumb_cache" => Some(Self::ThumbCache),
            _ => None,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            Self::ChatImages => "chat_images",
            Self::VideoFiles => "video_files",
            Self::DocumentFiles => "document_files",
            Self::InstallPackages => "install_packages",
            Self::CacheData => "cache_data",
            Self::VoiceFiles => "voice_files",
            Self::EmojiCache => "emoji_cache",
            Self::TempFiles => "temp_files",
            Self::ThumbCache => "thumb_cache",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Self::ChatImages => "聊天图片",
            Self::VideoFiles => "视频文件",
            Self::DocumentFiles => "文档文件",
            Self::InstallPackages => "安装包",
            Self::CacheData => "缓存数据",
            Self::VoiceFiles => "语音文件",
            Self::EmojiCache => "表情缓存",
            Self::TempFiles => "临时文件",
            Self::ThumbCache => "缩略图缓存",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::ChatImages => "聊天过程中发送和接收的图片，包括加密的.dat文件",
            Self::VideoFiles => "聊天过程中发送和接收的视频文件",
            Self::DocumentFiles => "聊天过程中接收的文档、压缩包等文件",
            Self::InstallPackages => "接收的安装包文件（apk、exe等）",
            Self::CacheData => "HTTP资源缓存、小程序图标缓存等",
            Self::VoiceFiles => "聊天语音消息文件",
            Self::EmojiCache => "表情包缓存文件",
            Self::TempFiles => "临时文件，可安全清理",
            Self::ThumbCache => "图片和视频的缩略图缓存",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AppCacheScanStatus {
    Idle,
    Scanning,
    Paused,
    Completed,
    Cancelled,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCacheFile {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub category: CleanCategory,
    pub app: AppType,
    pub chat_object: String,
    pub created_at: i64,
    pub modified_at: i64,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub is_encrypted: bool,
    #[serde(default)]
    pub original_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCacheScanOptions {
    pub apps: Vec<String>,
    pub categories: Vec<String>,
    #[serde(default)]
    pub incremental: bool,
    #[serde(default)]
    pub force_rescan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCacheScanProgress {
    pub scan_id: String,
    pub status: AppCacheScanStatus,
    pub current_path: String,
    pub scanned_files: u64,
    pub scanned_size: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub percent: f32,
    pub speed: f64,
    pub current_app: String,
    pub incremental: bool,
    pub skipped_files: u64,
}

impl AppCacheScanProgress {
    pub fn new(scan_id: &str) -> Self {
        Self {
            scan_id: scan_id.to_string(),
            status: AppCacheScanStatus::Idle,
            current_path: String::new(),
            scanned_files: 0,
            scanned_size: 0,
            total_files: 0,
            total_size: 0,
            percent: 0.0,
            speed: 0.0,
            current_app: String::new(),
            incremental: false,
            skipped_files: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppCacheScanResult {
    pub scan_id: String,
    pub files: Vec<AppCacheFile>,
    pub total_files: u64,
    pub total_size: u64,
    pub duration_ms: u64,
    pub status: AppCacheScanStatus,
    pub incremental: bool,
    pub skipped_files: u64,
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
    static ref SCAN_PROGRESS: Arc<RwLock<HashMap<String, AppCacheScanProgress>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_RESULTS: Arc<RwLock<HashMap<String, AppCacheScanResult>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_CONTROLLERS: Arc<RwLock<HashMap<String, ScanController>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref SCAN_CACHE: Arc<RwLock<Option<ScanCache>>> = Arc::new(RwLock::new(None));
}

const PROGRESS_UPDATE_INTERVAL: u64 = 20;

fn generate_scan_id() -> String {
    format!("app_cache_{}", Uuid::new_v4())
}

async fn get_or_load_scan_cache() -> ScanCache {
    let mut cache_guard = SCAN_CACHE.write().await;
    if cache_guard.is_none() {
        if let Some(cache_dir) = get_cache_dir() {
            *cache_guard = Some(ScanCache::load(&cache_dir));
        } else {
            *cache_guard = Some(ScanCache::new());
        }
    }
    cache_guard.clone().unwrap_or_default()
}

async fn save_scan_cache(cache: &ScanCache) {
    if let Some(cache_dir) = get_cache_dir() {
        let _ = cache.save(&cache_dir);
    }
}

fn should_scan_file_by_time(path: &Path, last_scan_timestamp: u64) -> bool {
    if last_scan_timestamp == 0 {
        return true;
    }

    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                return duration.as_secs() >= last_scan_timestamp;
            }
        }
    }

    true
}

pub async fn start_app_cache_scan(
    app: AppHandle,
    options: AppCacheScanOptions,
) -> Result<String, String> {
    let scan_id = generate_scan_id();

    let mut progress = AppCacheScanProgress::new(&scan_id);
    progress.incremental = options.incremental && !options.force_rescan;
    SCAN_PROGRESS
        .write()
        .await
        .insert(scan_id.clone(), progress);

    let (controller, mut pause_receiver, mut cancel_receiver) = ScanController::new();
    SCAN_CONTROLLERS
        .write()
        .await
        .insert(scan_id.clone(), controller);

    {
        let progress_map = SCAN_PROGRESS.read().await;
        if let Some(progress) = progress_map.get(&scan_id) {
            let _ = app.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
        }
    }

    let scan_id_clone = scan_id.clone();
    let app_clone = app.clone();
    tokio::spawn(async move {
        if let Err(e) = perform_app_cache_scan(
            &app_clone,
            &scan_id_clone,
            options,
            &mut pause_receiver,
            &mut cancel_receiver,
        )
        .await
        {
            let mut progress_map = SCAN_PROGRESS.write().await;
            if let Some(progress) = progress_map.get_mut(&scan_id_clone) {
                progress.status = AppCacheScanStatus::Error;
            }
            error!("App cache scan error: {}", e);
        }

        SCAN_CONTROLLERS.write().await.remove(&scan_id_clone);
    });

    Ok(scan_id)
}

async fn perform_app_cache_scan(
    app: &AppHandle,
    scan_id: &str,
    options: AppCacheScanOptions,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
) -> Result<(), String> {
    let start_instant = Instant::now();
    let mut files: Vec<AppCacheFile> = Vec::new();
    let mut last_update = 0u64;
    let mut skipped_files: u64 = 0;

    debug!(
        "[AppCacheScan] Starting scan with options: apps={:?}, categories={:?}, incremental={}",
        options.apps, options.categories, options.incremental
    );

    {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = AppCacheScanStatus::Scanning;
            let _ = app.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
        }
    }

    let apps: Vec<AppType> = options
        .apps
        .iter()
        .filter_map(|s| AppType::from_str(s))
        .collect();

    debug!(
        "[AppCacheScan] Parsed apps: {:?}",
        apps.iter().map(|a| a.to_str()).collect::<Vec<_>>()
    );

    let categories: Vec<CleanCategory> = if options.categories.is_empty() {
        vec![
            CleanCategory::ChatImages,
            CleanCategory::VideoFiles,
            CleanCategory::DocumentFiles,
            CleanCategory::InstallPackages,
            CleanCategory::CacheData,
            CleanCategory::VoiceFiles,
            CleanCategory::EmojiCache,
        ]
    } else {
        options
            .categories
            .iter()
            .filter_map(|s| CleanCategory::from_str(s))
            .collect()
    };

    let scan_cache = get_or_load_scan_cache().await;
    let last_scan_timestamp = if options.incremental && !options.force_rescan {
        scan_cache.get_app_last_scan("all").unwrap_or(0)
    } else {
        0
    };

    if last_scan_timestamp > 0 {
        debug!(
            "[AppCacheScan] Using incremental scan, last scan timestamp: {}",
            last_scan_timestamp
        );
    }

    let total_apps = apps.len() as u64;
    let mut current_app_index = 0u64;
    let is_incremental = last_scan_timestamp > 0;

    for app_type in &apps {
        debug!("[AppCacheScan] Processing app: {}", app_type.to_str());

        if *cancel_receiver.borrow() {
            let mut progress_map = SCAN_PROGRESS.write().await;
            if let Some(progress) = progress_map.get_mut(scan_id) {
                progress.status = AppCacheScanStatus::Cancelled;
            }
            return Ok(());
        }

        while *pause_receiver.borrow() {
            {
                let mut progress_map = SCAN_PROGRESS.write().await;
                if let Some(progress) = progress_map.get_mut(scan_id) {
                    progress.status = AppCacheScanStatus::Paused;
                    let _ = app.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            if *cancel_receiver.borrow() {
                let mut progress_map = SCAN_PROGRESS.write().await;
                if let Some(progress) = progress_map.get_mut(scan_id) {
                    progress.status = AppCacheScanStatus::Cancelled;
                }
                return Ok(());
            }
        }

        {
            let mut progress_map = SCAN_PROGRESS.write().await;
            if let Some(progress) = progress_map.get_mut(scan_id) {
                progress.current_app = app_type.to_str().to_string();
                progress.status = AppCacheScanStatus::Scanning;
            }
        }

        let resolved_paths = resolve_app_paths(app_type);
        debug!(
            "[AppCacheScan] Resolved {} paths for {}",
            resolved_paths.len(),
            app_type.to_str()
        );

        for resolved in resolved_paths {
            debug!(
                "[AppCacheScan] Scanning path: {} (source: {:?})",
                resolved.path.display(),
                resolved.source
            );
            if resolved.path.exists() {
                scan_app_path(
                    &resolved.path,
                    app_type,
                    &categories,
                    &mut files,
                    scan_id,
                    app,
                    &mut last_update,
                    &start_instant,
                    current_app_index,
                    total_apps,
                    pause_receiver,
                    cancel_receiver,
                    last_scan_timestamp,
                    &mut skipped_files,
                    resolved.account_id.as_deref(),
                )
                .await;
            }
        }

        current_app_index += 1;
    }

    let duration_ms = start_instant.elapsed().as_millis() as u64;
    let total_files = files.len() as u64;
    let total_size = files.iter().map(|f| f.size).sum();

    debug!(
        "[AppCacheScan] Scan completed: {} files, {} bytes, {} ms, {} skipped",
        total_files, total_size, duration_ms, skipped_files
    );

    let mut updated_cache = scan_cache.clone();
    updated_cache.update_app_info("all", vec![], total_files, total_size);
    save_scan_cache(&updated_cache).await;

    let result = AppCacheScanResult {
        scan_id: scan_id.to_string(),
        files,
        total_files,
        total_size,
        duration_ms,
        status: AppCacheScanStatus::Completed,
        incremental: is_incremental,
        skipped_files,
    };

    {
        let mut progress_map = SCAN_PROGRESS.write().await;
        if let Some(progress) = progress_map.get_mut(scan_id) {
            progress.status = AppCacheScanStatus::Completed;
            progress.percent = 100.0;
            progress.scanned_files = total_files;
            progress.scanned_size = total_size;
            progress.skipped_files = skipped_files;
            let _ = app.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
        }
    }

    SCAN_RESULTS
        .write()
        .await
        .insert(scan_id.to_string(), result.clone());
    debug!(
        "[AppCacheScan] Emitting complete event with {} files",
        result.files.len()
    );
    let _ = app.emit(EVENT_APP_CACHE_COMPLETE, &result);

    Ok(())
}

fn resolve_app_paths(app: &AppType) -> Vec<ResolvedAppPath> {
    let config = get_app_paths_config();
    match app {
        AppType::Wechat => {
            AppPathResolver::resolve_wechat_paths(config.get_path("wechat").as_ref())
        }
        AppType::Qq => AppPathResolver::resolve_qq_paths(config.get_path("qq").as_ref()),
        AppType::Dingtalk => {
            AppPathResolver::resolve_dingtalk_paths(config.get_path("dingtalk").as_ref())
        }
        AppType::Wework => {
            AppPathResolver::resolve_wework_paths(config.get_path("wework").as_ref())
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_app_path(
    base_path: &Path,
    app: &AppType,
    categories: &[CleanCategory],
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    account_id: Option<&str>,
) {
    match app {
        AppType::Wechat => {
            scan_wechat(
                base_path,
                categories,
                files,
                scan_id,
                app_handle,
                last_update,
                start_instant,
                current_app_index,
                total_apps,
                pause_receiver,
                cancel_receiver,
                last_scan_timestamp,
                skipped_files,
                account_id,
            )
            .await
        }
        AppType::Dingtalk => {
            scan_dingtalk(
                base_path,
                categories,
                files,
                scan_id,
                app_handle,
                last_update,
                start_instant,
                current_app_index,
                total_apps,
                pause_receiver,
                cancel_receiver,
                last_scan_timestamp,
                skipped_files,
                account_id,
            )
            .await
        }
        AppType::Qq => {
            scan_qq(
                base_path,
                categories,
                files,
                scan_id,
                app_handle,
                last_update,
                start_instant,
                current_app_index,
                total_apps,
                pause_receiver,
                cancel_receiver,
                last_scan_timestamp,
                skipped_files,
                account_id,
            )
            .await
        }
        AppType::Wework => {
            scan_wework(
                base_path,
                categories,
                files,
                scan_id,
                app_handle,
                last_update,
                start_instant,
                current_app_index,
                total_apps,
                pause_receiver,
                cancel_receiver,
                last_scan_timestamp,
                skipped_files,
                account_id,
            )
            .await
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat(
    base_path: &Path,
    categories: &[CleanCategory],
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    account_id: Option<&str>,
) {
    debug!(
        "[AppCacheScan] scan_wechat called with base_path: {}",
        base_path.display()
    );

    let chat_object = account_id
        .unwrap_or_else(|| {
            base_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知")
        })
        .to_string();

    let cache_dir = base_path.join("cache");
    let msg_dir = base_path.join("msg");
    let temp_dir = base_path.join("temp");

    if categories.contains(&CleanCategory::ChatImages) {
        scan_wechat_chat_images(
            &cache_dir,
            &msg_dir,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            &chat_object,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VideoFiles) {
        let video_dir = msg_dir.join("video");
        scan_wechat_video_files(
            &video_dir,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            &chat_object,
        )
        .await;
    }

    if categories.contains(&CleanCategory::DocumentFiles) {
        let file_dir = msg_dir.join("file");
        scan_wechat_document_files(
            &file_dir,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            &chat_object,
        )
        .await;
    }

    if categories.contains(&CleanCategory::CacheData) {
        scan_wechat_cache_data(
            &cache_dir,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            &chat_object,
        )
        .await;
    }

    if categories.contains(&CleanCategory::EmojiCache) {
        scan_wechat_emoji_cache(
            &cache_dir,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            &chat_object,
        )
        .await;
    }

    if categories.contains(&CleanCategory::TempFiles) {
        scan_directory_async(
            &temp_dir,
            CleanCategory::TempFiles,
            AppType::Wechat,
            &chat_object,
            files,
            true,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
        )
        .await;
    }

    if categories.contains(&CleanCategory::ThumbCache) {
        scan_wechat_thumb_cache(
            &cache_dir,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            &chat_object,
        )
        .await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_chat_images(
    cache_dir: &Path,
    msg_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    let attach_dir = msg_dir.join("attach");
    if attach_dir.exists() {
        if let Ok(entries) = fs::read_dir(&attach_dir) {
            for entry in entries.flatten() {
                let session_dir = entry.path();
                if session_dir.is_dir() {
                    scan_wechat_attach_session(
                        &session_dir,
                        files,
                        scan_id,
                        app_handle,
                        last_update,
                        start_instant,
                        current_app_index,
                        total_apps,
                        pause_receiver,
                        cancel_receiver,
                        last_scan_timestamp,
                        skipped_files,
                        chat_object,
                    )
                    .await;
                }
            }
        }
    }

    if cache_dir.exists() {
        if let Ok(month_entries) = fs::read_dir(cache_dir) {
            for month_entry in month_entries.flatten() {
                let month_dir = month_entry.path();
                if month_dir.is_dir() {
                    let msg_cache_dir = month_dir.join("Message");
                    if msg_cache_dir.exists() {
                        if let Ok(session_entries) = fs::read_dir(&msg_cache_dir) {
                            for session_entry in session_entries.flatten() {
                                let session_dir = session_entry.path();
                                if session_dir.is_dir() {
                                    let image_temp = session_dir.join("ImageTemp");
                                    if image_temp.exists() {
                                        scan_directory_async(
                                            &image_temp,
                                            CleanCategory::ChatImages,
                                            AppType::Wechat,
                                            chat_object,
                                            files,
                                            true,
                                            scan_id,
                                            app_handle,
                                            last_update,
                                            start_instant,
                                            current_app_index,
                                            total_apps,
                                            pause_receiver,
                                            cancel_receiver,
                                            last_scan_timestamp,
                                            skipped_files,
                                        )
                                        .await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_attach_session(
    session_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    _scan_id: &str,
    _app_handle: &AppHandle,
    _last_update: &mut u64,
    _start_instant: &Instant,
    _current_app_index: u64,
    _total_apps: u64,
    _pause_receiver: &mut watch::Receiver<bool>,
    _cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    let mut dirs_to_scan = vec![session_dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_scan.pop() {
        if let Ok(entries) = fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs_to_scan.push(path);
                } else if path.extension().map(|e| e == "dat").unwrap_or(false) {
                    if should_skip_file(&path) {
                        *skipped_files += 1;
                        continue;
                    }
                    if let Ok(metadata) = fs::metadata(&path) {
                        let modified = metadata.modified().ok();
                        let created = metadata.created().ok();
                        let modified_ts = modified
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64);
                        let created_ts = created
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64);

                        if let Some(modified_time) = modified_ts {
                            if modified_time < last_scan_timestamp as i64 {
                                *skipped_files += 1;
                                continue;
                            }
                        }

                        let file = AppCacheFile {
                            id: Uuid::new_v4().to_string(),
                            path: path.to_string_lossy().to_string(),
                            name: path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default(),
                            size: metadata.len(),
                            category: CleanCategory::ChatImages,
                            app: AppType::Wechat,
                            chat_object: chat_object.to_string(),
                            created_at: created_ts.unwrap_or(0),
                            modified_at: modified_ts.unwrap_or(0),
                            selected: false,
                            is_encrypted: true,
                            original_format: None,
                        };
                        files.push(file);
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_video_files(
    video_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    if !video_dir.exists() {
        return;
    }

    if let Ok(month_entries) = fs::read_dir(video_dir) {
        for month_entry in month_entries.flatten() {
            let month_dir = month_entry.path();
            if month_dir.is_dir() {
                scan_files_by_extension_async(
                    &month_dir,
                    VIDEO_EXTENSIONS,
                    CleanCategory::VideoFiles,
                    AppType::Wechat,
                    chat_object,
                    files,
                    scan_id,
                    app_handle,
                    last_update,
                    start_instant,
                    current_app_index,
                    total_apps,
                    pause_receiver,
                    cancel_receiver,
                    last_scan_timestamp,
                    skipped_files,
                    false,
                )
                .await;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_document_files(
    file_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    if !file_dir.exists() {
        return;
    }

    if let Ok(month_entries) = fs::read_dir(file_dir) {
        for month_entry in month_entries.flatten() {
            let month_dir = month_entry.path();
            if month_dir.is_dir() {
                scan_files_by_extension_async(
                    &month_dir,
                    DOC_EXTENSIONS,
                    CleanCategory::DocumentFiles,
                    AppType::Wechat,
                    chat_object,
                    files,
                    scan_id,
                    app_handle,
                    last_update,
                    start_instant,
                    current_app_index,
                    total_apps,
                    pause_receiver,
                    cancel_receiver,
                    last_scan_timestamp,
                    skipped_files,
                    false,
                )
                .await;
                scan_files_by_extension_async(
                    &month_dir,
                    INSTALL_EXTENSIONS,
                    CleanCategory::InstallPackages,
                    AppType::Wechat,
                    chat_object,
                    files,
                    scan_id,
                    app_handle,
                    last_update,
                    start_instant,
                    current_app_index,
                    total_apps,
                    pause_receiver,
                    cancel_receiver,
                    last_scan_timestamp,
                    skipped_files,
                    false,
                )
                .await;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_cache_data(
    cache_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    if !cache_dir.exists() {
        return;
    }

    if let Ok(month_entries) = fs::read_dir(cache_dir) {
        for month_entry in month_entries.flatten() {
            let month_dir = month_entry.path();
            if month_dir.is_dir() {
                let http_resource = month_dir.join("HttpResource");
                if http_resource.exists() {
                    scan_directory_async(
                        &http_resource,
                        CleanCategory::CacheData,
                        AppType::Wechat,
                        chat_object,
                        files,
                        true,
                        scan_id,
                        app_handle,
                        last_update,
                        start_instant,
                        current_app_index,
                        total_apps,
                        pause_receiver,
                        cancel_receiver,
                        last_scan_timestamp,
                        skipped_files,
                    )
                    .await;
                }

                let weapp_icon = month_dir.join("WeAppIcon");
                if weapp_icon.exists() {
                    scan_directory_async(
                        &weapp_icon,
                        CleanCategory::CacheData,
                        AppType::Wechat,
                        chat_object,
                        files,
                        true,
                        scan_id,
                        app_handle,
                        last_update,
                        start_instant,
                        current_app_index,
                        total_apps,
                        pause_receiver,
                        cancel_receiver,
                        last_scan_timestamp,
                        skipped_files,
                    )
                    .await;
                }

                let msg_cache_dir = month_dir.join("Message");
                if msg_cache_dir.exists() {
                    if let Ok(session_entries) = fs::read_dir(&msg_cache_dir) {
                        for session_entry in session_entries.flatten() {
                            let session_dir = session_entry.path();
                            if session_dir.is_dir() {
                                let file_temp = session_dir.join("FileTemp");
                                if file_temp.exists() {
                                    scan_directory_async(
                                        &file_temp,
                                        CleanCategory::CacheData,
                                        AppType::Wechat,
                                        chat_object,
                                        files,
                                        true,
                                        scan_id,
                                        app_handle,
                                        last_update,
                                        start_instant,
                                        current_app_index,
                                        total_apps,
                                        pause_receiver,
                                        cancel_receiver,
                                        last_scan_timestamp,
                                        skipped_files,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_emoji_cache(
    cache_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    if !cache_dir.exists() {
        return;
    }

    if let Ok(month_entries) = fs::read_dir(cache_dir) {
        for month_entry in month_entries.flatten() {
            let month_dir = month_entry.path();
            if month_dir.is_dir() {
                let emoticon_dir = month_dir.join("Emoticon");
                if emoticon_dir.exists() {
                    scan_directory_async(
                        &emoticon_dir,
                        CleanCategory::EmojiCache,
                        AppType::Wechat,
                        chat_object,
                        files,
                        true,
                        scan_id,
                        app_handle,
                        last_update,
                        start_instant,
                        current_app_index,
                        total_apps,
                        pause_receiver,
                        cancel_receiver,
                        last_scan_timestamp,
                        skipped_files,
                    )
                    .await;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wechat_thumb_cache(
    cache_dir: &Path,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    chat_object: &str,
) {
    if !cache_dir.exists() {
        return;
    }

    if let Ok(month_entries) = fs::read_dir(cache_dir) {
        for month_entry in month_entries.flatten() {
            let month_dir = month_entry.path();
            if month_dir.is_dir() {
                let msg_cache_dir = month_dir.join("Message");
                if msg_cache_dir.exists() {
                    if let Ok(session_entries) = fs::read_dir(&msg_cache_dir) {
                        for session_entry in session_entries.flatten() {
                            let session_dir = session_entry.path();
                            if session_dir.is_dir() {
                                let thumb_dir = session_dir.join("Thumb");
                                if thumb_dir.exists() {
                                    scan_directory_async(
                                        &thumb_dir,
                                        CleanCategory::ThumbCache,
                                        AppType::Wechat,
                                        chat_object,
                                        files,
                                        true,
                                        scan_id,
                                        app_handle,
                                        last_update,
                                        start_instant,
                                        current_app_index,
                                        total_apps,
                                        pause_receiver,
                                        cancel_receiver,
                                        last_scan_timestamp,
                                        skipped_files,
                                    )
                                    .await;
                                }
                            }
                        }
                    }
                }

                let sns_dir = month_dir.join("Sns");
                if sns_dir.exists() {
                    let sns_img = sns_dir.join("Img");
                    if sns_img.exists() {
                        scan_directory_async(
                            &sns_img,
                            CleanCategory::ThumbCache,
                            AppType::Wechat,
                            chat_object,
                            files,
                            true,
                            scan_id,
                            app_handle,
                            last_update,
                            start_instant,
                            current_app_index,
                            total_apps,
                            pause_receiver,
                            cancel_receiver,
                            last_scan_timestamp,
                            skipped_files,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_dingtalk(
    base_path: &Path,
    categories: &[CleanCategory],
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    account_id: Option<&str>,
) {
    let chat_object = account_id
        .unwrap_or_else(|| {
            base_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知")
        })
        .to_string();

    if categories.contains(&CleanCategory::ChatImages) {
        let image_path = base_path.join("Image");
        scan_files_by_extension_async(
            &image_path,
            IMAGE_EXTENSIONS,
            CleanCategory::ChatImages,
            AppType::Dingtalk,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
        let image_path2 = base_path.join("images");
        scan_files_by_extension_async(
            &image_path2,
            IMAGE_EXTENSIONS,
            CleanCategory::ChatImages,
            AppType::Dingtalk,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VideoFiles) {
        let video_path = base_path.join("Video");
        scan_files_by_extension_async(
            &video_path,
            VIDEO_EXTENSIONS,
            CleanCategory::VideoFiles,
            AppType::Dingtalk,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::DocumentFiles) {
        let file_path = base_path.join("File");
        scan_files_by_extension_async(
            &file_path,
            DOC_EXTENSIONS,
            CleanCategory::DocumentFiles,
            AppType::Dingtalk,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::CacheData) {
        let cache_path = base_path.join("Cache");
        scan_directory_async(
            &cache_path,
            CleanCategory::CacheData,
            AppType::Dingtalk,
            &chat_object,
            files,
            true,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
        )
        .await;
        let temp_path = base_path.join("Temp");
        scan_directory_async(
            &temp_path,
            CleanCategory::CacheData,
            AppType::Dingtalk,
            &chat_object,
            files,
            true,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VoiceFiles) {
        let voice_path = base_path.join("Voice");
        scan_files_by_extension_async(
            &voice_path,
            VOICE_EXTENSIONS,
            CleanCategory::VoiceFiles,
            AppType::Dingtalk,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_qq(
    base_path: &Path,
    categories: &[CleanCategory],
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    account_id: Option<&str>,
) {
    let chat_object = account_id
        .unwrap_or_else(|| {
            base_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知")
        })
        .to_string();

    if categories.contains(&CleanCategory::ChatImages) {
        let image_path = base_path.join("Image");
        scan_files_by_extension_async(
            &image_path,
            IMAGE_EXTENSIONS,
            CleanCategory::ChatImages,
            AppType::Qq,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
        let image_path2 = base_path.join("FileRecv/Image");
        scan_files_by_extension_async(
            &image_path2,
            IMAGE_EXTENSIONS,
            CleanCategory::ChatImages,
            AppType::Qq,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VideoFiles) {
        let video_path = base_path.join("Video");
        scan_files_by_extension_async(
            &video_path,
            VIDEO_EXTENSIONS,
            CleanCategory::VideoFiles,
            AppType::Qq,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::DocumentFiles) {
        let file_path = base_path.join("FileRecv");
        scan_files_by_extension_async(
            &file_path,
            DOC_EXTENSIONS,
            CleanCategory::DocumentFiles,
            AppType::Qq,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::CacheData) {
        let cache_path = base_path.join("Cache");
        scan_directory_async(
            &cache_path,
            CleanCategory::CacheData,
            AppType::Qq,
            &chat_object,
            files,
            true,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VoiceFiles) {
        let voice_path = base_path.join("Audio");
        scan_files_by_extension_async(
            &voice_path,
            VOICE_EXTENSIONS,
            CleanCategory::VoiceFiles,
            AppType::Qq,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::EmojiCache) {
        let emoji_path = base_path.join("Emoji");
        scan_files_by_extension_async(
            &emoji_path,
            EMOJI_EXTENSIONS,
            CleanCategory::EmojiCache,
            AppType::Qq,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_wework(
    base_path: &Path,
    categories: &[CleanCategory],
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    account_id: Option<&str>,
) {
    let chat_object = account_id
        .unwrap_or_else(|| {
            base_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知")
        })
        .to_string();

    if categories.contains(&CleanCategory::ChatImages) {
        let image_path = base_path.join("Image");
        scan_files_by_extension_async(
            &image_path,
            IMAGE_EXTENSIONS,
            CleanCategory::ChatImages,
            AppType::Wework,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VideoFiles) {
        let video_path = base_path.join("Video");
        scan_files_by_extension_async(
            &video_path,
            VIDEO_EXTENSIONS,
            CleanCategory::VideoFiles,
            AppType::Wework,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::DocumentFiles) {
        let file_path = base_path.join("File");
        scan_files_by_extension_async(
            &file_path,
            DOC_EXTENSIONS,
            CleanCategory::DocumentFiles,
            AppType::Wework,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }

    if categories.contains(&CleanCategory::CacheData) {
        let cache_path = base_path.join("Cache");
        scan_directory_async(
            &cache_path,
            CleanCategory::CacheData,
            AppType::Wework,
            &chat_object,
            files,
            true,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
        )
        .await;
    }

    if categories.contains(&CleanCategory::VoiceFiles) {
        let voice_path = base_path.join("Voice");
        scan_files_by_extension_async(
            &voice_path,
            VOICE_EXTENSIONS,
            CleanCategory::VoiceFiles,
            AppType::Wework,
            &chat_object,
            files,
            scan_id,
            app_handle,
            last_update,
            start_instant,
            current_app_index,
            total_apps,
            pause_receiver,
            cancel_receiver,
            last_scan_timestamp,
            skipped_files,
            false,
        )
        .await;
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_files_by_extension_async(
    dir: &Path,
    extensions: &[&str],
    category: CleanCategory,
    app: AppType,
    chat_object: &str,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    check_encrypted: bool,
) {
    if !dir.exists() {
        return;
    }

    scan_files_by_extension_recursive(
        dir,
        extensions,
        category,
        app,
        chat_object,
        files,
        scan_id,
        app_handle,
        last_update,
        start_instant,
        current_app_index,
        total_apps,
        pause_receiver,
        cancel_receiver,
        last_scan_timestamp,
        skipped_files,
        check_encrypted,
    )
    .await;
}

#[allow(clippy::too_many_arguments)]
async fn scan_files_by_extension_recursive(
    dir: &Path,
    extensions: &[&str],
    category: CleanCategory,
    app: AppType,
    chat_object: &str,
    files: &mut Vec<AppCacheFile>,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
    check_encrypted: bool,
) {
    if !dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if should_skip_file(&path) {
                continue;
            }

            if *cancel_receiver.borrow() {
                return;
            }

            while *pause_receiver.borrow() {
                {
                    let mut progress_map = SCAN_PROGRESS.write().await;
                    if let Some(progress) = progress_map.get_mut(scan_id) {
                        progress.status = AppCacheScanStatus::Paused;
                        let _ = app_handle.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                if *cancel_receiver.borrow() {
                    return;
                }
            }

            if path.is_dir() {
                Box::pin(scan_files_by_extension_recursive(
                    &path,
                    extensions,
                    category.clone(),
                    app.clone(),
                    chat_object,
                    files,
                    scan_id,
                    app_handle,
                    last_update,
                    start_instant,
                    current_app_index,
                    total_apps,
                    pause_receiver,
                    cancel_receiver,
                    last_scan_timestamp,
                    skipped_files,
                    check_encrypted,
                ))
                .await;
            } else {
                if !should_scan_file_by_time(&path, last_scan_timestamp) {
                    *skipped_files += 1;
                    continue;
                }

                let ext = path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                if extensions.iter().any(|&e| e == ext) {
                    if let Some(mut file) =
                        create_app_cache_file(&path, category.clone(), app.clone(), chat_object)
                    {
                        if check_encrypted && ext == "dat" {
                            if let Some(info) = WeChatDatDecoder::analyze_dat_file(&path) {
                                file.is_encrypted = true;
                                file.original_format =
                                    Some(info.original_format.extension().to_string());
                            }
                        }

                        files.push(file);

                        let current_count = files.len() as u64;
                        if current_count - *last_update >= PROGRESS_UPDATE_INTERVAL {
                            *last_update = current_count;
                            update_progress(
                                scan_id,
                                app_handle,
                                &path.display().to_string(),
                                current_count,
                                files.iter().map(|f| f.size).sum(),
                                start_instant,
                                current_app_index,
                                total_apps,
                            )
                            .await;
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn scan_directory_async(
    dir: &Path,
    category: CleanCategory,
    app: AppType,
    chat_object: &str,
    files: &mut Vec<AppCacheFile>,
    recursive: bool,
    scan_id: &str,
    app_handle: &AppHandle,
    last_update: &mut u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
    pause_receiver: &mut watch::Receiver<bool>,
    cancel_receiver: &mut watch::Receiver<bool>,
    last_scan_timestamp: u64,
    skipped_files: &mut u64,
) {
    if !dir.exists() {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if should_skip_file(&path) {
                continue;
            }

            if *cancel_receiver.borrow() {
                return;
            }

            while *pause_receiver.borrow() {
                {
                    let mut progress_map = SCAN_PROGRESS.write().await;
                    if let Some(progress) = progress_map.get_mut(scan_id) {
                        progress.status = AppCacheScanStatus::Paused;
                        let _ = app_handle.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                if *cancel_receiver.borrow() {
                    return;
                }
            }

            if path.is_dir() && recursive {
                Box::pin(scan_directory_async(
                    &path,
                    category.clone(),
                    app.clone(),
                    chat_object,
                    files,
                    true,
                    scan_id,
                    app_handle,
                    last_update,
                    start_instant,
                    current_app_index,
                    total_apps,
                    pause_receiver,
                    cancel_receiver,
                    last_scan_timestamp,
                    skipped_files,
                ))
                .await;
            } else if path.is_file() {
                if !should_scan_file_by_time(&path, last_scan_timestamp) {
                    *skipped_files += 1;
                    continue;
                }

                if let Some(file) =
                    create_app_cache_file(&path, category.clone(), app.clone(), chat_object)
                {
                    files.push(file);

                    let current_count = files.len() as u64;
                    if current_count - *last_update >= PROGRESS_UPDATE_INTERVAL {
                        *last_update = current_count;
                        update_progress(
                            scan_id,
                            app_handle,
                            &path.display().to_string(),
                            current_count,
                            files.iter().map(|f| f.size).sum(),
                            start_instant,
                            current_app_index,
                            total_apps,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

async fn update_progress(
    scan_id: &str,
    app_handle: &AppHandle,
    current_path: &str,
    scanned_files: u64,
    scanned_size: u64,
    start_instant: &Instant,
    current_app_index: u64,
    total_apps: u64,
) {
    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.current_path = current_path.to_string();
        progress.scanned_files = scanned_files;
        progress.scanned_size = scanned_size;

        let elapsed = start_instant.elapsed().as_secs_f64();
        progress.speed = if elapsed > 0.0 {
            scanned_size as f64 / elapsed
        } else {
            0.0
        };
        progress.status = AppCacheScanStatus::Scanning;

        let app_progress = if total_apps > 0 {
            (current_app_index as f32 / total_apps as f32) * 100.0
        } else {
            0.0
        };
        progress.percent = app_progress.min(99.0);

        let _ = app_handle.emit(EVENT_APP_CACHE_PROGRESS, progress.clone());
    }
}

fn create_app_cache_file(
    path: &Path,
    category: CleanCategory,
    app: AppType,
    chat_object: &str,
) -> Option<AppCacheFile> {
    let metadata = fs::metadata(path).ok()?;

    let size = metadata.len();
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let created_at = metadata
        .created()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(modified_at);

    Some(AppCacheFile {
        id: Uuid::new_v4().to_string(),
        path: path.to_string_lossy().to_string(),
        name,
        size,
        category,
        app,
        chat_object: chat_object.to_string(),
        created_at,
        modified_at,
        selected: false,
        is_encrypted: false,
        original_format: None,
    })
}

pub async fn get_app_cache_progress(scan_id: &str) -> Option<AppCacheScanProgress> {
    let progress_map = SCAN_PROGRESS.read().await;
    progress_map.get(scan_id).cloned()
}

pub async fn get_app_cache_result(scan_id: &str) -> Option<AppCacheScanResult> {
    let results_map = SCAN_RESULTS.read().await;
    results_map.get(scan_id).cloned()
}

pub async fn pause_app_cache_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.pause();
    }

    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.status = AppCacheScanStatus::Paused;
    }
    Ok(())
}

pub async fn resume_app_cache_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.resume();
    }

    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.status = AppCacheScanStatus::Scanning;
    }
    Ok(())
}

pub async fn cancel_app_cache_scan(scan_id: &str) -> Result<(), String> {
    let controllers = SCAN_CONTROLLERS.read().await;
    if let Some(controller) = controllers.get(scan_id) {
        controller.cancel();
    }

    let mut progress_map = SCAN_PROGRESS.write().await;
    if let Some(progress) = progress_map.get_mut(scan_id) {
        progress.status = AppCacheScanStatus::Cancelled;
    }
    Ok(())
}

pub async fn clear_app_cache_result(scan_id: &str) -> Result<(), String> {
    SCAN_RESULTS.write().await.remove(scan_id);
    SCAN_PROGRESS.write().await.remove(scan_id);
    Ok(())
}

pub fn decrypt_wechat_image(source: &Path, target: &Path) -> Option<String> {
    WeChatDatDecoder::decrypt_to_file(source, target).map(|f| f.extension().to_string())
}
