use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::path::SystemPaths;

const APP_PATHS_CONFIG_FILE: &str = "app_paths.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPathsConfig {
    pub paths: HashMap<String, String>,
}

impl Default for AppPathsConfig {
    fn default() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }
}

impl AppPathsConfig {
    pub fn new() -> Self {
        Self::default()
    }

    fn get_config_dir() -> Option<PathBuf> {
        SystemPaths::app_data_local().map(|p| p.join("DiskTidy"))
    }

    fn get_config_file() -> Option<PathBuf> {
        Self::get_config_dir().map(|d| d.join(APP_PATHS_CONFIG_FILE))
    }

    pub fn load() -> Self {
        if let Some(config_file) = Self::get_config_file() {
            if config_file.exists() {
                if let Ok(content) = fs::read_to_string(&config_file) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), String> {
        if let Some(config_dir) = Self::get_config_dir() {
            if !config_dir.exists() {
                fs::create_dir_all(&config_dir)
                    .map_err(|e| format!("Failed to create config directory: {}", e))?;
            }

            let config_file = config_dir.join(APP_PATHS_CONFIG_FILE);
            let content = serde_json::to_string_pretty(self)
                .map_err(|e| format!("Failed to serialize config: {}", e))?;

            fs::write(&config_file, content)
                .map_err(|e| format!("Failed to write config file: {}", e))?;
        }
        Ok(())
    }

    pub fn get_path(&self, app: &str) -> Option<PathBuf> {
        self.paths.get(app).map(|p| PathBuf::from(p))
    }

    pub fn set_path(&mut self, app: &str, path: &str) {
        self.paths.insert(app.to_string(), path.to_string());
    }

    pub fn remove_path(&mut self, app: &str) {
        self.paths.remove(app);
    }

    pub fn has_path(&self, app: &str) -> bool {
        self.paths.contains_key(app)
    }

    pub fn is_configured(&self, apps: &[&str]) -> bool {
        apps.iter().all(|app| self.paths.contains_key(*app))
    }

    pub fn get_configured_apps(&self) -> Vec<String> {
        self.paths.keys().cloned().collect()
    }
}

pub fn get_app_paths_config() -> AppPathsConfig {
    AppPathsConfig::load()
}

pub fn save_app_paths_config(config: &AppPathsConfig) -> Result<(), String> {
    config.save()
}

pub fn get_app_path(app: &str) -> Option<PathBuf> {
    let config = AppPathsConfig::load();
    config.get_path(app)
}

pub fn set_app_path(app: &str, path: &str) -> Result<(), String> {
    let mut config = AppPathsConfig::load();
    config.set_path(app, path);
    config.save()
}

pub fn is_app_configured(app: &str) -> bool {
    let config = AppPathsConfig::load();
    config.has_path(app)
}
