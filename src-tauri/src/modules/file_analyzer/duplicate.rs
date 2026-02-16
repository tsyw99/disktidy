use crate::models::cleaner::{
    DuplicateAnalysisResult, DuplicateDetectorOptions, DuplicateFile, DuplicateGroup,
};
use crate::utils::hash::{HashCalculator, HashResult};
use crate::utils::path::{PathUtils, SystemPaths};
use crate::utils::file_type::get_file_type;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[derive(Debug, Clone, PartialEq)]
pub enum DuplicatePhase {
    Scanning,
    GroupingBySize,
    CalculatingHash,
    GroupingByHash,
    Finalizing,
}

pub struct DuplicateDetectionProgress {
    pub current_phase: DuplicatePhase,
    pub current_path: String,
    pub processed_files: u64,
    pub total_files: u64,
    pub found_groups: u64,
    pub percent: f32,
}

pub type DuplicateProgressCallback = Box<dyn Fn(DuplicateDetectionProgress) + Send + Sync>;

pub struct DuplicateDetector {
    options: DuplicateDetectorOptions,
    hash_calculator: HashCalculator,
    protected_paths: Vec<PathBuf>,
}

impl DuplicateDetector {
    pub fn new() -> Self {
        Self::with_options(DuplicateDetectorOptions::default())
    }

    pub fn with_options(options: DuplicateDetectorOptions) -> Self {
        Self {
            hash_calculator: HashCalculator::with_cache(options.use_cache),
            protected_paths: SystemPaths::get_protected_paths(),
            options,
        }
    }

    pub fn clear_cache(&mut self) {
        self.hash_calculator.clear_cache();
    }

    pub fn cache_size(&self) -> usize {
        self.hash_calculator.cache_size()
    }

    pub fn find_duplicates(&self, paths: &[PathBuf]) -> DuplicateAnalysisResult {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let duplicate_groups = self.find_duplicates_internal(paths, None);

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let total_groups = duplicate_groups.len() as u64;
        let total_files: u64 = duplicate_groups.iter().map(|g| g.files.len() as u64).sum();
        let total_size: u64 = duplicate_groups.iter().map(|g| g.size * g.files.len() as u64).sum();
        let wasted_space: u64 = duplicate_groups.iter().map(|g| g.wasted_space).sum();

        DuplicateAnalysisResult {
            scan_id: uuid::Uuid::new_v4().to_string(),
            total_groups,
            total_files,
            total_size,
            wasted_space,
            groups: duplicate_groups,
            duration_ms: end_time - start_time,
        }
    }

    pub fn find_duplicates_with_progress(
        &self,
        paths: &[PathBuf],
        progress_callback: Option<&DuplicateProgressCallback>,
    ) -> DuplicateAnalysisResult {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let duplicate_groups = self.find_duplicates_internal(paths, progress_callback);

        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let total_groups = duplicate_groups.len() as u64;
        let total_files: u64 = duplicate_groups.iter().map(|g| g.files.len() as u64).sum();
        let total_size: u64 = duplicate_groups.iter().map(|g| g.size * g.files.len() as u64).sum();
        let wasted_space: u64 = duplicate_groups.iter().map(|g| g.wasted_space).sum();

        DuplicateAnalysisResult {
            scan_id: uuid::Uuid::new_v4().to_string(),
            total_groups,
            total_files,
            total_size,
            wasted_space,
            groups: duplicate_groups,
            duration_ms: end_time - start_time,
        }
    }

    fn find_duplicates_internal(
        &self,
        paths: &[PathBuf],
        progress_callback: Option<&DuplicateProgressCallback>,
    ) -> Vec<DuplicateGroup> {
        self.report_progress(
            &progress_callback,
            DuplicatePhase::Scanning,
            "",
            0,
            0,
            0,
            0.0,
        );

        let files = self.scan_files(paths);
        let total_files = files.len() as u64;

        self.report_progress(
            &progress_callback,
            DuplicatePhase::GroupingBySize,
            "",
            0,
            total_files,
            0,
            10.0,
        );

        let size_groups = self.group_by_size(&files);

        self.report_progress(
            &progress_callback,
            DuplicatePhase::CalculatingHash,
            "",
            0,
            total_files,
            0,
            20.0,
        );

        let hash_groups = self.calculate_and_group_by_hash(&size_groups, &progress_callback, total_files);

        self.report_progress(
            &progress_callback,
            DuplicatePhase::Finalizing,
            "",
            total_files,
            total_files,
            hash_groups.len() as u64,
            100.0,
        );

        self.create_duplicate_groups(hash_groups)
    }

    fn report_progress(
        &self,
        callback: &Option<&DuplicateProgressCallback>,
        phase: DuplicatePhase,
        current_path: &str,
        processed_files: u64,
        total_files: u64,
        found_groups: u64,
        percent: f32,
    ) {
        if let Some(cb) = callback {
            cb(DuplicateDetectionProgress {
                current_phase: phase,
                current_path: current_path.to_string(),
                processed_files,
                total_files,
                found_groups,
                percent,
            });
        }
    }

    fn scan_files(&self, paths: &[PathBuf]) -> Vec<PathBuf> {
        let mut files: Vec<PathBuf> = Vec::new();

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
                let path = entry.path().to_path_buf();

                if self.should_skip(&path) {
                    continue;
                }

                files.push(path);
            }
        }

        files
    }

    fn should_skip(&self, path: &Path) -> bool {
        for protected in &self.protected_paths {
            if path.starts_with(protected) {
                return true;
            }
        }

        if let Ok(metadata) = fs::metadata(path) {
            if metadata.len() < self.options.min_size {
                return true;
            }

            if let Some(max_size) = self.options.max_size {
                if metadata.len() > max_size {
                    return true;
                }
            }

            #[cfg(windows)]
            {
                if !self.options.include_hidden {
                    let attrs = metadata.file_attributes();
                    if attrs & 0x2 != 0 {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn group_by_size(&self, files: &[PathBuf]) -> HashMap<u64, Vec<PathBuf>> {
        let mut groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

        for path in files {
            if let Ok(metadata) = fs::metadata(path) {
                let size = metadata.len();
                if size >= self.options.min_size {
                    groups.entry(size).or_default().push(path.clone());
                }
            }
        }

        groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect()
    }

    fn calculate_and_group_by_hash(
        &self,
        size_groups: &HashMap<u64, Vec<PathBuf>>,
        progress_callback: &Option<&DuplicateProgressCallback>,
        total_files: u64,
    ) -> HashMap<String, Vec<(PathBuf, u64)>> {
        let mut hash_groups: HashMap<String, Vec<(PathBuf, u64)>> = HashMap::new();
        let mut processed = 0u64;

        for (size, files) in size_groups {
            let partial_hashes = self.calculate_partial_hashes(files);

            let mut partial_groups: HashMap<String, Vec<&PathBuf>> = HashMap::new();
            for (path, hash) in &partial_hashes {
                partial_groups.entry(hash.clone()).or_default().push(path);
            }

            for (_, same_partial_paths) in partial_groups {
                if same_partial_paths.len() > 1 {
                    for path in same_partial_paths {
                        if let Ok(hash_result) = self.hash_calculator.calculate_file_hash_sync(path) {
                            hash_groups
                                .entry(hash_result.hash)
                                .or_default()
                                .push((path.clone(), *size));

                            processed += 1;
                            let percent = 20.0 + (processed as f32 / total_files.max(1) as f32) * 70.0;
                            self.report_progress(
                                progress_callback,
                                DuplicatePhase::CalculatingHash,
                                &path.to_string_lossy(),
                                processed,
                                total_files,
                                hash_groups.len() as u64,
                                percent,
                            );
                        }
                    }
                }
            }
        }

        hash_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .collect()
    }

    fn calculate_partial_hashes(&self, files: &[PathBuf]) -> Vec<(PathBuf, String)> {
        let mut results: Vec<(PathBuf, String)> = Vec::new();

        for path in files {
            if let Ok(hash_result) = self.hash_calculator.calculate_partial_hash_sync(path) {
                results.push((path.clone(), hash_result.hash));
            }
        }

        results
    }

    fn create_duplicate_groups(
        &self,
        hash_groups: HashMap<String, Vec<(PathBuf, u64)>>,
    ) -> Vec<DuplicateGroup> {
        let mut duplicate_groups: Vec<DuplicateGroup> = Vec::new();

        for (hash, files) in hash_groups {
            if files.len() < 2 {
                continue;
            }

            let size = files[0].1;
            let wasted_space = size * (files.len() as u64 - 1);

            let mut duplicate_files: Vec<DuplicateFile> = files
                .into_iter()
                .enumerate()
                .map(|(index, (path, _))| {
                    let name = PathUtils::get_filename(&path).unwrap_or_default();
                    let modified_time = fs::metadata(&path)
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs() as i64)
                        .unwrap_or(0);

                    DuplicateFile {
                        path: path.to_string_lossy().to_string(),
                        name,
                        modified_time,
                        is_original: index == 0,
                    }
                })
                .collect();

            duplicate_files.sort_by(|a, b| a.modified_time.cmp(&b.modified_time));
            if !duplicate_files.is_empty() {
                duplicate_files[0].is_original = true;
            }

            let file_type = duplicate_files
                .first()
                .map(|f| {
                    let ext = PathUtils::get_extension(Path::new(&f.path)).unwrap_or_default();
                    get_file_type(&ext)
                })
                .unwrap_or_default();

            duplicate_groups.push(DuplicateGroup {
                hash,
                size,
                files: duplicate_files,
                wasted_space,
                file_type,
            });
        }

        duplicate_groups.sort_by(|a, b| b.wasted_space.cmp(&a.wasted_space));

        duplicate_groups
    }

    pub fn quick_compare(&self, path1: &Path, path2: &Path) -> bool {
        let meta1 = match fs::metadata(path1) {
            Ok(m) => m,
            Err(_) => return false,
        };
        let meta2 = match fs::metadata(path2) {
            Ok(m) => m,
            Err(_) => return false,
        };

        if meta1.len() != meta2.len() {
            return false;
        }

        self.hash_calculator.quick_compare_sync(path1, path2)
    }

    pub fn batch_quick_compare(&self, files: &[PathBuf]) -> Vec<Vec<PathBuf>> {
        if files.len() < 2 {
            return vec![];
        }

        let mut groups: Vec<Vec<PathBuf>> = vec![];
        let mut processed: Vec<bool> = vec![false; files.len()];

        for i in 0..files.len() {
            if processed[i] {
                continue;
            }

            let mut group: Vec<PathBuf> = vec![files[i].clone()];
            processed[i] = true;

            for j in (i + 1)..files.len() {
                if processed[j] {
                    continue;
                }

                if self.quick_compare(&files[i], &files[j]) {
                    group.push(files[j].clone());
                    processed[j] = true;
                }
            }

            if group.len() > 1 {
                groups.push(group);
            }
        }

        groups
    }

    pub fn suggest_files_to_delete(&self, group: &DuplicateGroup) -> Vec<String> {
        if group.files.len() < 2 {
            return vec![];
        }

        let mut files_with_scores: Vec<(usize, i32)> = group
            .files
            .iter()
            .enumerate()
            .map(|(index, file)| {
                let score = self.calculate_delete_score(file);
                (index, score)
            })
            .collect();

        files_with_scores.sort_by(|a, b| b.1.cmp(&a.1));

        let original_index = files_with_scores
            .iter()
            .find(|(_, score)| *score < 0)
            .map(|(index, _)| *index)
            .unwrap_or(0);

        group
            .files
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != original_index)
            .map(|(_, file)| file.path.clone())
            .collect()
    }

    fn calculate_delete_score(&self, file: &DuplicateFile) -> i32 {
        let mut score = 0;

        let path_lower = file.path.to_lowercase();

        if path_lower.contains("\\temp\\") {
            score += 50;
        }

        if path_lower.contains("\\downloads\\") {
            score += 30;
        }

        if path_lower.contains("\\desktop\\") {
            score -= 20;
        }

        if path_lower.contains("\\documents\\") {
            score -= 30;
        }

        if file.is_original {
            score -= 100;
        }

        score
    }

    pub fn suggest_original<'a>(&self, group: &'a DuplicateGroup) -> Option<&'a DuplicateFile> {
        group.files.iter().find(|f| f.is_original)
    }
}

impl Default for DuplicateDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl HashCalculator {
    fn calculate_file_hash_sync(&self, path: &Path) -> Result<HashResult, crate::models::DiskTidyError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| crate::models::DiskTidyError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        rt.block_on(self.calculate_file_hash(path))
    }

    fn calculate_partial_hash_sync(&self, path: &Path) -> Result<HashResult, crate::models::DiskTidyError> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| crate::models::DiskTidyError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        rt.block_on(self.calculate_partial_hash(path))
    }

    fn quick_compare_sync(&self, path1: &Path, path2: &Path) -> bool {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(r) => r,
            Err(_) => return false,
        };
        rt.block_on(self.quick_compare(path1, path2)).unwrap_or(false)
    }
}
