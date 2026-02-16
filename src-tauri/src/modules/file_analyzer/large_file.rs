use crate::models::cleaner::{
    LargeFile, LargeFileAnalysisResult, LargeFileAnalyzerOptions, LargeFileDetails,
};
use crate::utils::path::{PathUtils, SystemPaths};
use crate::utils::file_type::get_file_type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

pub struct LargeFileAnalysisProgress {
    pub current_path: String,
    pub scanned_files: u64,
    pub found_files: u64,
    pub total_size: u64,
}

pub type LargeFileProgressCallback = Box<dyn Fn(LargeFileAnalysisProgress) + Send + Sync>;

#[derive(Clone)]
pub struct TypeStats {
    pub file_type: String,
    pub count: u64,
    pub total_size: u64,
    pub percentage: f32,
}

pub struct LargeFileStats {
    pub total_count: u64,
    pub total_size: u64,
    pub average_size: u64,
    pub largest_file: Option<LargeFile>,
    pub type_distribution: HashMap<String, TypeStats>,
}

pub struct LargeFileAnalyzer {
    options: LargeFileAnalyzerOptions,
    protected_paths: Vec<PathBuf>,
}

impl LargeFileAnalyzer {
    pub fn new() -> Self {
        Self::with_options(LargeFileAnalyzerOptions::default())
    }

    pub fn with_options(options: LargeFileAnalyzerOptions) -> Self {
        Self {
            options,
            protected_paths: SystemPaths::get_protected_paths(),
        }
    }

    pub fn set_threshold(&mut self, size: u64) {
        self.options.threshold = size;
    }

    pub fn add_exclude_path(&mut self, path: PathBuf) {
        self.options.exclude_paths.push(path.to_string_lossy().to_string());
    }

    pub fn analyze(&self, paths: &[PathBuf]) -> LargeFileAnalysisResult {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let large_files = self.analyze_paths(paths, None);

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let total_size: u64 = large_files.iter().map(|f| f.size).sum();

        LargeFileAnalysisResult {
            scan_id: uuid::Uuid::new_v4().to_string(),
            total_files: large_files.len() as u64,
            total_size,
            files: large_files,
            threshold: self.options.threshold,
            duration_ms: end_time - start_time,
        }
    }

    pub fn analyze_with_progress(
        &self,
        paths: &[PathBuf],
        progress_callback: Option<&LargeFileProgressCallback>,
    ) -> LargeFileAnalysisResult {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let large_files = self.analyze_paths(paths, progress_callback);

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let total_size: u64 = large_files.iter().map(|f| f.size).sum();

        LargeFileAnalysisResult {
            scan_id: uuid::Uuid::new_v4().to_string(),
            total_files: large_files.len() as u64,
            total_size,
            files: large_files,
            threshold: self.options.threshold,
            duration_ms: end_time - start_time,
        }
    }

    fn analyze_paths(
        &self,
        paths: &[PathBuf],
        progress_callback: Option<&LargeFileProgressCallback>,
    ) -> Vec<LargeFile> {
        let mut large_files: Vec<LargeFile> = Vec::new();
        let mut scanned_count = 0u64;

        for scan_path in paths {
            if !scan_path.exists() {
                continue;
            }

            let walker = WalkDir::new(scan_path)
                .follow_links(false)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                let path = entry.path();

                if self.should_skip(path) {
                    continue;
                }

                scanned_count += 1;

                if let Some(ref callback) = progress_callback {
                    callback(LargeFileAnalysisProgress {
                        current_path: path.to_string_lossy().to_string(),
                        scanned_files: scanned_count,
                        found_files: large_files.len() as u64,
                        total_size: large_files.iter().map(|f| f.size).sum(),
                    });
                }

                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.len() >= self.options.threshold {
                        if let Some(large_file) = self.create_large_file(path, &metadata) {
                            large_files.push(large_file);
                        }
                    }
                }
            }
        }

        large_files.sort_by(|a, b| b.size.cmp(&a.size));
        large_files
    }

    fn should_skip(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        for exclude in &self.options.exclude_paths {
            if path_str.to_lowercase().starts_with(&exclude.to_lowercase()) {
                return true;
            }
        }

        for protected in &self.protected_paths {
            if path.starts_with(protected) {
                return true;
            }
        }

        if !self.options.include_hidden {
            if let Ok(metadata) = fs::metadata(path) {
                let attrs = metadata.file_attributes();
                if attrs & 0x2 != 0 {
                    return true;
                }
            }
        }

        if !self.options.include_system {
            if let Ok(metadata) = fs::metadata(path) {
                let attrs = metadata.file_attributes();
                if attrs & 0x4 != 0 {
                    return true;
                }
            }
        }

        false
    }

    fn create_large_file(&self, path: &Path, metadata: &fs::Metadata) -> Option<LargeFile> {
        let name = PathUtils::get_filename(path)?;
        let extension = PathUtils::get_extension(path).unwrap_or_default();
        let file_type = get_file_type(&extension);

        Some(LargeFile {
            path: path.to_string_lossy().to_string(),
            name,
            size: metadata.len(),
            modified_time: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            accessed_time: metadata
                .accessed()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            created_time: metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0),
            file_type,
            extension,
        })
    }

    pub fn get_file_details(&self, path: &Path) -> Option<LargeFileDetails> {
        let metadata = fs::metadata(path).ok()?;
        let large_file = self.create_large_file(path, &metadata)?;

        let attrs = metadata.file_attributes();
        let is_readonly = attrs & 0x1 != 0;
        let is_hidden = attrs & 0x2 != 0;
        let is_system = attrs & 0x4 != 0;

        let owner = self.get_file_owner(path);

        Some(LargeFileDetails {
            file: large_file,
            owner,
            is_readonly,
            is_hidden,
            is_system,
            attributes: attrs,
        })
    }

    #[cfg(windows)]
    fn get_file_owner(&self, _path: &Path) -> Option<String> {
        Some("Current User".to_string())
    }

    #[cfg(not(windows))]
    fn get_file_owner(&self, _path: &Path) -> Option<String> {
        None
    }

    pub fn group_by_type<'a>(&self, files: &'a [LargeFile]) -> HashMap<String, Vec<&'a LargeFile>> {
        let mut groups: HashMap<String, Vec<&'a LargeFile>> = HashMap::new();

        for file in files {
            groups
                .entry(file.file_type.clone())
                .or_default()
                .push(file);
        }

        groups
    }

    pub fn group_by_extension<'a>(&self, files: &'a [LargeFile]) -> HashMap<String, Vec<&'a LargeFile>> {
        let mut groups: HashMap<String, Vec<&'a LargeFile>> = HashMap::new();

        for file in files {
            let ext = if file.extension.is_empty() {
                "无扩展名".to_string()
            } else {
                file.extension.clone()
            };
            groups.entry(ext).or_default().push(file);
        }

        groups
    }

    pub fn group_by_directory<'a>(&self, files: &'a [LargeFile]) -> HashMap<String, Vec<&'a LargeFile>> {
        let mut groups: HashMap<String, Vec<&'a LargeFile>> = HashMap::new();

        for file in files {
            let path = PathBuf::from(&file.path);
            if let Some(parent) = path.parent() {
                let dir = parent.to_string_lossy().to_string();
                groups.entry(dir).or_default().push(file);
            }
        }

        groups
    }

    pub fn calculate_stats(&self, files: &[LargeFile]) -> LargeFileStats {
        let total_count = files.len() as u64;
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        let average_size = if total_count > 0 {
            total_size / total_count
        } else {
            0
        };
        let largest_file = files.first().cloned();

        let groups = self.group_by_type(files);
        let mut type_distribution = HashMap::new();

        for (file_type, type_files) in groups {
            let count = type_files.len() as u64;
            let type_size: u64 = type_files.iter().map(|f| f.size).sum();
            let percentage = if total_size > 0 {
                (type_size as f32 / total_size as f32) * 100.0
            } else {
                0.0
            };

            type_distribution.insert(
                file_type.clone(),
                TypeStats {
                    file_type,
                    count,
                    total_size: type_size,
                    percentage,
                },
            );
        }

        LargeFileStats {
            total_count,
            total_size,
            average_size,
            largest_file,
            type_distribution,
        }
    }
}

impl Default for LargeFileAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
