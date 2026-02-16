use serde::{Deserialize, Serialize};

use super::cleaner::CleanMode;
use super::file_analyzer::JunkCategory;
use super::scan::ScanMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub auto_scan: bool,
    pub scan_on_startup: bool,
    pub default_scan_mode: ScanMode,
    pub default_clean_mode: CleanMode,
    pub confirm_before_clean: bool,
    pub show_notifications: bool,
    pub language: String,
    pub theme: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            auto_scan: false,
            scan_on_startup: false,
            default_scan_mode: ScanMode::Quick,
            default_clean_mode: CleanMode::MoveToTrash,
            confirm_before_clean: true,
            show_notifications: true,
            language: "zh-CN".to_string(),
            theme: "dark".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub category: JunkCategory,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistSettings {
    pub enabled: bool,
    pub paths: Vec<WhitelistPath>,
    pub extensions: Vec<WhitelistExtension>,
    pub patterns: Vec<WhitelistPattern>,
}

impl Default for WhitelistSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            paths: Vec::new(),
            extensions: Vec::new(),
            patterns: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistPath {
    pub path: String,
    pub description: String,
    pub enabled: bool,
    pub added_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistExtension {
    pub extension: String,
    pub description: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistPattern {
    pub pattern: String,
    pub description: String,
    pub enabled: bool,
}

impl WhitelistSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_path(&mut self, path: String, description: String) {
        self.paths.push(WhitelistPath {
            path,
            description,
            enabled: true,
            added_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
    }

    pub fn add_extension(&mut self, extension: String, description: String) {
        let ext = if extension.starts_with('.') {
            extension
        } else {
            format!(".{}", extension)
        };
        self.extensions.push(WhitelistExtension {
            extension: ext,
            description,
            enabled: true,
        });
    }

    pub fn add_pattern(&mut self, pattern: String, description: String) {
        self.patterns.push(WhitelistPattern {
            pattern,
            description,
            enabled: true,
        });
    }

    pub fn is_path_whitelisted(&self, path: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.paths
            .iter()
            .any(|p| p.enabled && path.to_lowercase().starts_with(&p.path.to_lowercase()))
    }

    pub fn is_extension_whitelisted(&self, extension: &str) -> bool {
        if !self.enabled {
            return false;
        }
        let ext = extension.to_lowercase();
        self.extensions
            .iter()
            .any(|e| e.enabled && e.extension.to_lowercase() == ext)
    }

    pub fn matches_pattern(&self, filename: &str) -> bool {
        if !self.enabled {
            return false;
        }
        self.patterns.iter().any(|p| {
            p.enabled && glob_match::glob_match(&p.pattern, filename)
        })
    }

    pub fn remove_path(&mut self, path: &str) -> bool {
        let initial_len = self.paths.len();
        self.paths.retain(|p| p.path != path);
        self.paths.len() != initial_len
    }

    pub fn remove_extension(&mut self, extension: &str) -> bool {
        let ext = if extension.starts_with('.') {
            extension.to_string()
        } else {
            format!(".{}", extension)
        };
        let initial_len = self.extensions.len();
        self.extensions.retain(|e| e.extension != ext);
        self.extensions.len() != initial_len
    }

    pub fn remove_pattern(&mut self, pattern: &str) -> bool {
        let initial_len = self.patterns.len();
        self.patterns.retain(|p| p.pattern != pattern);
        self.patterns.len() != initial_len
    }

    pub fn toggle_path(&mut self, path: &str) -> bool {
        if let Some(p) = self.paths.iter_mut().find(|p| p.path == path) {
            p.enabled = !p.enabled;
            p.enabled
        } else {
            false
        }
    }

    pub fn toggle_extension(&mut self, extension: &str) -> bool {
        let ext = if extension.starts_with('.') {
            extension.to_string()
        } else {
            format!(".{}", extension)
        };
        if let Some(e) = self.extensions.iter_mut().find(|e| e.extension == ext) {
            e.enabled = !e.enabled;
            e.enabled
        } else {
            false
        }
    }

    pub fn toggle_pattern(&mut self, pattern: &str) -> bool {
        if let Some(p) = self.patterns.iter_mut().find(|p| p.pattern == pattern) {
            p.enabled = !p.enabled;
            p.enabled
        } else {
            false
        }
    }
}
