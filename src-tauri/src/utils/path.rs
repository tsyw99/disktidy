use std::path::{Component, Path, PathBuf};

pub struct PathUtils;

impl PathUtils {
    pub fn normalize(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();

        for component in path.components() {
            match component {
                Component::ParentDir => {
                    if !normalized.pop() {
                        normalized.push(component);
                    }
                }
                Component::CurDir => {}
                _ => normalized.push(component),
            }
        }

        normalized
    }

    pub fn normalize_string(path: &str) -> String {
        Self::normalize(Path::new(path))
            .to_string_lossy()
            .to_string()
    }

    pub fn is_subpath(parent: &Path, child: &Path) -> bool {
        child.starts_with(parent)
    }

    pub fn equals_ignore_case(path1: &Path, path2: &Path) -> bool {
        path1.to_string_lossy().to_lowercase() == path2.to_string_lossy().to_lowercase()
    }

    pub fn matches_pattern(path: &Path, pattern: &str) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let pattern_lower = pattern.to_lowercase();

        if pattern_lower.contains('*') || pattern_lower.contains('?') {
            glob_match::glob_match(&pattern_lower, &path_str)
        } else {
            path_str.contains(&pattern_lower)
        }
    }

    pub fn matches_patterns(path: &Path, patterns: &[String]) -> bool {
        patterns.iter().any(|p| Self::matches_pattern(path, p))
    }

    pub fn get_extension(path: &Path) -> Option<String> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
    }

    pub fn has_extension(path: &Path, extensions: &[&str]) -> bool {
        Self::get_extension(path)
            .map(|ext| extensions.iter().any(|e| e.to_lowercase() == ext))
            .unwrap_or(false)
    }

    pub fn get_filename(path: &Path) -> Option<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
    }

    pub fn get_parent(path: &Path) -> Option<PathBuf> {
        path.parent().map(|p| p.to_path_buf())
    }

    pub fn is_valid_path(path: &str) -> bool {
        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
        !path.chars().any(|c| invalid_chars.contains(&c))
    }

    pub fn is_absolute_path(path: &Path) -> bool {
        path.is_absolute()
    }

    pub fn to_absolute(path: &Path, base: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        }
    }
}

pub struct SystemPaths;

impl SystemPaths {
    pub fn temp_dir() -> PathBuf {
        std::env::temp_dir()
    }

    pub fn home_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }

    pub fn documents_dir() -> Option<PathBuf> {
        dirs::document_dir()
    }

    pub fn app_data_local() -> Option<PathBuf> {
        dirs::cache_dir()
    }

    pub fn app_data_roaming() -> Option<PathBuf> {
        dirs::config_dir()
    }

    pub fn program_files() -> PathBuf {
        PathBuf::from("C:\\Program Files")
    }

    pub fn program_files_x86() -> PathBuf {
        PathBuf::from("C:\\Program Files (x86)")
    }

    pub fn windows_dir() -> PathBuf {
        PathBuf::from("C:\\Windows")
    }

    pub fn system32_dir() -> PathBuf {
        PathBuf::from("C:\\Windows\\System32")
    }

    pub fn get_browser_cache_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(local_app_data) = Self::app_data_local() {
            paths.push(
                local_app_data.join("Google\\Chrome\\User Data\\Default\\Cache"),
            );
            paths.push(
                local_app_data.join("Microsoft\\Edge\\User Data\\Default\\Cache"),
            );
            paths.push(local_app_data.join("Mozilla\\Firefox\\Profiles"));
        }

        paths
    }

    pub fn get_temp_paths() -> Vec<PathBuf> {
        vec![Self::temp_dir(), PathBuf::from("C:\\Windows\\Temp")]
    }

    pub fn get_protected_paths() -> Vec<PathBuf> {
        vec![
            Self::system32_dir(),
            PathBuf::from("C:\\Windows\\SysWOW64"),
            PathBuf::from("C:\\Windows\\WinSxS"),
            Self::program_files(),
            Self::program_files_x86(),
            PathBuf::from("C:\\ProgramData"),
            PathBuf::from("C:\\Boot"),
            PathBuf::from("C:\\EFI"),
            PathBuf::from("C:\\Recovery"),
        ]
    }
}
