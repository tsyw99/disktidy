use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use walkdir::WalkDir;

use lazy_static::lazy_static;

use crate::modules::cleaner::safety::SafetyChecker;
use crate::utils::path::{PathUtils, SystemPaths};

lazy_static! {
    static ref KNOWN_APP_PATTERNS: HashMap<String, String> = {
        let mut patterns = HashMap::new();
        patterns.insert("wechat".to_string(), "微信".to_string());
        patterns.insert("tencent".to_string(), "腾讯".to_string());
        patterns.insert("qq".to_string(), "QQ".to_string());
        patterns.insert("dingtalk".to_string(), "钉钉".to_string());
        patterns.insert("wework".to_string(), "企业微信".to_string());
        patterns.insert("adobe".to_string(), "Adobe".to_string());
        patterns.insert("microsoft".to_string(), "微软".to_string());
        patterns.insert("google".to_string(), "Google".to_string());
        patterns.insert("chrome".to_string(), "Chrome".to_string());
        patterns.insert("mozilla".to_string(), "Mozilla".to_string());
        patterns.insert("firefox".to_string(), "Firefox".to_string());
        patterns.insert("steam".to_string(), "Steam".to_string());
        patterns.insert("unity".to_string(), "Unity".to_string());
        patterns.insert("nvidia".to_string(), "NVIDIA".to_string());
        patterns.insert("intel".to_string(), "Intel".to_string());
        patterns.insert("nodejs".to_string(), "Node.js".to_string());
        patterns.insert("python".to_string(), "Python".to_string());
        patterns.insert("java".to_string(), "Java".to_string());
        patterns.insert("docker".to_string(), "Docker".to_string());
        patterns.insert("vscode".to_string(), "VS Code".to_string());
        patterns.insert("visual studio".to_string(), "Visual Studio".to_string());
        patterns.insert("jetbrains".to_string(), "JetBrains".to_string());
        patterns.insert("notepad++".to_string(), "Notepad++".to_string());
        patterns.insert("7-zip".to_string(), "7-Zip".to_string());
        patterns.insert("winrar".to_string(), "WinRAR".to_string());
        patterns.insert("obs".to_string(), "OBS".to_string());
        patterns.insert("spotify".to_string(), "Spotify".to_string());
        patterns.insert("discord".to_string(), "Discord".to_string());
        patterns.insert("slack".to_string(), "Slack".to_string());
        patterns.insert("zoom".to_string(), "Zoom".to_string());
        patterns.insert("teams".to_string(), "Teams".to_string());
        patterns.insert("skype".to_string(), "Skype".to_string());
        patterns.insert("baidu".to_string(), "百度".to_string());
        patterns.insert("360".to_string(), "360".to_string());
        patterns.insert("thunder".to_string(), "迅雷".to_string());
        patterns.insert("netease".to_string(), "网易".to_string());
        patterns.insert("wps".to_string(), "WPS".to_string());
        patterns
    };
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResidueType {
    LeftoverFolder,
    RegistryKey,
    CacheFile,
    ConfigFile,
}

impl ResidueType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::LeftoverFolder => "遗留目录",
            Self::RegistryKey => "注册表项",
            Self::CacheFile => "缓存文件",
            Self::ConfigFile => "配置文件",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::LeftoverFolder => "已卸载软件的残留目录",
            Self::RegistryKey => "已卸载软件的注册表残留项",
            Self::CacheFile => "已卸载软件的缓存文件",
            Self::ConfigFile => "已卸载软件的配置文件",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueItem {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub residue_type: ResidueType,
    pub app_name: String,
    pub description: String,
    pub last_modified: i64,
    pub safe_to_delete: bool,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueScanResult {
    pub residue_type: ResidueType,
    pub items: Vec<ResidueItem>,
    pub total_size: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSoftware {
    pub name: String,
    pub publisher: Option<String>,
    pub install_location: Option<String>,
    pub uninstall_string: Option<String>,
    pub version: Option<String>,
    pub install_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueScanProgress {
    pub percent: f32,
    pub current_phase: String,
    pub current_path: String,
    pub scanned_count: u32,
    pub found_count: u32,
    pub elapsed_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidueScanOptions {
    pub include_leftover_folders: bool,
    pub include_registry_keys: bool,
    pub include_cache_files: bool,
    pub include_config_files: bool,
    pub scan_all_drives: bool,
    pub custom_scan_paths: Vec<String>,
}

impl Default for ResidueScanOptions {
    fn default() -> Self {
        Self {
            include_leftover_folders: true,
            include_registry_keys: true,
            include_cache_files: true,
            include_config_files: true,
            scan_all_drives: true,
            custom_scan_paths: vec![],
        }
    }
}

pub struct SoftwareResidueScanner {
    options: ResidueScanOptions,
    safety_checker: SafetyChecker,
    installed_software: Vec<InstalledSoftware>,
    installed_names: HashSet<String>,
    is_scanning: Arc<AtomicBool>,
    is_paused: Arc<AtomicBool>,
    is_cancelled: Arc<AtomicBool>,
    progress: Arc<RwLock<Option<ResidueScanProgress>>>,
    results: Arc<RwLock<Vec<ResidueScanResult>>>,
    start_time: Arc<RwLock<Option<SystemTime>>>,
    scanned_count: Arc<AtomicU32>,
    found_count: Arc<AtomicU32>,
}

impl SoftwareResidueScanner {
    pub fn new() -> Self {
        Self::with_options(ResidueScanOptions::default())
    }

    pub fn with_options(options: ResidueScanOptions) -> Self {
        let installed_software = Self::get_installed_software();
        let installed_names: HashSet<String> = installed_software
            .iter()
            .filter_map(|s| {
                let name = s.name.to_lowercase();
                if name.is_empty() {
                    None
                } else {
                    Some(name)
                }
            })
            .collect();

        Self {
            options,
            safety_checker: SafetyChecker::new(),
            installed_software,
            installed_names,
            is_scanning: Arc::new(AtomicBool::new(false)),
            is_paused: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
            progress: Arc::new(RwLock::new(None)),
            results: Arc::new(RwLock::new(Vec::new())),
            start_time: Arc::new(RwLock::new(None)),
            scanned_count: Arc::new(AtomicU32::new(0)),
            found_count: Arc::new(AtomicU32::new(0)),
        }
    }

    #[cfg(windows)]
    fn get_installed_software() -> Vec<InstalledSoftware> {
        use winreg::enums::*;
        use winreg::RegKey;

        let mut software_list = Vec::new();
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        let paths = vec![
            "SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            "SOFTWARE\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
        ];

        for path in paths {
            if let Ok(uninstall_key) = hklm.open_subkey(path) {
                for subkey in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
                    if let Ok(app_key) = uninstall_key.open_subkey(&subkey) {
                        let name: String = app_key
                            .get_value("DisplayName")
                            .unwrap_or_default();

                        if !name.is_empty() {
                            let software = InstalledSoftware {
                                name,
                                publisher: app_key.get_value("Publisher").ok(),
                                install_location: app_key.get_value("InstallLocation").ok(),
                                uninstall_string: app_key.get_value("UninstallString").ok(),
                                version: app_key.get_value("DisplayVersion").ok(),
                                install_date: app_key.get_value("InstallDate").ok(),
                            };
                            software_list.push(software);
                        }
                    }
                }
            }
        }

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(uninstall_key) = hkcu.open_subkey("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall") {
            for subkey in uninstall_key.enum_keys().filter_map(|k| k.ok()) {
                if let Ok(app_key) = uninstall_key.open_subkey(&subkey) {
                    let name: String = app_key
                        .get_value("DisplayName")
                        .unwrap_or_default();

                    if !name.is_empty() {
                        let software = InstalledSoftware {
                            name,
                            publisher: app_key.get_value("Publisher").ok(),
                            install_location: app_key.get_value("InstallLocation").ok(),
                            uninstall_string: app_key.get_value("UninstallString").ok(),
                            version: app_key.get_value("DisplayVersion").ok(),
                            install_date: app_key.get_value("InstallDate").ok(),
                        };
                        software_list.push(software);
                    }
                }
            }
        }

        software_list
    }

    #[cfg(not(windows))]
    fn get_installed_software() -> Vec<InstalledSoftware> {
        Vec::new()
    }

    pub async fn start_scan(&self) -> Result<Vec<ResidueScanResult>, String> {
        if self.is_scanning.load(Ordering::SeqCst) {
            return Err("扫描已在进行中".to_string());
        }

        self.is_scanning.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
        self.is_cancelled.store(false, Ordering::SeqCst);
        self.scanned_count.store(0, Ordering::SeqCst);
        self.found_count.store(0, Ordering::SeqCst);

        {
            let mut start_time = self.start_time.write().await;
            *start_time = Some(SystemTime::now());
        }

        let mut results = Vec::new();

        self.update_progress(0.0, "初始化扫描", "").await;

        if self.options.include_leftover_folders {
            if self.check_cancelled() {
                return Ok(vec![]);
            }
            self.update_progress(10.0, "扫描遗留目录", "").await;
            if let Some(result) = self.scan_leftover_folders().await {
                results.push(result);
            }
        }

        if self.options.include_registry_keys {
            if self.check_cancelled() {
                return Ok(vec![]);
            }
            self.update_progress(40.0, "扫描注册表残留", "").await;
            if let Some(result) = self.scan_registry_keys().await {
                results.push(result);
            }
        }

        if self.options.include_cache_files {
            if self.check_cancelled() {
                return Ok(vec![]);
            }
            self.update_progress(60.0, "扫描缓存文件", "").await;
            if let Some(result) = self.scan_cache_files().await {
                results.push(result);
            }
        }

        if self.options.include_config_files {
            if self.check_cancelled() {
                return Ok(vec![]);
            }
            self.update_progress(80.0, "扫描配置文件", "").await;
            if let Some(result) = self.scan_config_files().await {
                results.push(result);
            }
        }

        self.update_progress(100.0, "扫描完成", "").await;

        {
            let mut results_lock = self.results.write().await;
            *results_lock = results.clone();
        }

        self.is_scanning.store(false, Ordering::SeqCst);

        Ok(results)
    }

    fn check_cancelled(&self) -> bool {
        self.is_cancelled.load(Ordering::SeqCst)
    }

    async fn wait_if_paused(&self) {
        while self.is_paused.load(Ordering::SeqCst) && !self.is_cancelled.load(Ordering::SeqCst) {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    async fn update_progress(&self, percent: f32, phase: &str, path: &str) {
        let elapsed = {
            let start_time = self.start_time.read().await;
            start_time
                .map(|t| t.elapsed().unwrap_or_default().as_secs())
                .unwrap_or(0)
        };

        let mut progress = self.progress.write().await;
        *progress = Some(ResidueScanProgress {
            percent,
            current_phase: phase.to_string(),
            current_path: path.to_string(),
            scanned_count: self.scanned_count.load(Ordering::SeqCst),
            found_count: self.found_count.load(Ordering::SeqCst),
            elapsed_time: elapsed,
        });
    }

    async fn scan_leftover_folders(&self) -> Option<ResidueScanResult> {
        let mut items = Vec::new();
        let scan_paths = self.get_scan_paths();

        let known_app_folders = self.get_known_app_folder_patterns();

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .max_depth(2)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_dir());

            for entry in walker {
                if self.check_cancelled() {
                    return None;
                }

                self.wait_if_paused().await;

                let path = entry.path();
                let folder_name = PathUtils::get_filename(path)?;

                if self.is_protected_path(path) {
                    continue;
                }

                if self.is_installed_software_folder(&folder_name) {
                    continue;
                }

                if self.is_potential_leftover_folder(path, &folder_name, &known_app_folders) {
                    if let Some(item) = self.create_residue_item(
                        path,
                        0,
                        ResidueType::LeftoverFolder,
                        &folder_name,
                        "可能为已卸载软件的残留目录",
                    ).await {
                        self.found_count.fetch_add(1, Ordering::SeqCst);
                        items.push(item);
                    }
                }

                self.scanned_count.fetch_add(1, Ordering::SeqCst);
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size: u64 = items.iter().map(|i| i.size).sum();
        Some(ResidueScanResult {
            residue_type: ResidueType::LeftoverFolder,
            items,
            total_size,
            count: self.found_count.load(Ordering::SeqCst) as u64,
        })
    }

    async fn scan_registry_keys(&self) -> Option<ResidueScanResult> {
        #[cfg(windows)]
        {
            let mut items = Vec::new();

            use winreg::enums::*;
            use winreg::RegKey;

            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);

            let software_paths = vec![
                (hklm, "SOFTWARE", "HKLM"),
                (hkcu, "SOFTWARE", "HKCU"),
            ];

            for (root_key, path, root_name) in software_paths {
                if self.check_cancelled() {
                    return None;
                }

                self.wait_if_paused().await;

                if let Ok(software_key) = root_key.open_subkey(path) {
                    for subkey in software_key.enum_keys().filter_map(|k| k.ok()) {
                        if self.check_cancelled() {
                            return None;
                        }

                        let subkey_lower = subkey.to_lowercase();

                        if self.is_installed_software_name(&subkey) {
                            continue;
                        }

                        if self.is_potential_leftover_registry_key(&subkey_lower) {
                            let full_path = format!("{}\\{}\\{}", root_name, path, subkey);
                            let item = ResidueItem {
                                id: Uuid::new_v4().to_string(),
                                name: subkey.clone(),
                                path: full_path.clone(),
                                size: 0,
                                residue_type: ResidueType::RegistryKey,
                                app_name: subkey,
                                description: "可能为已卸载软件的注册表残留项".to_string(),
                                last_modified: 0,
                                safe_to_delete: false,
                                risk_level: "High".to_string(),
                            };
                            self.found_count.fetch_add(1, Ordering::SeqCst);
                            items.push(item);
                        }

                        self.scanned_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }

            if items.is_empty() {
                return None;
            }

            Some(ResidueScanResult {
                residue_type: ResidueType::RegistryKey,
                items,
                total_size: 0,
                count: self.found_count.load(Ordering::SeqCst) as u64,
            })
        }

        #[cfg(not(windows))]
        None
    }

    async fn scan_cache_files(&self) -> Option<ResidueScanResult> {
        let mut items = Vec::new();
        let cache_paths = self.get_cache_scan_paths();

        for cache_path in &cache_paths {
            if !cache_path.exists() {
                continue;
            }

            if self.check_cancelled() {
                return None;
            }

            self.wait_if_paused().await;

            if let Ok(entries) = fs::read_dir(cache_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let folder_name = PathUtils::get_filename(&path).unwrap_or_default();

                        if self.is_protected_path(&path) {
                            continue;
                        }

                        if self.is_installed_software_folder(&folder_name) {
                            continue;
                        }

                        if self.is_potential_leftover_cache(&folder_name) {
                            if let Some(item) = self.create_residue_item(
                                &path,
                                0,
                                ResidueType::CacheFile,
                                &folder_name,
                                "可能为已卸载软件的缓存文件",
                            ).await {
                                self.found_count.fetch_add(1, Ordering::SeqCst);
                                items.push(item);
                            }
                        }
                    }

                    self.scanned_count.fetch_add(1, Ordering::SeqCst);
                }
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size: u64 = items.iter().map(|i| i.size).sum();
        Some(ResidueScanResult {
            residue_type: ResidueType::CacheFile,
            items,
            total_size,
            count: self.found_count.load(Ordering::SeqCst) as u64,
        })
    }

    async fn scan_config_files(&self) -> Option<ResidueScanResult> {
        let mut items = Vec::new();
        let config_paths = self.get_config_scan_paths();

        for config_path in &config_paths {
            if !config_path.exists() {
                continue;
            }

            if self.check_cancelled() {
                return None;
            }

            self.wait_if_paused().await;

            if let Ok(entries) = fs::read_dir(config_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        let folder_name = PathUtils::get_filename(&path).unwrap_or_default();

                        if self.is_protected_path(&path) {
                            continue;
                        }

                        if self.is_installed_software_folder(&folder_name) {
                            continue;
                        }

                        if self.is_potential_leftover_config(&folder_name) {
                            if let Some(item) = self.create_residue_item(
                                &path,
                                0,
                                ResidueType::ConfigFile,
                                &folder_name,
                                "可能为已卸载软件的配置文件",
                            ).await {
                                self.found_count.fetch_add(1, Ordering::SeqCst);
                                items.push(item);
                            }
                        }
                    }

                    self.scanned_count.fetch_add(1, Ordering::SeqCst);
                }
            }
        }

        if items.is_empty() {
            return None;
        }

        let total_size: u64 = items.iter().map(|i| i.size).sum();
        Some(ResidueScanResult {
            residue_type: ResidueType::ConfigFile,
            items,
            total_size,
            count: self.found_count.load(Ordering::SeqCst) as u64,
        })
    }

    fn get_scan_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        paths.push(SystemPaths::program_files());
        paths.push(SystemPaths::program_files_x86());
        paths.push(PathBuf::from("C:\\ProgramData"));

        if let Some(home) = SystemPaths::home_dir() {
            paths.push(home.clone());
        }

        for custom_path in &self.options.custom_scan_paths {
            paths.push(PathBuf::from(custom_path));
        }

        paths.retain(|p| p.exists());
        paths
    }

    fn get_cache_scan_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            paths.push(local_app_data.clone());
        }

        paths.retain(|p| p.exists());
        paths
    }

    fn get_config_scan_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(roaming_app_data) = SystemPaths::app_data_roaming() {
            paths.push(roaming_app_data);
        }

        if let Some(local_app_data) = SystemPaths::app_data_local() {
            paths.push(local_app_data);
        }

        paths.retain(|p| p.exists());
        paths
    }

    fn get_known_app_folder_patterns(&self) -> &'static HashMap<String, String> {
        &KNOWN_APP_PATTERNS
    }

    fn is_installed_software_folder(&self, folder_name: &str) -> bool {
        let name_lower = folder_name.to_lowercase();

        for installed_name in &self.installed_names {
            if name_lower.contains(installed_name) || installed_name.contains(&name_lower) {
                return true;
            }
        }

        false
    }

    fn is_installed_software_name(&self, name: &str) -> bool {
        let name_lower = name.to_lowercase();

        for installed_name in &self.installed_names {
            if name_lower == *installed_name {
                return true;
            }
        }

        false
    }

    fn is_potential_leftover_folder(
        &self,
        path: &Path,
        folder_name: &str,
        known_patterns: &'static HashMap<String, String>,
    ) -> bool {
        let name_lower = folder_name.to_lowercase();

        for (pattern, _) in known_patterns {
            if name_lower.contains(pattern.as_str()) {
                if !self.is_installed_software_folder(folder_name) {
                    return true;
                }
            }
        }

        if let Ok(entries) = fs::read_dir(path) {
            let entry_count = entries.count();
            if entry_count == 0 {
                return true;
            }
        }

        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > 180 * 24 * 60 * 60 {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn is_potential_leftover_registry_key(&self, key_name: &str) -> bool {
        let known_patterns = self.get_known_app_folder_patterns();

        for (pattern, _) in known_patterns {
            if key_name.contains(pattern.as_str()) {
                return true;
            }
        }

        false
    }

    fn is_potential_leftover_cache(&self, folder_name: &str) -> bool {
        let name_lower = folder_name.to_lowercase();
        let known_patterns = self.get_known_app_folder_patterns();

        for (pattern, _) in known_patterns {
            if name_lower.contains(pattern.as_str()) {
                if !self.is_installed_software_folder(folder_name) {
                    return true;
                }
            }
        }

        false
    }

    fn is_potential_leftover_config(&self, folder_name: &str) -> bool {
        let name_lower = folder_name.to_lowercase();
        let known_patterns = self.get_known_app_folder_patterns();

        for (pattern, _) in known_patterns {
            if name_lower.contains(pattern.as_str()) {
                if !self.is_installed_software_folder(folder_name) {
                    return true;
                }
            }
        }

        false
    }

    fn is_protected_path(&self, path: &Path) -> bool {
        let protected = SystemPaths::get_protected_paths();
        let path_str = path.to_string_lossy().to_lowercase();

        for p in protected {
            if path_str.starts_with(&p.to_string_lossy().to_lowercase()) {
                return true;
            }
        }

        let system_folders = [
            "windows", "system32", "syswow64", "winsxs", "microsoft", "windowsapps",
            "program files", "program files (x86)", "programdata",
        ];

        for folder in system_folders {
            if path_str.contains(folder) {
                return true;
            }
        }

        false
    }

    async fn create_residue_item(
        &self,
        path: &Path,
        size: u64,
        residue_type: ResidueType,
        app_name: &str,
        description: &str,
    ) -> Option<ResidueItem> {
        let metadata = fs::metadata(path).ok()?;
        let safety_result = self.safety_checker.check(path);

        let actual_size = if size == 0 && path.is_dir() {
            self.calculate_folder_size(path).await
        } else {
            size
        };

        Some(ResidueItem {
            id: Uuid::new_v4().to_string(),
            name: PathUtils::get_filename(path)?,
            path: path.to_string_lossy().to_string(),
            size: actual_size,
            residue_type,
            app_name: app_name.to_string(),
            description: description.to_string(),
            last_modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            safe_to_delete: safety_result.safe_to_delete,
            risk_level: format!("{:?}", safety_result.risk_level),
        })
    }

    async fn calculate_folder_size(&self, path: &Path) -> u64 {
        let mut total_size = 0;

        let walker = WalkDir::new(path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in walker {
            if let Ok(metadata) = entry.metadata() {
                total_size += metadata.len();
            }
        }

        total_size
    }

    pub fn pause_scan(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
    }

    pub fn resume_scan(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub fn cancel_scan(&self) {
        self.is_cancelled.store(true, Ordering::SeqCst);
        self.is_paused.store(false, Ordering::SeqCst);
    }

    pub async fn get_progress(&self) -> Option<ResidueScanProgress> {
        self.progress.read().await.clone()
    }

    pub async fn get_results(&self) -> Vec<ResidueScanResult> {
        self.results.read().await.clone()
    }

    pub fn is_scanning(&self) -> bool {
        self.is_scanning.load(Ordering::SeqCst)
    }

    pub fn get_installed_software_list(&self) -> &[InstalledSoftware] {
        &self.installed_software
    }
}

impl Default for SoftwareResidueScanner {
    fn default() -> Self {
        Self::new()
    }
}
