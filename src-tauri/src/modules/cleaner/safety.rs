use std::path::{Path, PathBuf};
use crate::models::RiskLevel;
use crate::utils::path::SystemPaths;

#[derive(Debug, Clone)]
pub struct SafetyCheckResult {
    pub safe_to_delete: bool,
    pub risk_level: RiskLevel,
    pub reason: Option<String>,
}

pub struct SafetyChecker {
    protected_paths: Vec<PathBuf>,
    protected_extensions: Vec<String>,
    sensitive_patterns: Vec<String>,
}

impl SafetyChecker {
    pub fn new() -> Self {
        Self {
            protected_paths: Self::get_default_protected_paths(),
            protected_extensions: Self::get_default_protected_extensions(),
            sensitive_patterns: Self::get_default_sensitive_patterns(),
        }
    }

    pub fn check(&self, path: &Path) -> SafetyCheckResult {
        if self.is_protected_path(path) {
            return SafetyCheckResult {
                safe_to_delete: false,
                risk_level: RiskLevel::Critical,
                reason: Some("系统受保护路径".to_string()),
            };
        }

        if self.is_protected_extension(path) {
            return SafetyCheckResult {
                safe_to_delete: false,
                risk_level: RiskLevel::High,
                reason: Some("受保护的文件类型".to_string()),
            };
        }

        if self.is_sensitive_file(path) {
            return SafetyCheckResult {
                safe_to_delete: false,
                risk_level: RiskLevel::High,
                reason: Some("敏感文件".to_string()),
            };
        }

        let risk_level = self.assess_risk_level(path);

        SafetyCheckResult {
            safe_to_delete: risk_level != RiskLevel::Critical,
            risk_level,
            reason: None,
        }
    }

    pub fn is_safe_to_delete(&self, path: &Path) -> bool {
        !self.is_protected_path(path) && !self.is_protected_extension(path) && !self.is_sensitive_file(path)
    }

    fn is_protected_path(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        self.protected_paths.iter().any(|p| {
            let p_str = p.to_string_lossy().to_lowercase();
            path_str.starts_with(&p_str)
        })
    }

    fn is_protected_extension(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| {
                let ext_str = ext.to_string_lossy().to_lowercase();
                self.protected_extensions.contains(&ext_str)
            })
            .unwrap_or(false)
    }

    fn is_sensitive_file(&self, path: &Path) -> bool {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        self.sensitive_patterns.iter().any(|p| file_name.contains(&p.to_lowercase()))
    }

    fn assess_risk_level(&self, path: &Path) -> RiskLevel {
        let path_str = path.to_string_lossy().to_lowercase();

        if path_str.contains("desktop") || path_str.contains("documents") {
            return RiskLevel::Medium;
        }

        if path_str.contains("downloads") {
            return RiskLevel::Low;
        }

        if path_str.contains("temp") || path_str.contains("cache") {
            return RiskLevel::Low;
        }

        RiskLevel::Medium
    }

    fn get_default_protected_paths() -> Vec<PathBuf> {
        SystemPaths::get_protected_paths()
    }

    fn get_default_protected_extensions() -> Vec<String> {
        vec![
            "sys".to_string(),
            "dll".to_string(),
            "exe".to_string(),
            "bat".to_string(),
            "cmd".to_string(),
            "reg".to_string(),
            "ini".to_string(),
            "drv".to_string(),
        ]
    }

    fn get_default_sensitive_patterns() -> Vec<String> {
        vec![
            "password".to_string(),
            "credential".to_string(),
            "secret".to_string(),
            "key".to_string(),
            "token".to_string(),
            "wallet".to_string(),
            "backup".to_string(),
        ]
    }

    pub fn add_protected_path(&mut self, path: PathBuf) {
        if !self.protected_paths.contains(&path) {
            self.protected_paths.push(path);
        }
    }

    pub fn remove_protected_path(&mut self, path: &Path) {
        self.protected_paths.retain(|p| p != path);
    }

    pub fn add_protected_extension(&mut self, ext: String) {
        let ext_lower = ext.to_lowercase();
        if !self.protected_extensions.contains(&ext_lower) {
            self.protected_extensions.push(ext_lower);
        }
    }

    pub fn remove_protected_extension(&mut self, ext: &str) {
        let ext_lower = ext.to_lowercase();
        self.protected_extensions.retain(|e| e != &ext_lower);
    }

    pub fn add_sensitive_pattern(&mut self, pattern: String) {
        let pattern_lower = pattern.to_lowercase();
        if !self.sensitive_patterns.contains(&pattern_lower) {
            self.sensitive_patterns.push(pattern_lower);
        }
    }

    pub fn remove_sensitive_pattern(&mut self, pattern: &str) {
        let pattern_lower = pattern.to_lowercase();
        self.sensitive_patterns.retain(|p| p != &pattern_lower);
    }

    pub fn get_protected_paths(&self) -> &[PathBuf] {
        &self.protected_paths
    }

    pub fn get_protected_extensions(&self) -> &[String] {
        &self.protected_extensions
    }

    pub fn get_sensitive_patterns(&self) -> &[String] {
        &self.sensitive_patterns
    }
}

impl Default for SafetyChecker {
    fn default() -> Self {
        Self::new()
    }
}
