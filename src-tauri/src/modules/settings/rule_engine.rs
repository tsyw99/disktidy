use std::path::Path;
use crate::models::settings::CleanRule;
use crate::models::file_analyzer::JunkCategory;

pub struct RuleEngine {
    rules: Vec<CleanRule>,
}

impl RuleEngine {
    pub fn new() -> Self {
        Self {
            rules: Self::get_default_rules(),
        }
    }

    pub fn with_rules(rules: Vec<CleanRule>) -> Self {
        Self { rules }
    }

    pub fn get_rules(&self) -> &[CleanRule] {
        &self.rules
    }

    pub fn get_enabled_rules(&self) -> Vec<&CleanRule> {
        self.rules.iter().filter(|r| r.enabled).collect()
    }

    pub fn matches(&self, path: &Path) -> Vec<&CleanRule> {
        let path_str = path.to_string_lossy();
        let filename = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        self.rules.iter()
            .filter(|r| r.enabled && self.matches_pattern(filename, &path_str, &r.pattern))
            .collect()
    }

    pub fn matches_category(&self, path: &Path, category: &JunkCategory) -> Vec<&CleanRule> {
        self.matches(path)
            .into_iter()
            .filter(|r| &r.category == category)
            .collect()
    }

    fn matches_pattern(&self, filename: &str, path: &str, pattern: &str) -> bool {
        if pattern.starts_with('*') && pattern.ends_with('*') {
            let middle = &pattern[1..pattern.len()-1];
            return filename.contains(middle) || path.contains(middle);
        }

        if pattern.starts_with('*') {
            let suffix = &pattern[1..];
            return filename.ends_with(suffix);
        }

        if pattern.ends_with('*') {
            let prefix = &pattern[..pattern.len()-1];
            return filename.starts_with(prefix);
        }

        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return filename.starts_with(parts[0]) && filename.ends_with(parts[1]);
            }
        }

        filename == pattern || path.contains(pattern)
    }

    pub fn should_clean(&self, path: &Path) -> bool {
        !self.matches(path).is_empty()
    }

    pub fn get_category_for_path(&self, path: &Path) -> Option<JunkCategory> {
        self.matches(path).first().map(|r| r.category.clone())
    }

    fn get_default_rules() -> Vec<CleanRule> {
        vec![
            CleanRule {
                id: "temp-files".to_string(),
                name: "临时文件".to_string(),
                description: "系统临时文件".to_string(),
                pattern: "*.tmp".to_string(),
                category: JunkCategory::TempFiles,
                enabled: true,
            },
            CleanRule {
                id: "log-files".to_string(),
                name: "日志文件".to_string(),
                description: "应用程序日志文件".to_string(),
                pattern: "*.log".to_string(),
                category: JunkCategory::Logs,
                enabled: true,
            },
            CleanRule {
                id: "cache-files".to_string(),
                name: "缓存文件".to_string(),
                description: "应用程序缓存文件".to_string(),
                pattern: "*cache*".to_string(),
                category: JunkCategory::Cache,
                enabled: true,
            },
            CleanRule {
                id: "bak-files".to_string(),
                name: "备份文件".to_string(),
                description: "备份文件".to_string(),
                pattern: "*.bak".to_string(),
                category: JunkCategory::TempFiles,
                enabled: true,
            },
            CleanRule {
                id: "old-files".to_string(),
                name: "旧文件".to_string(),
                description: "旧版本文件".to_string(),
                pattern: "*.old".to_string(),
                category: JunkCategory::TempFiles,
                enabled: true,
            },
        ]
    }

    pub fn add_rule(&mut self, rule: CleanRule) {
        if !self.rules.iter().any(|r| r.id == rule.id) {
            self.rules.push(rule);
        }
    }

    pub fn remove_rule(&mut self, rule_id: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|r| r.id != rule_id);
        self.rules.len() < initial_len
    }

    pub fn update_rule(&mut self, rule: CleanRule) -> bool {
        if let Some(existing) = self.rules.iter_mut().find(|r| r.id == rule.id) {
            *existing = rule;
            true
        } else {
            false
        }
    }

    pub fn toggle_rule(&mut self, rule_id: &str) -> Option<bool> {
        if let Some(rule) = self.rules.iter_mut().find(|r| r.id == rule_id) {
            rule.enabled = !rule.enabled;
            Some(rule.enabled)
        } else {
            None
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
