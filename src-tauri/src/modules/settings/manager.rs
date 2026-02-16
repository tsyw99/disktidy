use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
use std::sync::Mutex;

use crate::models::settings::{AppSettings, CleanRule};
use crate::models::DiskTidyError;

pub struct SettingsManager {
    settings: Mutex<AppSettings>,
    config_path: PathBuf,
    rules: Mutex<Vec<CleanRule>>,
}

impl SettingsManager {
    pub fn new() -> Self {
        let config_path = Self::get_config_path();
        let settings = Self::load_settings(&config_path).unwrap_or_default();
        let rules = Self::load_rules(&config_path).unwrap_or_default();

        Self {
            settings: Mutex::new(settings),
            config_path,
            rules: Mutex::new(rules),
        }
    }

    pub fn with_path(config_path: PathBuf) -> Self {
        let settings = Self::load_settings(&config_path).unwrap_or_default();
        let rules = Self::load_rules(&config_path).unwrap_or_default();

        Self {
            settings: Mutex::new(settings),
            config_path,
            rules: Mutex::new(rules),
        }
    }

    fn get_config_path() -> PathBuf {
        if let Some(config_dir) = dirs::config_dir() {
            let app_config = config_dir.join("DiskTidy");
            if !app_config.exists() {
                let _ = fs::create_dir_all(&app_config);
            }
            app_config.join("settings.json")
        } else {
            PathBuf::from("settings.json")
        }
    }

    fn load_settings(path: &PathBuf) -> Result<AppSettings, DiskTidyError> {
        if !path.exists() {
            return Ok(AppSettings::default());
        }

        let mut file = fs::File::open(path)
            .map_err(|e| DiskTidyError::SettingsLoadFailed(e.to_string()))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| DiskTidyError::SettingsLoadFailed(e.to_string()))?;

        let config: ConfigFile = serde_json::from_str(&content)
            .map_err(|e| DiskTidyError::ConfigError { message: e.to_string() })?;

        Ok(config.settings)
    }

    fn load_rules(path: &PathBuf) -> Result<Vec<CleanRule>, DiskTidyError> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = fs::File::open(path)
            .map_err(|e| DiskTidyError::SettingsLoadFailed(e.to_string()))?;

        let mut content = String::new();
        file.read_to_string(&mut content)
            .map_err(|e| DiskTidyError::SettingsLoadFailed(e.to_string()))?;

        let config: ConfigFile = serde_json::from_str(&content)
            .map_err(|e| DiskTidyError::ConfigError { message: e.to_string() })?;

        Ok(config.rules)
    }

    pub fn get_settings(&self) -> AppSettings {
        let settings = self.settings.lock().unwrap();
        settings.clone()
    }

    pub fn update_settings(&self, new_settings: AppSettings) -> Result<AppSettings, DiskTidyError> {
        {
            let mut settings = self.settings.lock().unwrap();
            *settings = new_settings.clone();
        }
        self.save_to_file()?;
        Ok(new_settings)
    }

    pub fn update_settings_partial(&self, updates: SettingsUpdate) -> Result<AppSettings, DiskTidyError> {
        let mut settings = self.settings.lock().unwrap();

        if let Some(auto_scan) = updates.auto_scan {
            settings.auto_scan = auto_scan;
        }
        if let Some(scan_on_startup) = updates.scan_on_startup {
            settings.scan_on_startup = scan_on_startup;
        }
        if let Some(default_scan_mode) = updates.default_scan_mode {
            settings.default_scan_mode = default_scan_mode;
        }
        if let Some(default_clean_mode) = updates.default_clean_mode {
            settings.default_clean_mode = default_clean_mode;
        }
        if let Some(confirm_before_clean) = updates.confirm_before_clean {
            settings.confirm_before_clean = confirm_before_clean;
        }
        if let Some(show_notifications) = updates.show_notifications {
            settings.show_notifications = show_notifications;
        }
        if let Some(language) = updates.language {
            settings.language = language;
        }
        if let Some(theme) = updates.theme {
            settings.theme = theme;
        }

        let result = settings.clone();
        drop(settings);
        self.save_to_file()?;

        Ok(result)
    }

    pub fn reset_settings(&self) -> Result<AppSettings, DiskTidyError> {
        let default_settings = AppSettings::default();
        {
            let mut settings = self.settings.lock().unwrap();
            *settings = default_settings.clone();
        }
        self.save_to_file()?;
        Ok(default_settings)
    }

    pub fn get_rules(&self) -> Vec<CleanRule> {
        let rules = self.rules.lock().unwrap();
        rules.clone()
    }

    pub fn add_rule(&self, rule: CleanRule) -> Result<CleanRule, DiskTidyError> {
        {
            let mut rules = self.rules.lock().unwrap();
            rules.push(rule.clone());
        }
        self.save_to_file()?;
        Ok(rule)
    }

    pub fn remove_rule(&self, rule_id: &str) -> Result<bool, DiskTidyError> {
        let removed = {
            let mut rules = self.rules.lock().unwrap();
            let initial_len = rules.len();
            rules.retain(|r| r.id != rule_id);
            rules.len() < initial_len
        };

        if removed {
            self.save_to_file()?;
        }

        Ok(removed)
    }

    pub fn update_rule(&self, rule: CleanRule) -> Result<Option<CleanRule>, DiskTidyError> {
        let updated = {
            let mut rules = self.rules.lock().unwrap();
            if let Some(existing) = rules.iter_mut().find(|r| r.id == rule.id) {
                *existing = rule.clone();
                Some(rule)
            } else {
                None
            }
        };

        if updated.is_some() {
            self.save_to_file()?;
        }

        Ok(updated)
    }

    pub fn get_rule_by_id(&self, rule_id: &str) -> Option<CleanRule> {
        let rules = self.rules.lock().unwrap();
        rules.iter().find(|r| r.id == rule_id).cloned()
    }

    pub fn toggle_rule(&self, rule_id: &str) -> Result<Option<CleanRule>, DiskTidyError> {
        let updated = {
            let mut rules = self.rules.lock().unwrap();
            if let Some(rule) = rules.iter_mut().find(|r| r.id == rule_id) {
                rule.enabled = !rule.enabled;
                Some(rule.clone())
            } else {
                None
            }
        };

        if updated.is_some() {
            self.save_to_file()?;
        }

        Ok(updated)
    }

    fn save_to_file(&self) -> Result<(), DiskTidyError> {
        let settings = self.settings.lock().unwrap().clone();
        let rules = self.rules.lock().unwrap().clone();

        let config = ConfigFile {
            settings,
            rules,
        };

        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| DiskTidyError::ConfigError { message: e.to_string() })?;

        let mut file = fs::File::create(&self.config_path)
            .map_err(|e| DiskTidyError::SettingsSaveFailed(e.to_string()))?;

        file.write_all(content.as_bytes())
            .map_err(|e| DiskTidyError::SettingsSaveFailed(e.to_string()))?;

        Ok(())
    }

    pub fn export_settings(&self) -> Result<String, DiskTidyError> {
        let settings = self.settings.lock().unwrap().clone();
        let rules = self.rules.lock().unwrap().clone();

        let config = ConfigFile {
            settings,
            rules,
        };

        serde_json::to_string_pretty(&config)
            .map_err(|e| DiskTidyError::ConfigError { message: e.to_string() })
    }

    pub fn import_settings(&self, json: &str) -> Result<(), DiskTidyError> {
        let config: ConfigFile = serde_json::from_str(json)
            .map_err(|e| DiskTidyError::ConfigError { message: e.to_string() })?;

        {
            let mut settings = self.settings.lock().unwrap();
            *settings = config.settings;
        }

        {
            let mut rules = self.rules.lock().unwrap();
            *rules = config.rules;
        }

        self.save_to_file()
    }
}

impl Default for SettingsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SettingsUpdate {
    pub auto_scan: Option<bool>,
    pub scan_on_startup: Option<bool>,
    pub default_scan_mode: Option<crate::models::scan::ScanMode>,
    pub default_clean_mode: Option<crate::models::cleaner::CleanMode>,
    pub confirm_before_clean: Option<bool>,
    pub show_notifications: Option<bool>,
    pub language: Option<String>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ConfigFile {
    settings: AppSettings,
    rules: Vec<CleanRule>,
}
