use std::path::Path;

#[derive(Debug, Clone)]
pub struct FileCategoryRule {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub clean_safety: CleanSafety,
    pub path_patterns: Vec<String>,
    pub extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CleanSafety {
    Safe,
    LowRisk,
    MediumRisk,
    HighRisk,
    NotCleanable,
}

impl FileCategoryRule {
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let path_str_lower = path_str.as_str();

        for exclude in &self.exclude_patterns {
            if path_str_lower.contains(&exclude.to_lowercase()) {
                return false;
            }
        }

        for pattern in &self.path_patterns {
            if path_str_lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            if self.extensions.iter().any(|e| e.to_lowercase() == ext_lower) {
                return true;
            }
        }

        false
    }

    pub fn is_cleanable(&self) -> bool {
        matches!(
            self.clean_safety,
            CleanSafety::Safe | CleanSafety::LowRisk | CleanSafety::MediumRisk
        )
    }
}

pub struct FileCategoryRegistry {
    categories: Vec<FileCategoryRule>,
    protected_extensions: Vec<String>,
    protected_path_patterns: Vec<String>,
}

impl FileCategoryRegistry {
    pub fn new() -> Self {
        let categories = vec![
            FileCategoryRule {
                name: "system_temp".to_string(),
                display_name: "系统临时文件".to_string(),
                description: "该类文件通常为应用程序运行过程中产生的临时数据，一般可安全清理。包括系统临时目录和用户临时目录中的文件。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\temp\\".to_string(),
                    "\\tmp\\".to_string(),
                    "\\windows\\temp\\".to_string(),
                ],
                extensions: vec!["tmp".to_string(), "temp".to_string(), "bak".to_string()],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "browser_cache".to_string(),
                display_name: "浏览器缓存".to_string(),
                description: "浏览器在访问网页时下载的缓存文件，包括图片、脚本、样式表等。清理后可释放磁盘空间，但可能导致网页首次加载稍慢。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\chrome\\user data\\default\\cache".to_string(),
                    "\\edge\\user data\\default\\cache".to_string(),
                    "\\firefox\\profiles".to_string(),
                    "\\browser\\cache".to_string(),
                ],
                extensions: vec![],
                exclude_patterns: vec![
                    "\\cookies".to_string(),
                    "\\bookmarks".to_string(),
                    "\\history".to_string(),
                    "\\logins".to_string(),
                    "\\extensions".to_string(),
                    "\\preferences".to_string(),
                ],
            },
            FileCategoryRule {
                name: "app_cache".to_string(),
                display_name: "应用程序缓存".to_string(),
                description: "各类应用程序产生的缓存数据，如IDE编译缓存、工具软件缓存等。清理前建议确认应用程序已关闭。".to_string(),
                clean_safety: CleanSafety::LowRisk,
                path_patterns: vec![
                    "\\cache\\".to_string(),
                    "\\cached\\".to_string(),
                    "\\caches\\".to_string(),
                    "\\__pycache__\\".to_string(),
                    "\\.cache\\".to_string(),
                    "\\node_modules\\.cache\\".to_string(),
                ],
                extensions: vec![],
                exclude_patterns: vec![
                    "\\lib\\".to_string(),
                    "\\libs\\".to_string(),
                    "\\bin\\".to_string(),
                    "\\plugins\\".to_string(),
                    "\\.jar".to_string(),
                    "\\.dll".to_string(),
                    "\\.exe".to_string(),
                    "\\.so".to_string(),
                ],
            },
            FileCategoryRule {
                name: "log_files".to_string(),
                display_name: "日志文件".to_string(),
                description: "应用程序和系统运行产生的日志记录文件。清理后不影响程序运行，但可能丢失历史运行记录。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\logs\\".to_string(),
                    "\\log\\".to_string(),
                ],
                extensions: vec!["log".to_string(), "logs".to_string()],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "recycle_bin".to_string(),
                display_name: "回收站".to_string(),
                description: "已删除但尚未永久清除的文件。清理回收站将永久删除这些文件，请确认其中没有重要数据。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "$recycle.bin".to_string(),
                ],
                extensions: vec![],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "npm_cache".to_string(),
                display_name: "NPM缓存".to_string(),
                description: "Node.js包管理器下载的包缓存。清理后下次安装相同包时需要重新下载。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\npm-cache\\".to_string(),
                    "\\npm\\cache\\".to_string(),
                ],
                extensions: vec![],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "pip_cache".to_string(),
                display_name: "Pip缓存".to_string(),
                description: "Python包管理器下载的包缓存。清理后下次安装相同包时需要重新下载。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\pip\\cache\\".to_string(),
                ],
                extensions: vec![],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "thumbnail_cache".to_string(),
                display_name: "缩略图缓存".to_string(),
                description: "系统为图片和视频文件生成的缩略图缓存。清理后系统会重新生成缩略图。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\thumbcache_".to_string(),
                    "\\microsoft\\windows\\explorer\\".to_string(),
                ],
                extensions: vec!["db".to_string()],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "crash_dumps".to_string(),
                display_name: "崩溃转储文件".to_string(),
                description: "程序崩溃时生成的调试信息文件。清理后不影响系统运行，但可能影响问题排查。".to_string(),
                clean_safety: CleanSafety::Safe,
                path_patterns: vec![
                    "\\crashdumps\\".to_string(),
                    "\\dumps\\".to_string(),
                ],
                extensions: vec!["dmp".to_string(), "dump".to_string()],
                exclude_patterns: vec![],
            },
            FileCategoryRule {
                name: "windows_update".to_string(),
                display_name: "Windows更新缓存".to_string(),
                description: "Windows更新下载的临时文件。清理可能影响系统更新，建议在系统稳定后清理。".to_string(),
                clean_safety: CleanSafety::MediumRisk,
                path_patterns: vec![
                    "\\windows\\softwaredistribution\\download\\".to_string(),
                ],
                extensions: vec![],
                exclude_patterns: vec![],
            },
        ];

        let protected_extensions = vec![
            "jar".to_string(),
            "dll".to_string(),
            "exe".to_string(),
            "sys".to_string(),
            "so".to_string(),
            "dylib".to_string(),
            "lib".to_string(),
            "ocx".to_string(),
            "drv".to_string(),
            "efi".to_string(),
        ];

        let protected_path_patterns = vec![
            "\\windows\\system32\\".to_string(),
            "\\windows\\syswow64\\".to_string(),
            "\\windows\\winsxs\\".to_string(),
            "\\program files\\".to_string(),
            "\\program files (x86)\\".to_string(),
            "\\programdata\\".to_string(),
            "\\lib\\".to_string(),
            "\\libs\\".to_string(),
            "\\plugins\\".to_string(),
            "\\node_modules\\".to_string(),
            "\\.nuget\\".to_string(),
            "\\.gradle\\".to_string(),
            "\\.m2\\".to_string(),
            "\\venv\\".to_string(),
            "\\site-packages\\".to_string(),
        ];

        Self {
            categories,
            protected_extensions,
            protected_path_patterns,
        }
    }

    pub fn categorize_file(&self, path: &Path) -> Option<&FileCategoryRule> {
        let path_str = path.to_string_lossy().to_lowercase();

        if self.is_protected_path(&path_str) {
            return None;
        }

        for category in &self.categories {
            if category.matches(path) {
                return Some(category);
            }
        }

        None
    }

    pub fn is_protected_path(&self, path_lower: &str) -> bool {
        for ext in &self.protected_extensions {
            if path_lower.ends_with(&format!(".{}", ext)) {
                return true;
            }
        }

        for pattern in &self.protected_path_patterns {
            if path_lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }

        false
    }

    pub fn get_all_categories(&self) -> &[FileCategoryRule] {
        &self.categories
    }

    pub fn get_category(&self, name: &str) -> Option<&FileCategoryRule> {
        self.categories.iter().find(|c| c.name == name)
    }

    pub fn is_file_cleanable(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        
        if self.is_protected_path(&path_str) {
            return false;
        }

        if let Some(category) = self.categorize_file(path) {
            category.is_cleanable()
        } else {
            false
        }
    }
}

impl Default for FileCategoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_category_display_name(name: &str) -> String {
    let registry = FileCategoryRegistry::new();
    registry
        .get_category(name)
        .map(|c| c.display_name.clone())
        .unwrap_or_else(|| name.to_string())
}

pub fn get_category_description(name: &str) -> String {
    let registry = FileCategoryRegistry::new();
    registry
        .get_category(name)
        .map(|c| c.description.clone())
        .unwrap_or_else(|| "未知文件类型".to_string())
}

pub fn get_all_category_info() -> Vec<(String, String, String)> {
    let registry = FileCategoryRegistry::new();
    registry
        .categories
        .iter()
        .map(|c| (c.name.clone(), c.display_name.clone(), c.description.clone()))
        .collect()
}
