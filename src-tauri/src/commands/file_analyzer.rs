use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;

use crate::models::{
    DuplicateAnalysisResult, DuplicateDetectorOptions, DuplicateGroup, GarbageAnalysisResult,
    GarbageCategory, LargeFile, LargeFileAnalysisResult, LargeFileAnalyzerOptions,
    LargeFileDetails,
};
use crate::modules::file_analyzer::{
    cancel_junk_file_scan, clear_junk_file_scan_result, get_junk_category_files,
    get_junk_file_scan_progress, get_junk_file_scan_result, pause_junk_file_scan,
    resume_junk_file_scan, start_junk_file_scan, DuplicateDetector, GarbageDetector,
    GarbageDetectorOptions, JunkCategoryFilesResponse, JunkFileDetector, JunkFileScanProgress,
    JunkFileType, JunkScanOptions, JunkScanResult, LargeFileAnalyzer, LargeFileStats,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageCategoryInfo {
    pub category: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
}

#[tauri::command]
pub async fn analyze_garbage_files(
    options: Option<GarbageDetectorOptionsJson>,
) -> GarbageAnalysisResult {
    let opts = options
        .map(|o| GarbageDetectorOptions {
            include_system_temp: o.include_system_temp,
            include_browser_cache: o.include_browser_cache,
            include_app_cache: o.include_app_cache,
            include_recycle_bin: o.include_recycle_bin,
            include_log_files: o.include_log_files,
            min_file_age_days: o.min_file_age_days,
            max_files_per_category: o.max_files_per_category,
        })
        .unwrap_or_default();

    let detector = GarbageDetector::with_options(opts);
    detector.detect_all()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageDetectorOptionsJson {
    pub include_system_temp: bool,
    pub include_browser_cache: bool,
    pub include_app_cache: bool,
    pub include_recycle_bin: bool,
    pub include_log_files: bool,
    pub min_file_age_days: u32,
    pub max_files_per_category: Option<usize>,
}

impl Default for GarbageDetectorOptionsJson {
    fn default() -> Self {
        Self {
            include_system_temp: true,
            include_browser_cache: true,
            include_app_cache: true,
            include_recycle_bin: true,
            include_log_files: true,
            min_file_age_days: 0,
            max_files_per_category: None,
        }
    }
}

#[tauri::command]
pub async fn analyze_garbage_by_category(
    category: String,
) -> Result<GarbageAnalysisResult, String> {
    let parsed_category =
        parse_garbage_category(&category).map_err(|_| format!("Invalid category: {}", category))?;

    let detector = GarbageDetector::new();
    let files = match parsed_category {
        GarbageCategory::SystemTemp => detector.detect_system_temp(),
        GarbageCategory::BrowserCache => detector.detect_browser_cache(),
        GarbageCategory::AppCache => detector.detect_app_cache(),
        GarbageCategory::RecycleBin => detector.detect_recycle_bin(),
        GarbageCategory::LogFile => detector.detect_log_files(),
        GarbageCategory::Other => vec![],
    };

    let total_files = files.len() as u64;
    let total_size = files.iter().map(|f| f.size).sum();
    let high_risk_count = files
        .iter()
        .filter(|f| f.risk_level >= crate::models::RiskLevel::High)
        .count() as u64;

    let mut categories = std::collections::HashMap::new();
    categories.insert(
        category.clone(),
        crate::models::CategoryStats {
            category: parsed_category.display_name().to_string(),
            file_count: total_files,
            total_size,
            files,
        },
    );

    Ok(GarbageAnalysisResult {
        scan_id: uuid::Uuid::new_v4().to_string(),
        total_files,
        total_size,
        categories,
        high_risk_count,
        duration_ms: 0,
    })
}

fn parse_garbage_category(s: &str) -> Result<GarbageCategory, ()> {
    match s.to_lowercase().as_str() {
        "systemtemp" | "system_temp" => Ok(GarbageCategory::SystemTemp),
        "browsercache" | "browser_cache" => Ok(GarbageCategory::BrowserCache),
        "appcache" | "app_cache" => Ok(GarbageCategory::AppCache),
        "recyclebin" | "recycle_bin" => Ok(GarbageCategory::RecycleBin),
        "logfile" | "log_file" => Ok(GarbageCategory::LogFile),
        "other" => Ok(GarbageCategory::Other),
        _ => Err(()),
    }
}

#[tauri::command]
pub fn get_garbage_categories() -> Vec<GarbageCategoryInfo> {
    vec![
        GarbageCategoryInfo {
            category: "SystemTemp".to_string(),
            display_name: "系统临时文件".to_string(),
            description: "系统临时文件夹中的文件".to_string(),
            icon: "folder-temp".to_string(),
        },
        GarbageCategoryInfo {
            category: "BrowserCache".to_string(),
            display_name: "浏览器缓存".to_string(),
            description: "浏览器产生的缓存文件".to_string(),
            icon: "browser".to_string(),
        },
        GarbageCategoryInfo {
            category: "AppCache".to_string(),
            display_name: "应用程序缓存".to_string(),
            description: "应用程序产生的缓存文件".to_string(),
            icon: "app".to_string(),
        },
        GarbageCategoryInfo {
            category: "RecycleBin".to_string(),
            display_name: "回收站".to_string(),
            description: "已删除的文件".to_string(),
            icon: "trash".to_string(),
        },
        GarbageCategoryInfo {
            category: "LogFile".to_string(),
            display_name: "日志文件".to_string(),
            description: "系统和应用程序日志".to_string(),
            icon: "file-text".to_string(),
        },
    ]
}

#[tauri::command]
pub async fn analyze_large_files(
    path: String,
    threshold: Option<u64>,
    options: Option<LargeFileAnalyzerOptionsJson>,
) -> LargeFileAnalysisResult {
    let mut opts = options
        .map(|o| LargeFileAnalyzerOptions {
            threshold: o.threshold,
            exclude_paths: o.exclude_paths,
            include_hidden: o.include_hidden,
            include_system: o.include_system,
        })
        .unwrap_or_default();

    if let Some(t) = threshold {
        opts.threshold = t;
    }

    let analyzer = LargeFileAnalyzer::with_options(opts);
    let paths = vec![PathBuf::from(&path)];

    analyzer.analyze(&paths)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileAnalyzerOptionsJson {
    pub threshold: u64,
    pub exclude_paths: Vec<String>,
    pub include_hidden: bool,
    pub include_system: bool,
}

impl Default for LargeFileAnalyzerOptionsJson {
    fn default() -> Self {
        Self {
            threshold: 100 * 1024 * 1024,
            exclude_paths: vec![],
            include_hidden: false,
            include_system: false,
        }
    }
}

#[tauri::command]
pub async fn analyze_large_files_with_stats(
    path: String,
    threshold: Option<u64>,
) -> LargeFileAnalysisWithStats {
    let options = LargeFileAnalyzerOptions {
        threshold: threshold.unwrap_or(100 * 1024 * 1024),
        ..Default::default()
    };

    let analyzer = LargeFileAnalyzer::with_options(options);
    let paths = vec![PathBuf::from(&path)];

    let result = analyzer.analyze(&paths);
    let stats = analyzer.calculate_stats(&result.files);

    LargeFileAnalysisWithStats {
        result,
        stats: LargeFileStatsJson::from(stats),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileAnalysisWithStats {
    pub result: LargeFileAnalysisResult,
    pub stats: LargeFileStatsJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LargeFileStatsJson {
    pub total_count: u64,
    pub total_size: u64,
    pub average_size: u64,
    pub largest_file: Option<LargeFile>,
    pub type_distribution: Vec<TypeStatsJson>,
}

impl From<LargeFileStats> for LargeFileStatsJson {
    fn from(stats: LargeFileStats) -> Self {
        Self {
            total_count: stats.total_count,
            total_size: stats.total_size,
            average_size: stats.average_size,
            largest_file: stats.largest_file,
            type_distribution: stats
                .type_distribution
                .into_values()
                .map(|t| TypeStatsJson {
                    file_type: t.file_type,
                    count: t.count,
                    total_size: t.total_size,
                    percentage: t.percentage,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeStatsJson {
    pub file_type: String,
    pub count: u64,
    pub total_size: u64,
    pub percentage: f32,
}

#[tauri::command]
pub async fn get_large_file_details(path: String) -> Option<LargeFileDetails> {
    let analyzer = LargeFileAnalyzer::new();
    analyzer.get_file_details(PathBuf::from(&path).as_path())
}

#[tauri::command]
pub async fn find_duplicate_files(
    paths: Vec<String>,
    min_size: Option<u64>,
    options: Option<DuplicateDetectorOptionsJson>,
) -> DuplicateAnalysisResult {
    let mut opts = options
        .map(|o| DuplicateDetectorOptions {
            min_size: o.min_size,
            max_size: o.max_size,
            include_hidden: o.include_hidden,
            use_cache: o.use_cache,
        })
        .unwrap_or_default();

    if let Some(size) = min_size {
        opts.min_size = size;
    }

    let detector = DuplicateDetector::with_options(opts);
    let path_bufs: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();

    detector.find_duplicates(&path_bufs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectorOptionsJson {
    pub min_size: u64,
    pub max_size: Option<u64>,
    pub include_hidden: bool,
    pub use_cache: bool,
}

impl Default for DuplicateDetectorOptionsJson {
    fn default() -> Self {
        Self {
            min_size: 1024 * 1024,
            max_size: None,
            include_hidden: false,
            use_cache: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSuggestions {
    pub files_to_delete: Vec<String>,
    pub original_file: Option<String>,
}

#[tauri::command]
pub async fn get_duplicate_suggestions(group: DuplicateGroup) -> DuplicateSuggestions {
    let detector = DuplicateDetector::new();

    let files_to_delete = detector.suggest_files_to_delete(&group);
    let original = detector.suggest_original(&group);

    DuplicateSuggestions {
        files_to_delete,
        original_file: original.map(|f| f.path.clone()),
    }
}

#[tauri::command]
pub async fn scan_junk_files(options: Option<JunkScanOptionsJson>) -> Vec<JunkScanResult> {
    let opts = options
        .map(|o| JunkScanOptions {
            scan_paths: o.scan_paths,
            include_empty_folders: o.include_empty_folders,
            include_invalid_shortcuts: o.include_invalid_shortcuts,
            include_old_logs: o.include_old_logs,
            include_old_installers: o.include_old_installers,
            include_invalid_downloads: o.include_invalid_downloads,
            include_small_files: o.include_small_files,
            small_file_max_size: o.small_file_max_size,
            log_max_age_days: o.log_max_age_days,
            installer_max_age_days: o.installer_max_age_days,
            exclude_paths: o.exclude_paths,
            include_hidden: o.include_hidden,
            include_system: o.include_system,
        })
        .unwrap_or_default();

    let detector = JunkFileDetector::with_options(opts);
    detector.detect_all()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JunkScanOptionsJson {
    pub scan_paths: Vec<String>,
    pub include_empty_folders: bool,
    pub include_invalid_shortcuts: bool,
    pub include_old_logs: bool,
    pub include_old_installers: bool,
    pub include_invalid_downloads: bool,
    pub include_small_files: bool,
    pub small_file_max_size: u64,
    pub log_max_age_days: u32,
    pub installer_max_age_days: u32,
    pub exclude_paths: Vec<String>,
    pub include_hidden: bool,
    pub include_system: bool,
}

impl Default for JunkScanOptionsJson {
    fn default() -> Self {
        Self {
            scan_paths: vec![],
            include_empty_folders: true,
            include_invalid_shortcuts: true,
            include_old_logs: true,
            include_old_installers: true,
            include_invalid_downloads: true,
            include_small_files: false,
            small_file_max_size: 1024 * 100,
            log_max_age_days: 30,
            installer_max_age_days: 90,
            exclude_paths: vec![],
            include_hidden: false,
            include_system: false,
        }
    }
}

#[tauri::command]
pub async fn scan_junk_by_type(
    file_type: String,
    options: Option<JunkScanOptionsJson>,
) -> Option<JunkScanResult> {
    let parsed_type = parse_junk_file_type(&file_type)?;
    let opts = options.unwrap_or_default();

    let detector = JunkFileDetector::with_options(JunkScanOptions {
        scan_paths: opts.scan_paths,
        include_empty_folders: true,
        include_invalid_shortcuts: true,
        include_old_logs: true,
        include_old_installers: true,
        include_invalid_downloads: true,
        include_small_files: true,
        small_file_max_size: opts.small_file_max_size,
        log_max_age_days: opts.log_max_age_days,
        installer_max_age_days: opts.installer_max_age_days,
        exclude_paths: opts.exclude_paths,
        include_hidden: opts.include_hidden,
        include_system: opts.include_system,
    });

    detector.detect_by_type(parsed_type)
}

fn parse_junk_file_type(s: &str) -> Option<JunkFileType> {
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

#[tauri::command]
pub fn get_junk_file_types() -> Vec<JunkTypeInfo> {
    vec![
        JunkTypeInfo {
            type_name: "empty_folders".to_string(),
            display_name: "空文件夹".to_string(),
            description: "不包含任何文件的空目录".to_string(),
            icon: "folder-x".to_string(),
        },
        JunkTypeInfo {
            type_name: "invalid_shortcuts".to_string(),
            display_name: "无效快捷方式".to_string(),
            description: "目标路径不存在的快捷方式".to_string(),
            icon: "link-broken".to_string(),
        },
        JunkTypeInfo {
            type_name: "old_logs".to_string(),
            display_name: "过期日志文件".to_string(),
            description: "超过指定天数未访问的日志文件".to_string(),
            icon: "file-text".to_string(),
        },
        JunkTypeInfo {
            type_name: "old_installers".to_string(),
            display_name: "旧版本安装包".to_string(),
            description: "已安装程序的旧版本安装包".to_string(),
            icon: "package".to_string(),
        },
        JunkTypeInfo {
            type_name: "invalid_downloads".to_string(),
            display_name: "无效下载文件".to_string(),
            description: "下载目录中未完成或损坏的文件".to_string(),
            icon: "download".to_string(),
        },
        JunkTypeInfo {
            type_name: "small_files".to_string(),
            display_name: "零散小文件".to_string(),
            description: "小于指定大小的零散文件".to_string(),
            icon: "file".to_string(),
        },
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkTypeInfo {
    pub type_name: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
}

// Async junk file scan commands with progress support
#[tauri::command]
pub async fn junk_file_scan_start(
    app: AppHandle,
    options: Option<JunkScanOptionsJson>,
) -> Result<String, String> {
    let opts = options.map(|o| JunkScanOptions {
        scan_paths: o.scan_paths,
        include_empty_folders: o.include_empty_folders,
        include_invalid_shortcuts: o.include_invalid_shortcuts,
        include_old_logs: o.include_old_logs,
        include_old_installers: o.include_old_installers,
        include_invalid_downloads: o.include_invalid_downloads,
        include_small_files: o.include_small_files,
        small_file_max_size: o.small_file_max_size,
        log_max_age_days: o.log_max_age_days,
        installer_max_age_days: o.installer_max_age_days,
        exclude_paths: o.exclude_paths,
        include_hidden: o.include_hidden,
        include_system: o.include_system,
    });

    start_junk_file_scan(app, opts).await
}

#[tauri::command]
pub async fn junk_file_scan_pause(scan_id: String) -> Result<(), String> {
    pause_junk_file_scan(&scan_id).await
}

#[tauri::command]
pub async fn junk_file_scan_resume(scan_id: String) -> Result<(), String> {
    resume_junk_file_scan(&scan_id).await
}

#[tauri::command]
pub async fn junk_file_scan_cancel(scan_id: String) -> Result<(), String> {
    cancel_junk_file_scan(&scan_id).await
}

#[tauri::command]
pub async fn junk_file_scan_progress(scan_id: String) -> Option<JunkFileScanProgress> {
    get_junk_file_scan_progress(&scan_id).await
}

#[tauri::command]
pub async fn junk_file_scan_result(scan_id: String) -> Option<Vec<JunkScanResult>> {
    get_junk_file_scan_result(&scan_id).await
}

#[tauri::command]
pub async fn junk_file_scan_clear(scan_id: String) -> Result<(), String> {
    clear_junk_file_scan_result(&scan_id).await
}

#[tauri::command]
pub async fn junk_file_category_files(
    scan_id: String,
    file_type: String,
    offset: u64,
    limit: u64,
) -> Option<JunkCategoryFilesResponse> {
    get_junk_category_files(&scan_id, &file_type, offset, limit).await
}

// 确保前端 camelCase 参数名能够正确映射
// Tauri 会自动处理 scanId -> scan_id, fileType -> file_type 的转换
