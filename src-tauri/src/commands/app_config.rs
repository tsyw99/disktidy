use std::collections::HashMap;
use tauri::command;

use crate::utils::{get_app_paths_config, save_app_paths_config};

#[command]
pub fn get_app_config() -> HashMap<String, String> {
    let config = get_app_paths_config();
    config.paths
}

#[command]
pub fn set_app_path(app: String, path: String) -> Result<(), String> {
    let mut config = get_app_paths_config();
    config.set_path(&app, &path);
    save_app_paths_config(&config)
}

#[command]
pub fn remove_app_path(app: String) -> Result<(), String> {
    let mut config = get_app_paths_config();
    config.remove_path(&app);
    save_app_paths_config(&config)
}

#[command]
pub fn is_app_configured(app: String) -> bool {
    let config = get_app_paths_config();
    config.has_path(&app)
}

#[command]
pub fn get_all_configured_apps() -> Vec<String> {
    let config = get_app_paths_config();
    config.get_configured_apps()
}

#[command]
pub fn validate_path(path: String) -> Result<bool, String> {
    let path = std::path::Path::new(&path);
    if !path.exists() {
        return Err("路径不存在".to_string());
    }
    if !path.is_dir() {
        return Err("路径不是目录".to_string());
    }
    Ok(true)
}
