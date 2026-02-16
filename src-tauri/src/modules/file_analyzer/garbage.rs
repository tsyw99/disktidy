use crate::models::cleaner::{
    CategoryStats, GarbageAnalysisResult, GarbageCategory, GarbageFile, RiskLevel,
};
use crate::utils::path::{PathUtils, SystemPaths};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const TEMP_FILE_EXTENSIONS: &[&str] = &["tmp", "temp", "bak", "old", "swp"];
const LOG_FILE_EXTENSIONS: &[&str] = &["log", "logs"];
const CACHE_FOLDER_NAMES: &[&str] = &["cache", "caches", "temp", "tmp", "log", "logs"];

#[derive(Debug, Clone)]
pub struct GarbageDetectorOptions {
    pub include_system_temp: bool,
    pub include_browser_cache: bool,
    pub include_app_cache: bool,
    pub include_recycle_bin: bool,
    pub include_log_files: bool,
    pub min_file_age_days: u32,
    pub max_files_per_category: Option<usize>,
}

impl Default for GarbageDetectorOptions {
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

#[derive(Debug, Clone)]
pub struct BrowserInfo {
    pub name: String,
    pub cache_paths: Vec<PathBuf>,
    pub cookie_paths: Vec<PathBuf>,
    pub history_paths: Vec<PathBuf>,
}

impl BrowserInfo {
    pub fn chrome() -> Self {
        let mut cache_paths: Vec<PathBuf> = Vec::new();
        let mut cookie_paths: Vec<PathBuf> = Vec::new();
        let mut history_paths: Vec<PathBuf> = Vec::new();

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            let chrome_base: PathBuf = local_app_data.join("Google/Chrome/User Data");
            cache_paths.push(chrome_base.join("Default/Cache"));
            cache_paths.push(chrome_base.join("Default/Code Cache"));
            cache_paths.push(chrome_base.join("Default/GPUCache"));
            cache_paths.push(chrome_base.join("Default/Service Worker/Cache"));
            cookie_paths.push(chrome_base.join("Default/Cookies"));
            cookie_paths.push(chrome_base.join("Default/Cookies-journal"));
            history_paths.push(chrome_base.join("Default/History"));
            history_paths.push(chrome_base.join("Default/History-journal"));
        }

        Self {
            name: "Google Chrome".to_string(),
            cache_paths,
            cookie_paths,
            history_paths,
        }
    }

    pub fn edge() -> Self {
        let mut cache_paths: Vec<PathBuf> = Vec::new();
        let mut cookie_paths: Vec<PathBuf> = Vec::new();
        let mut history_paths: Vec<PathBuf> = Vec::new();

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            let edge_base: PathBuf = local_app_data.join("Microsoft/Edge/User Data");
            cache_paths.push(edge_base.join("Default/Cache"));
            cache_paths.push(edge_base.join("Default/Code Cache"));
            cache_paths.push(edge_base.join("Default/GPUCache"));
            cookie_paths.push(edge_base.join("Default/Cookies"));
            cookie_paths.push(edge_base.join("Default/Cookies-journal"));
            history_paths.push(edge_base.join("Default/History"));
        }

        Self {
            name: "Microsoft Edge".to_string(),
            cache_paths,
            cookie_paths,
            history_paths,
        }
    }

    pub fn firefox() -> Self {
        let mut cache_paths: Vec<PathBuf> = Vec::new();
        let mut cookie_paths: Vec<PathBuf> = Vec::new();
        let mut history_paths: Vec<PathBuf> = Vec::new();

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            let firefox_base: PathBuf = local_app_data.join("Mozilla/Firefox/Profiles");
            if firefox_base.exists() {
                if let Ok(entries) = fs::read_dir(&firefox_base) {
                    for entry in entries.flatten() {
                        let profile_path: PathBuf = entry.path();
                        if profile_path.is_dir() {
                            cache_paths.push(profile_path.join("cache2"));
                            cookie_paths.push(profile_path.join("cookies.sqlite"));
                            history_paths.push(profile_path.join("places.sqlite"));
                        }
                    }
                }
            }
        }

        Self {
            name: "Mozilla Firefox".to_string(),
            cache_paths,
            cookie_paths,
            history_paths,
        }
    }

    pub fn all_browsers() -> Vec<Self> {
        vec![Self::chrome(), Self::edge(), Self::firefox()]
    }
}

pub struct GarbageDetector {
    options: GarbageDetectorOptions,
    protected_paths: Vec<PathBuf>,
}

impl GarbageDetector {
    pub fn new() -> Self {
        Self {
            options: GarbageDetectorOptions::default(),
            protected_paths: SystemPaths::get_protected_paths(),
        }
    }

    pub fn with_options(options: GarbageDetectorOptions) -> Self {
        Self {
            options,
            protected_paths: SystemPaths::get_protected_paths(),
        }
    }

    pub fn detect_all(&self) -> GarbageAnalysisResult {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let mut categories: HashMap<String, CategoryStats> = HashMap::new();
        let mut total_files = 0u64;
        let mut total_size = 0u64;
        let mut high_risk_count = 0u64;

        if self.options.include_system_temp {
            let files = self.detect_system_temp();
            let stats = self.build_category_stats(files);
            total_files += stats.file_count;
            total_size += stats.total_size;
            high_risk_count += stats.files.iter().filter(|f| f.risk_level >= RiskLevel::High).count() as u64;
            categories.insert("systemTemp".to_string(), stats);
        }

        if self.options.include_browser_cache {
            let files = self.detect_browser_cache();
            let stats = self.build_category_stats(files);
            total_files += stats.file_count;
            total_size += stats.total_size;
            high_risk_count += stats.files.iter().filter(|f| f.risk_level >= RiskLevel::High).count() as u64;
            categories.insert("browserCache".to_string(), stats);
        }

        if self.options.include_app_cache {
            let files = self.detect_app_cache();
            let stats = self.build_category_stats(files);
            total_files += stats.file_count;
            total_size += stats.total_size;
            high_risk_count += stats.files.iter().filter(|f| f.risk_level >= RiskLevel::High).count() as u64;
            categories.insert("appCache".to_string(), stats);
        }

        if self.options.include_recycle_bin {
            let files = self.detect_recycle_bin();
            let stats = self.build_category_stats(files);
            total_files += stats.file_count;
            total_size += stats.total_size;
            high_risk_count += stats.files.iter().filter(|f| f.risk_level >= RiskLevel::High).count() as u64;
            categories.insert("recycleBin".to_string(), stats);
        }

        if self.options.include_log_files {
            let files = self.detect_log_files();
            let stats = self.build_category_stats(files);
            total_files += stats.file_count;
            total_size += stats.total_size;
            high_risk_count += stats.files.iter().filter(|f| f.risk_level >= RiskLevel::High).count() as u64;
            categories.insert("logFile".to_string(), stats);
        }

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        GarbageAnalysisResult {
            scan_id: uuid::Uuid::new_v4().to_string(),
            total_files,
            total_size,
            categories,
            high_risk_count,
            duration_ms: end_time - start_time,
        }
    }

    pub fn detect_system_temp(&self) -> Vec<GarbageFile> {
        let mut files: Vec<GarbageFile> = Vec::new();
        let temp_paths: Vec<PathBuf> = SystemPaths::get_temp_paths();

        for temp_path in temp_paths {
            if temp_path.exists() {
                self.scan_directory(&temp_path, GarbageCategory::SystemTemp, &mut files);
            }
        }

        self.apply_file_limit(files)
    }

    pub fn detect_browser_cache(&self) -> Vec<GarbageFile> {
        let mut files: Vec<GarbageFile> = Vec::new();

        for browser in BrowserInfo::all_browsers() {
            for cache_path in &browser.cache_paths {
                if cache_path.exists() {
                    self.scan_directory(cache_path, GarbageCategory::BrowserCache, &mut files);
                }
            }
        }

        self.apply_file_limit(files)
    }

    pub fn detect_app_cache(&self) -> Vec<GarbageFile> {
        let mut files: Vec<GarbageFile> = Vec::new();

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            if let Ok(entries) = fs::read_dir(&local_app_data) {
                for entry in entries.flatten() {
                    let app_path: PathBuf = entry.path();
                    if app_path.is_dir() {
                        for cache_name in CACHE_FOLDER_NAMES {
                            let cache_dir: PathBuf = app_path.join(cache_name);
                            if cache_dir.exists() && !self.is_protected_path(&cache_dir) {
                                self.scan_directory(&cache_dir, GarbageCategory::AppCache, &mut files);
                            }
                        }
                    }
                }
            }
        }

        if let Some(roaming_app_data) = SystemPaths::app_data_roaming() {
            if let Ok(entries) = fs::read_dir(&roaming_app_data) {
                for entry in entries.flatten() {
                    let app_path: PathBuf = entry.path();
                    if app_path.is_dir() {
                        for cache_name in CACHE_FOLDER_NAMES {
                            let cache_dir: PathBuf = app_path.join(cache_name);
                            if cache_dir.exists() && !self.is_protected_path(&cache_dir) {
                                self.scan_directory(&cache_dir, GarbageCategory::AppCache, &mut files);
                            }
                        }
                    }
                }
            }
        }

        self.apply_file_limit(files)
    }

    pub fn detect_recycle_bin(&self) -> Vec<GarbageFile> {
        let mut files: Vec<GarbageFile> = Vec::new();

        let recycle_bin_paths: Vec<PathBuf> = self.get_recycle_bin_paths();
        for recycle_path in recycle_bin_paths {
            if recycle_path.exists() {
                self.scan_directory(&recycle_path, GarbageCategory::RecycleBin, &mut files);
            }
        }

        self.apply_file_limit(files)
    }

    pub fn detect_log_files(&self) -> Vec<GarbageFile> {
        let mut files: Vec<GarbageFile> = Vec::new();

        let log_search_paths: Vec<PathBuf> = self.get_log_search_paths();
        for search_path in log_search_paths {
            if search_path.exists() {
                self.scan_log_directory(&search_path, &mut files);
            }
        }

        self.apply_file_limit(files)
    }

    fn scan_directory(&self, dir: &Path, category: GarbageCategory, files: &mut Vec<GarbageFile>) {
        if self.is_protected_path(dir) {
            return;
        }

        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_dir() {
                self.scan_directory(&path, category.clone(), files);
            } else {
                if let Some(garbage_file) = self.create_garbage_file(&path, category.clone()) {
                    files.push(garbage_file);
                }
            }
        }
    }

    fn scan_log_directory(&self, dir: &Path, files: &mut Vec<GarbageFile>) {
        if self.is_protected_path(dir) {
            return;
        }

        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(_) => return,
        };

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_dir() {
                let dir_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                if dir_name == "logs" || dir_name == "log" {
                    self.scan_directory(&path, GarbageCategory::LogFile, files);
                } else {
                    self.scan_log_directory(&path, files);
                }
            } else if PathUtils::has_extension(&path, LOG_FILE_EXTENSIONS) {
                if let Some(garbage_file) = self.create_garbage_file(&path, GarbageCategory::LogFile) {
                    files.push(garbage_file);
                }
            }
        }
    }

    fn create_garbage_file(&self, path: &Path, category: GarbageCategory) -> Option<GarbageFile> {
        let metadata = fs::metadata(path).ok()?;

        let size = metadata.len();
        let modified_time = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let accessed_time = metadata
            .accessed()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if self.options.min_file_age_days > 0 {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let file_age_days = (now as i64 - modified_time) / 86400;
            if file_age_days < self.options.min_file_age_days as i64 {
                return None;
            }
        }

        let risk_level = self.assess_risk_level(path, &category);
        let safe_to_delete = risk_level < RiskLevel::Critical && !self.is_protected_path(path);

        Some(GarbageFile {
            path: path.to_string_lossy().to_string(),
            size,
            category,
            safe_to_delete,
            risk_level,
            modified_time,
            accessed_time,
        })
    }

    fn assess_risk_level(&self, path: &Path, category: &GarbageCategory) -> RiskLevel {
        if self.is_protected_path(path) {
            return RiskLevel::Critical;
        }

        let extension = PathUtils::get_extension(path);
        let filename = PathUtils::get_filename(path).unwrap_or_default().to_lowercase();

        if extension.as_deref() == Some("sys")
            || extension.as_deref() == Some("dll")
            || extension.as_deref() == Some("exe")
        {
            return RiskLevel::Critical;
        }

        if filename.contains("config") || filename.contains("settings") || filename.contains("preferences") {
            return RiskLevel::High;
        }

        if filename.starts_with(".") || filename.starts_with("~") {
            return RiskLevel::Medium;
        }

        match category {
            GarbageCategory::SystemTemp => {
                if PathUtils::has_extension(path, TEMP_FILE_EXTENSIONS) {
                    RiskLevel::Low
                } else {
                    RiskLevel::Medium
                }
            }
            GarbageCategory::BrowserCache => RiskLevel::Low,
            GarbageCategory::AppCache => RiskLevel::Medium,
            GarbageCategory::RecycleBin => RiskLevel::Low,
            GarbageCategory::LogFile => RiskLevel::Low,
            GarbageCategory::Other => RiskLevel::Medium,
        }
    }

    fn is_protected_path(&self, path: &Path) -> bool {
        let path_lower = path.to_string_lossy().to_lowercase();
        
        for protected in &self.protected_paths {
            if PathUtils::is_subpath(protected, path) || PathUtils::equals_ignore_case(path, protected) {
                return true;
            }
        }

        let protected_patterns = [
            "windows\\system32",
            "windows\\syswow64",
            "program files",
            "program files (x86)",
            "programdata",
        ];

        for pattern in &protected_patterns {
            if path_lower.contains(pattern) {
                return true;
            }
        }

        false
    }

    fn get_recycle_bin_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();

        paths.push(PathBuf::from("C:\\$Recycle.Bin"));

        if let Some(home) = SystemPaths::home_dir() {
            let sid_path: PathBuf = home.join("$Recycle.Bin");
            if sid_path.exists() {
                paths.push(sid_path);
            }
        }

        paths
    }

    fn get_log_search_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            paths.push(local_app_data);
        }

        if let Some(roaming_app_data) = SystemPaths::app_data_roaming() {
            paths.push(roaming_app_data);
        }

        paths.push(SystemPaths::temp_dir());
        paths.push(SystemPaths::windows_dir());

        paths
    }

    fn build_category_stats(&self, files: Vec<GarbageFile>) -> CategoryStats {
        let file_count = files.len() as u64;
        let total_size = files.iter().map(|f| f.size).sum();

        CategoryStats {
            category: files.first().map(|f| f.category.display_name().to_string()).unwrap_or_default(),
            file_count,
            total_size,
            files,
        }
    }

    fn apply_file_limit(&self, mut files: Vec<GarbageFile>) -> Vec<GarbageFile> {
        if let Some(max) = self.options.max_files_per_category {
            if files.len() > max {
                files.sort_by(|a, b| b.size.cmp(&a.size));
                files.truncate(max);
            }
        }
        files
    }
}

impl Default for GarbageDetector {
    fn default() -> Self {
        Self::new()
    }
}
