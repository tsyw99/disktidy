use tempfile::TempDir;
use disktidy_lib::modules::settings::{SettingsManager, RuleEngine};
use disktidy_lib::models::settings::CleanRule;
use disktidy_lib::models::file_analyzer::JunkCategory;

#[test]
fn test_settings_manager_creation() {
    let manager = SettingsManager::new();
    let settings = manager.get_settings();
    
    assert!(!settings.auto_scan);
    assert!(settings.confirm_before_clean);
}

#[test]
fn test_settings_manager_with_path() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.json");
    
    let manager = SettingsManager::with_path(config_path);
    let settings = manager.get_settings();
    
    assert_eq!(settings.language, "zh-CN");
}

#[test]
fn test_update_settings() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.json");
    let manager = SettingsManager::with_path(config_path);
    
    let mut settings = manager.get_settings();
    settings.auto_scan = true;
    settings.language = "en-US".to_string();
    
    let result = manager.update_settings(settings).unwrap();
    
    assert!(result.auto_scan);
    assert_eq!(result.language, "en-US");
}

#[test]
fn test_reset_settings() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.json");
    let manager = SettingsManager::with_path(config_path);
    
    let mut settings = manager.get_settings();
    settings.auto_scan = true;
    manager.update_settings(settings).unwrap();
    
    let reset = manager.reset_settings().unwrap();
    assert!(!reset.auto_scan);
}

#[test]
fn test_add_remove_rule() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.json");
    let manager = SettingsManager::with_path(config_path);
    
    let rule = CleanRule {
        id: "test-rule".to_string(),
        name: "Test Rule".to_string(),
        description: "A test rule".to_string(),
        pattern: "*.test".to_string(),
        category: JunkCategory::TempFiles,
        enabled: true,
    };
    
    manager.add_rule(rule.clone()).unwrap();
    let rules = manager.get_rules();
    assert_eq!(rules.len(), 1);
    
    manager.remove_rule("test-rule").unwrap();
    let rules = manager.get_rules();
    assert!(rules.is_empty());
}

#[test]
fn test_rule_engine_creation() {
    let engine = RuleEngine::new();
    let rules = engine.get_rules();
    
    assert!(!rules.is_empty());
}

#[test]
fn test_rule_engine_matches() {
    use std::path::Path;
    
    let engine = RuleEngine::new();
    
    let matches = engine.matches(Path::new("test.tmp"));
    assert!(!matches.is_empty());
    
    let matches = engine.matches(Path::new("document.pdf"));
    assert!(matches.is_empty());
}

#[test]
fn test_rule_engine_toggle() {
    let mut engine = RuleEngine::new();
    
    let new_state = engine.toggle_rule("temp-files");
    assert_eq!(new_state, Some(false));
    
    let new_state = engine.toggle_rule("temp-files");
    assert_eq!(new_state, Some(true));
}

#[test]
fn test_rule_engine_add_custom() {
    let mut engine = RuleEngine::new();
    let initial_count = engine.get_rules().len();
    
    let rule = CleanRule {
        id: "custom-rule".to_string(),
        name: "Custom Rule".to_string(),
        description: "A custom rule".to_string(),
        pattern: "*.custom".to_string(),
        category: JunkCategory::TempFiles,
        enabled: true,
    };
    
    engine.add_rule(rule);
    assert_eq!(engine.get_rules().len(), initial_count + 1);
}

#[test]
fn test_settings_export_import() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("settings.json");
    let manager = SettingsManager::with_path(config_path);
    
    let mut settings = manager.get_settings();
    settings.auto_scan = true;
    manager.update_settings(settings).unwrap();
    
    let exported = manager.export_settings().unwrap();
    assert!(exported.contains("auto_scan"));
    
    let manager2 = SettingsManager::with_path(temp_dir.path().join("settings2.json"));
    manager2.import_settings(&exported).unwrap();
    let imported = manager2.get_settings();
    assert!(imported.auto_scan);
}
