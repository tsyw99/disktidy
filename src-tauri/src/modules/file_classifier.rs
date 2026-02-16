use std::path::PathBuf;
use std::collections::HashMap;
use std::time::Instant;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::utils::file_type::get_file_type;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTypeStats {
    pub category: String,
    pub display_name: String,
    pub count: u64,
    pub total_size: u64,
    pub percentage: f64,
    pub extensions: HashMap<String, ExtensionStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionStats {
    pub extension: String,
    pub count: u64,
    pub total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassificationResult {
    pub scan_id: String,
    pub path: String,
    pub total_files: u64,
    pub total_size: u64,
    pub total_folders: u64,
    pub categories: Vec<FileTypeStats>,
    pub duration_ms: u64,
    pub largest_files: Vec<FileBriefInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileBriefInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub extension: String,
    pub category: String,
    pub modified_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileClassificationOptions {
    pub max_depth: Option<usize>,
    pub include_hidden: bool,
    pub include_system: bool,
    pub exclude_paths: Vec<String>,
    pub max_files: Option<usize>,
    pub top_n_categories: Option<usize>,
}

impl Default for FileClassificationOptions {
    fn default() -> Self {
        Self {
            max_depth: None,
            include_hidden: false,
            include_system: false,
            exclude_paths: vec![],
            max_files: None,
            top_n_categories: Some(15),
        }
    }
}

pub struct FileClassifier {
    options: FileClassificationOptions,
}

impl FileClassifier {
    pub fn new() -> Self {
        Self {
            options: FileClassificationOptions::default(),
        }
    }

    pub fn with_options(options: FileClassificationOptions) -> Self {
        Self { options }
    }

    pub fn classify(&self, path: &str) -> FileClassificationResult {
        let start = Instant::now();
        let scan_id = uuid::Uuid::new_v4().to_string();
        
        let root_path = PathBuf::from(path);
        let mut categories: HashMap<String, FileTypeStats> = HashMap::new();
        let mut total_files: u64 = 0;
        let mut total_size: u64 = 0;
        let mut total_folders: u64 = 0;
        let mut largest_files: Vec<FileBriefInfo> = Vec::new();
        let mut file_count = 0;

        let walker = WalkDir::new(&root_path)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                if !self.options.include_hidden {
                    if let Some(name) = e.file_name().to_str() {
                        if name.starts_with('.') {
                            return false;
                        }
                    }
                }
                
                let path_str = e.path().to_string_lossy().to_lowercase();
                for exclude in &self.options.exclude_paths {
                    if path_str.contains(&exclude.to_lowercase()) {
                        return false;
                    }
                }
                
                true
            });

        for entry in walker {
            if let Ok(entry) = entry {
                if self.options.max_depth.is_some() {
                    let depth = entry.depth();
                    if depth > self.options.max_depth.unwrap() {
                        continue;
                    }
                }

                let path = entry.path();
                
                if entry.file_type().is_dir() {
                    total_folders += 1;
                    continue;
                }

                if entry.file_type().is_file() {
                    if let Ok(metadata) = entry.metadata() {
                        let size = metadata.len();
                        let modified_time = metadata.modified()
                            .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                                .map(|d| d.as_secs())
                                .unwrap_or(0))
                            .unwrap_or(0);

                        let extension = path.extension()
                            .and_then(|e| e.to_str())
                            .unwrap_or("")
                            .to_lowercase();

                        let category = get_file_type(&extension);
                        let file_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        total_files += 1;
                        total_size += size;

                        let stats = categories.entry(category.clone()).or_insert_with(|| FileTypeStats {
                            category: category.clone(),
                            display_name: category.clone(),
                            count: 0,
                            total_size: 0,
                            percentage: 0.0,
                            extensions: HashMap::new(),
                        });

                        stats.count += 1;
                        stats.total_size += size;

                        let ext_stats = stats.extensions.entry(extension.clone()).or_insert_with(|| ExtensionStats {
                            extension: extension.clone(),
                            count: 0,
                            total_size: 0,
                        });
                        ext_stats.count += 1;
                        ext_stats.total_size += size;

                        largest_files.push(FileBriefInfo {
                            path: path.to_string_lossy().to_string(),
                            name: file_name,
                            size,
                            extension,
                            category,
                            modified_time,
                        });

                        file_count += 1;
                        if let Some(max) = self.options.max_files {
                            if file_count >= max {
                                break;
                            }
                        }
                    }
                }
            }
        }

        for stats in categories.values_mut() {
            if total_size > 0 {
                stats.percentage = (stats.total_size as f64 / total_size as f64) * 100.0;
            }
        }

        largest_files.sort_by(|a, b| b.size.cmp(&a.size));
        largest_files.truncate(10);

        let mut categories_vec: Vec<FileTypeStats> = categories.into_values().collect();
        categories_vec.sort_by(|a, b| b.total_size.cmp(&a.total_size));
        
        if let Some(top_n) = self.options.top_n_categories {
            categories_vec.truncate(top_n);
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        FileClassificationResult {
            scan_id,
            path: path.to_string(),
            total_files,
            total_size,
            total_folders,
            categories: categories_vec,
            duration_ms,
            largest_files,
        }
    }
}

impl Default for FileClassifier {
    fn default() -> Self {
        Self::new()
    }
}
