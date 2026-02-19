use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

use crate::modules::cleaner::safety::SafetyChecker;
use crate::utils::path::{PathUtils, SystemPaths};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum JunkFileType {
    EmptyFolders,
    InvalidShortcuts,
    OldLogs,
    OldInstallers,
    InvalidDownloads,
    SmallFiles,
    OrphanedFiles,
}

impl JunkFileType {
    pub fn display_name(&self) -> &str {
        match self {
            Self::EmptyFolders => "空文件夹",
            Self::InvalidShortcuts => "无效快捷方式",
            Self::OldLogs => "过期日志文件",
            Self::OldInstallers => "旧版本安装包",
            Self::InvalidDownloads => "无效下载文件",
            Self::SmallFiles => "零散小文件",
            Self::OrphanedFiles => "孤立文件",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::EmptyFolders => "不包含任何文件的空目录",
            Self::InvalidShortcuts => "目标路径不存在的快捷方式",
            Self::OldLogs => "超过指定天数未访问的日志文件",
            Self::OldInstallers => "已安装程序的旧版本安装包",
            Self::InvalidDownloads => "下载目录中未完成或损坏的文件",
            Self::SmallFiles => "小于指定大小的零散文件",
            Self::OrphanedFiles => "不属于任何已知程序的孤立文件",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::EmptyFolders => "folder-x",
            Self::InvalidShortcuts => "link-broken",
            Self::OldLogs => "file-text",
            Self::OldInstallers => "package",
            Self::InvalidDownloads => "download",
            Self::SmallFiles => "file",
            Self::OrphanedFiles => "file-question",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkFile {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub file_type: JunkFileType,
    pub description: String,
    pub modified_time: i64,
    pub created_time: i64,
    pub safe_to_delete: bool,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkScanResult {
    pub file_type: JunkFileType,
    pub items: Vec<JunkFile>,
    pub total_size: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunkScanOptions {
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

impl Default for JunkScanOptions {
    fn default() -> Self {
        Self {
            scan_paths: SystemPaths::home_dir()
                .map(|h| vec![h.to_string_lossy().to_string()])
                .unwrap_or_default(),
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

pub struct JunkFileDetector {
    options: JunkScanOptions,
    safety_checker: SafetyChecker,
    protected_paths: Vec<PathBuf>,
}

impl JunkFileDetector {
    pub fn new() -> Self {
        Self::with_options(JunkScanOptions::default())
    }

    pub fn with_options(options: JunkScanOptions) -> Self {
        Self {
            protected_paths: SystemPaths::get_protected_paths(),
            safety_checker: SafetyChecker::new(),
            options,
        }
    }

    pub fn detect_all(&self) -> Vec<JunkScanResult> {
        let mut results = Vec::new();

        if self.options.include_empty_folders {
            if let Some(result) = self.detect_empty_folders() {
                results.push(result);
            }
        }

        if self.options.include_invalid_shortcuts {
            if let Some(result) = self.detect_invalid_shortcuts() {
                results.push(result);
            }
        }

        if self.options.include_old_logs {
            if let Some(result) = self.detect_old_logs() {
                results.push(result);
            }
        }

        if self.options.include_old_installers {
            if let Some(result) = self.detect_old_installers() {
                results.push(result);
            }
        }

        if self.options.include_invalid_downloads {
            if let Some(result) = self.detect_invalid_downloads() {
                results.push(result);
            }
        }

        if self.options.include_small_files {
            if let Some(result) = self.detect_small_files() {
                results.push(result);
            }
        }

        results
    }

    pub fn detect_by_type(&self, file_type: JunkFileType) -> Option<JunkScanResult> {
        match file_type {
            JunkFileType::EmptyFolders => self.detect_empty_folders(),
            JunkFileType::InvalidShortcuts => self.detect_invalid_shortcuts(),
            JunkFileType::OldLogs => self.detect_old_logs(),
            JunkFileType::OldInstallers => self.detect_old_installers(),
            JunkFileType::InvalidDownloads => self.detect_invalid_downloads(),
            JunkFileType::SmallFiles => self.detect_small_files(),
            JunkFileType::OrphanedFiles => None,
        }
    }

    fn detect_empty_folders(&self) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let scan_paths = self.get_scan_paths();

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_dir());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                if self.is_empty_folder(path) {
                    if let Some(junk_file) = self.create_junk_file(
                        path,
                        0,
                        JunkFileType::EmptyFolders,
                        "空文件夹",
                    ) {
                        items.push(junk_file);
                    }
                }
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

    fn detect_invalid_shortcuts(&self) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let scan_paths = self.get_scan_paths();

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                if self.is_invalid_shortcut(path) {
                    let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                    if let Some(junk_file) = self.create_junk_file(
                        path,
                        size,
                        JunkFileType::InvalidShortcuts,
                        "目标路径不存在的快捷方式",
                    ) {
                        items.push(junk_file);
                    }
                }
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

    fn detect_old_logs(&self) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let scan_paths = self.get_scan_paths();
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - (self.options.log_max_age_days as i64 * 24 * 60 * 60);

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                if self.is_log_file(path) {
                    if let Ok(metadata) = fs::metadata(path) {
                        let modified_time = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        if modified_time < cutoff_time {
                            let size = metadata.len();
                            if let Some(junk_file) = self.create_junk_file(
                                path,
                                size,
                                JunkFileType::OldLogs,
                                &format!("超过 {} 天未修改的日志文件", self.options.log_max_age_days),
                            ) {
                                items.push(junk_file);
                            }
                        }
                    }
                }
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

    fn detect_old_installers(&self) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let scan_paths = self.get_scan_paths();
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
            - (self.options.installer_max_age_days as i64 * 24 * 60 * 60);

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                if self.is_installer_file(path) {
                    if let Ok(metadata) = fs::metadata(path) {
                        let modified_time = metadata
                            .modified()
                            .ok()
                            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64)
                            .unwrap_or(0);

                        if modified_time < cutoff_time {
                            let size = metadata.len();
                            if let Some(junk_file) = self.create_junk_file(
                                path,
                                size,
                                JunkFileType::OldInstallers,
                                &format!(
                                    "超过 {} 天的安装包文件",
                                    self.options.installer_max_age_days
                                ),
                            ) {
                                items.push(junk_file);
                            }
                        }
                    }
                }
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

    fn detect_invalid_downloads(&self) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let download_paths = self.get_download_paths();

        for download_path in &download_paths {
            if !download_path.exists() {
                continue;
            }

            let walker = WalkDir::new(download_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                if self.is_invalid_download(path) {
                    if let Ok(metadata) = fs::metadata(path) {
                        let size = metadata.len();
                        if let Some(junk_file) = self.create_junk_file(
                            path,
                            size,
                            JunkFileType::InvalidDownloads,
                            "未完成或损坏的下载文件",
                        ) {
                            items.push(junk_file);
                        }
                    }
                }
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

    fn detect_small_files(&self) -> Option<JunkScanResult> {
        let mut items = Vec::new();
        let scan_paths = self.get_scan_paths();

        for scan_path in &scan_paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                if let Ok(metadata) = fs::metadata(path) {
                    let size = metadata.len();
                    if size > 0 && size <= self.options.small_file_max_size {
                        if let Some(junk_file) = self.create_junk_file(
                            path,
                            size,
                            JunkFileType::SmallFiles,
                            &format!(
                                "小于 {} KB 的零散文件",
                                self.options.small_file_max_size / 1024
                            ),
                        ) {
                            items.push(junk_file);
                        }
                    }
                }
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

    fn get_scan_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = self
            .options
            .scan_paths
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect();

        if paths.is_empty() {
            if let Some(home) = SystemPaths::home_dir() {
                paths.push(home);
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

    fn should_skip(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for exclude in &self.options.exclude_paths {
            if path_str.to_lowercase().starts_with(&exclude.to_lowercase()) {
                return true;
            }
        }

        for protected in &self.protected_paths {
            if path.starts_with(protected) {
                return true;
            }
        }

        if !self.options.include_hidden {
            if let Ok(metadata) = fs::metadata(path) {
                #[cfg(windows)]
                {
                    let attrs = metadata.file_attributes();
                    if attrs & 0x2 != 0 {
                        return true;
                    }
                }
            }
        }

        if !self.options.include_system {
            if let Ok(metadata) = fs::metadata(path) {
                #[cfg(windows)]
                {
                    let attrs = metadata.file_attributes();
                    if attrs & 0x4 != 0 {
                        return true;
                    }
                }
            }
        }

        false
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

    fn is_invalid_shortcut(&self, path: &Path) -> bool {
        let extension = PathUtils::get_extension(path);
        if extension.as_deref() != Some("lnk") {
            return false;
        }

        #[cfg(windows)]
        {
            if let Ok(target) = self.resolve_shortcut_target(path) {
                return !Path::new(&target).exists();
            }
        }

        false
    }

    #[cfg(windows)]
    fn resolve_shortcut_target(&self, path: &Path) -> Result<String, std::io::Error> {
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER, STGM,
        };
        use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
        use windows::core::{Interface, PCWSTR};

        unsafe {
            CoInitialize(None).ok()?;

            let result = (|| {
                let shell_link: IShellLinkW =
                    CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

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
                    &target[..target.iter().position(|&c| c == 0).unwrap_or(target.len())]
                );

                Ok(target_string)
            })();

            CoUninitialize();
            result
        }
    }

    fn is_log_file(&self, path: &Path) -> bool {
        let extension = PathUtils::get_extension(path);
        matches!(extension.as_deref(), Some("log") | Some("txt") | Some("old"))
            || PathUtils::get_filename(path)
                .map(|name| {
                    name.to_lowercase().ends_with(".log")
                        || name.to_lowercase().ends_with(".txt")
                        || name.to_lowercase().contains("log")
                })
                .unwrap_or(false)
    }

    fn is_installer_file(&self, path: &Path) -> bool {
        let extension = PathUtils::get_extension(path);
        matches!(
            extension.as_deref(),
            Some("exe") | Some("msi") | Some("dmg") | Some("pkg") | Some("deb") | Some("rpm")
        ) && PathUtils::get_filename(path)
            .map(|name| {
                let name_lower = name.to_lowercase();
                name_lower.contains("setup")
                    || name_lower.contains("install")
                    || name_lower.contains("installer")
                    || name_lower.contains("uninstall")
                    || name_lower.contains("v")
                    || name_lower.contains("_")
            })
            .unwrap_or(false)
    }

    fn is_invalid_download(&self, path: &Path) -> bool {
        let filename = PathUtils::get_filename(path);
        if let Some(name) = filename {
            let name_lower = name.to_lowercase();
            if name_lower.ends_with(".part")
                || name_lower.ends_with(".crdownload")
                || name_lower.ends_with(".download")
                || name_lower.ends_with(".tmp")
                || name_lower.ends_with(".temp")
            {
                return true;
            }

            if name_lower.starts_with("unconfirmed")
                || name_lower.starts_with(".com.google.chrome")
            {
                return true;
            }
        }
        false
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
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            created_time: metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            safe_to_delete: safety_result.safe_to_delete,
            risk_level: format!("{:?}", safety_result.risk_level),
        })
    }
}

impl Default for JunkFileDetector {
    fn default() -> Self {
        Self::new()
    }
}
