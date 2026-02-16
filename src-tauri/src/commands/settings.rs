use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};

use crate::models::settings::{AppSettings, CleanRule};
use crate::models::file_analyzer::JunkCategory;
use crate::modules::settings::{SettingsManager, SettingsUpdate, RuleEngine};

pub struct SettingsState {
    manager: Arc<Mutex<SettingsManager>>,
    rule_engine: Arc<Mutex<RuleEngine>>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            manager: Arc::new(Mutex::new(SettingsManager::new())),
            rule_engine: Arc::new(Mutex::new(RuleEngine::new())),
        }
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn settings_get(
    state: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let manager = state.manager.lock().await;
    Ok(manager.get_settings())
}

#[tauri::command]
pub async fn settings_update(
    settings: AppSettings,
    state: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let manager = state.manager.lock().await;
    manager.update_settings(settings)
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn settings_update_partial(
    updates: SettingsUpdateJson,
    state: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let manager = state.manager.lock().await;
    let update = SettingsUpdate {
        auto_scan: updates.auto_scan,
        scan_on_startup: updates.scan_on_startup,
        default_scan_mode: updates.default_scan_mode,
        default_clean_mode: updates.default_clean_mode,
        confirm_before_clean: updates.confirm_before_clean,
        show_notifications: updates.show_notifications,
        language: updates.language,
        theme: updates.theme,
    };
    manager.update_settings_partial(update)
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsUpdateJson {
    pub auto_scan: Option<bool>,
    pub scan_on_startup: Option<bool>,
    pub default_scan_mode: Option<crate::models::scan::ScanMode>,
    pub default_clean_mode: Option<crate::models::cleaner::CleanMode>,
    pub confirm_before_clean: Option<bool>,
    pub show_notifications: Option<bool>,
    pub language: Option<String>,
    pub theme: Option<String>,
}

#[tauri::command]
pub async fn settings_reset(
    state: State<'_, SettingsState>,
) -> Result<AppSettings, String> {
    let manager = state.manager.lock().await;
    manager.reset_settings()
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn settings_export(
    state: State<'_, SettingsState>,
) -> Result<String, String> {
    let manager = state.manager.lock().await;
    manager.export_settings()
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn settings_import(
    json: String,
    state: State<'_, SettingsState>,
) -> Result<(), String> {
    let manager = state.manager.lock().await;
    manager.import_settings(&json)
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn rule_list(
    state: State<'_, SettingsState>,
) -> Result<Vec<CleanRule>, String> {
    let manager = state.manager.lock().await;
    Ok(manager.get_rules())
}

#[tauri::command]
pub async fn rule_get(
    rule_id: String,
    state: State<'_, SettingsState>,
) -> Result<CleanRule, String> {
    let manager = state.manager.lock().await;
    manager.get_rule_by_id(&rule_id)
        .ok_or_else(|| "Rule not found".to_string())
}

#[tauri::command]
pub async fn rule_add(
    rule: CleanRuleJson,
    state: State<'_, SettingsState>,
) -> Result<CleanRule, String> {
    let manager = state.manager.lock().await;
    let clean_rule = CleanRule {
        id: uuid::Uuid::new_v4().to_string(),
        name: rule.name,
        description: rule.description,
        pattern: rule.pattern,
        category: rule.category,
        enabled: true,
    };
    manager.add_rule(clean_rule)
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanRuleJson {
    pub name: String,
    pub description: String,
    pub pattern: String,
    pub category: JunkCategory,
}

#[tauri::command]
pub async fn rule_update(
    rule: CleanRule,
    state: State<'_, SettingsState>,
) -> Result<CleanRule, String> {
    let manager = state.manager.lock().await;
    manager.update_rule(rule.clone())
        .map_err(|e| format!("{}: {}", e.error_code(), e))?
        .ok_or_else(|| "Rule not found".to_string())
}

#[tauri::command]
pub async fn rule_remove(
    rule_id: String,
    state: State<'_, SettingsState>,
) -> Result<bool, String> {
    let manager = state.manager.lock().await;
    manager.remove_rule(&rule_id)
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn rule_toggle(
    rule_id: String,
    state: State<'_, SettingsState>,
) -> Result<CleanRule, String> {
    let manager = state.manager.lock().await;
    manager.toggle_rule(&rule_id)
        .map_err(|e| format!("{}: {}", e.error_code(), e))?
        .ok_or_else(|| "Rule not found".to_string())
}

#[tauri::command]
pub async fn rule_test(
    path: String,
    state: State<'_, SettingsState>,
) -> Result<Vec<CleanRule>, String> {
    let engine = state.rule_engine.lock().await;
    Ok(engine.matches(std::path::Path::new(&path))
        .into_iter()
        .cloned()
        .collect())
}

#[tauri::command]
pub async fn rule_get_defaults() -> Result<Vec<CleanRule>, String> {
    Ok(RuleEngine::new().get_rules().to_vec())
}
