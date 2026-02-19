use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

use crate::models::{
    CleanError, CleanOptions, CleanProgress, CleanResult, DiskTidyError, GarbageCategory,
    GarbageFile, DuplicateGroup,
};
use super::safety::SafetyChecker;
use super::recycle_bin::RecycleBin;

pub const SECURE_OVERWRITE_PATTERNS: [u8; 3] = [0x00, 0xFF, 0xAA];
pub const DEFAULT_CHUNK_SIZE: usize = 64 * 1024;

pub struct CleanerExecutor {
    options: CleanOptions,
    safety_checker: SafetyChecker,
    cancelled: Arc<RwLock<bool>>,
}

pub struct CleanContext {
    pub clean_id: String,
    pub total_files: u64,
    pub cleaned_files: u64,
    pub failed_files: u64,
    pub skipped_files: u64,
    pub total_size: u64,
    pub cleaned_size: u64,
    pub errors: Vec<CleanError>,
    pub start_time: Instant,
}

pub type ProgressCallback = Arc<dyn Fn(CleanProgress) + Send + Sync>;

impl CleanerExecutor {
    pub fn new() -> Self {
        Self::with_options(CleanOptions::default())
    }

    pub fn with_options(options: CleanOptions) -> Self {
        Self {
            safety_checker: SafetyChecker::new(),
            options,
            cancelled: Arc::new(RwLock::new(false)),
        }
    }

    pub fn set_options(&mut self, options: CleanOptions) {
        self.options = options;
    }

    pub async fn cancel(&self) {
        let mut cancelled = self.cancelled.write().await;
        *cancelled = true;
    }

    pub async fn is_cancelled(&self) -> bool {
        *self.cancelled.read().await
    }

    pub async fn clean(
        &self,
        files: Vec<PathBuf>,
    ) -> Result<CleanResult, DiskTidyError> {
        self.clean_with_progress(files, None).await
    }

    pub async fn clean_with_progress(
        &self,
        files: Vec<PathBuf>,
        progress_callback: Option<ProgressCallback>,
    ) -> Result<CleanResult, DiskTidyError> {
        let clean_id = uuid::Uuid::new_v4().to_string();
        let mut ctx = CleanContext {
            clean_id: clean_id.clone(),
            total_files: files.len() as u64,
            cleaned_files: 0,
            failed_files: 0,
            skipped_files: 0,
            total_size: 0,
            cleaned_size: 0,
            errors: Vec::new(),
            start_time: Instant::now(),
        };

        for path in &files {
            if self.is_cancelled().await {
                break;
            }

            if let Some(ref callback) = progress_callback {
                callback(CleanProgress {
                    total_files: ctx.total_files,
                    cleaned_files: ctx.cleaned_files,
                    current_file: path.to_string_lossy().to_string(),
                    percent: if ctx.total_files > 0 {
                        ctx.cleaned_files as f32 / ctx.total_files as f32 * 100.0
                    } else {
                        0.0
                    },
                    speed: self.calculate_speed(&ctx),
                });
            }

            match self.clean_single(path).await {
                Ok(size) => {
                    ctx.cleaned_files += 1;
                    ctx.cleaned_size += size;
                }
                Err(e) => {
                    ctx.failed_files += 1;
                    ctx.errors.push(CleanError {
                        path: path.to_string_lossy().to_string(),
                        error_code: e.error_code().to_string(),
                        error_message: e.to_string(),
                    });
                }
            }
        }

        Ok(CleanResult {
            scan_id: clean_id,
            total_files: ctx.total_files,
            cleaned_files: ctx.cleaned_files,
            failed_files: ctx.failed_files,
            skipped_files: ctx.skipped_files,
            total_size: ctx.total_size,
            cleaned_size: ctx.cleaned_size,
            errors: ctx.errors,
            duration_ms: ctx.start_time.elapsed().as_millis() as u64,
        })
    }

    fn calculate_speed(&self, ctx: &CleanContext) -> u64 {
        let elapsed = ctx.start_time.elapsed().as_secs();
        if elapsed > 0 {
            ctx.cleaned_size / elapsed
        } else {
            0
        }
    }

    pub async fn clean_single(&self, path: &Path) -> Result<u64, DiskTidyError> {
        if !path.exists() {
            return Err(DiskTidyError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            });
        }

        let safety_result = self.safety_checker.check(path);
        if !safety_result.safe_to_delete {
            return Err(DiskTidyError::ProtectedPath {
                path: path.to_string_lossy().to_string(),
            });
        }

        let metadata = fs::metadata(path).await.map_err(DiskTidyError::IoError)?;
        let file_size = metadata.len();

        if self.options.move_to_recycle_bin {
            self.move_to_recycle_bin(path).await?;
        } else if self.options.secure_delete {
            self.secure_delete(path, self.options.secure_pass_count).await?;
        } else {
            self.permanent_delete(path).await?;
        }

        Ok(file_size)
    }

    pub async fn permanent_delete(&self, path: &Path) -> Result<(), DiskTidyError> {
        if path.is_dir() {
            fs::remove_dir_all(path)
                .await
                .map_err(|e| self.handle_delete_error(path, e))?;
        } else {
            fs::remove_file(path)
                .await
                .map_err(|e| self.handle_delete_error(path, e))?;
        }

        Ok(())
    }

    fn handle_delete_error(&self, path: &Path, error: std::io::Error) -> DiskTidyError {
        match error.kind() {
            std::io::ErrorKind::PermissionDenied => DiskTidyError::PermissionDenied {
                path: path.to_string_lossy().to_string(),
            },
            std::io::ErrorKind::NotFound => DiskTidyError::FileNotFound {
                path: path.to_string_lossy().to_string(),
            },
            _ => DiskTidyError::IoError(error),
        }
    }

    pub async fn secure_delete(
        &self,
        path: &Path,
        passes: u8,
    ) -> Result<(), DiskTidyError> {
        if !path.is_file() {
            return self.permanent_delete(path).await;
        }

        let metadata = fs::metadata(path).await.map_err(DiskTidyError::IoError)?;
        let file_size = metadata.len();

        for pass in 0..passes {
            self.overwrite_file(path, file_size, pass).await?;
        }

        self.permanent_delete(path).await?;

        Ok(())
    }

    async fn overwrite_file(
        &self,
        path: &Path,
        size: u64,
        pass: u8,
    ) -> Result<(), DiskTidyError> {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .open(path)
            .await
            .map_err(DiskTidyError::IoError)?;

        let pattern = SECURE_OVERWRITE_PATTERNS[pass as usize % SECURE_OVERWRITE_PATTERNS.len()];
        let mut buffer = vec![pattern; DEFAULT_CHUNK_SIZE];
        let mut remaining = size;

        while remaining > 0 {
            let write_size = std::cmp::min(remaining, DEFAULT_CHUNK_SIZE as u64) as usize;
            
            if write_size < DEFAULT_CHUNK_SIZE {
                buffer.truncate(write_size);
            }

            file.write_all(&buffer[..write_size])
                .await
                .map_err(DiskTidyError::IoError)?;

            remaining -= write_size as u64;
        }

        file.flush().await.map_err(DiskTidyError::IoError)?;

        Ok(())
    }

    #[cfg(windows)]
    pub async fn move_to_recycle_bin(&self, path: &Path) -> Result<(), DiskTidyError> {
        RecycleBin::move_to_recycle_bin(path).await
    }

    #[cfg(not(windows))]
    pub async fn move_to_recycle_bin(&self, path: &Path) -> Result<(), DiskTidyError> {
        let trash_path = dirs::trash_dir().ok_or_else(|| DiskTidyError::SystemCallFailed {
            api: "trash_dir".to_string(),
            message: "Cannot find trash directory".to_string(),
        })?;

        let file_name = path.file_name().ok_or_else(|| DiskTidyError::InvalidParameter {
            message: "Invalid file name".to_string(),
        })?;

        let dest = trash_path.join(file_name);
        fs::rename(path, dest)
            .await
            .map_err(DiskTidyError::IoError)?;

        Ok(())
    }

    pub async fn clean_by_category(
        &self,
        category: GarbageCategory,
        garbage_files: &[GarbageFile],
    ) -> Result<CleanResult, DiskTidyError> {
        let files_to_clean: Vec<PathBuf> = garbage_files
            .iter()
            .filter(|f| f.category == category && f.safe_to_delete)
            .map(|f| PathBuf::from(&f.path))
            .collect();

        self.clean(files_to_clean).await
    }

    pub async fn clean_duplicates(
        &self,
        duplicate_groups: &[DuplicateGroup],
        keep_originals: bool,
    ) -> Result<CleanResult, DiskTidyError> {
        let mut files_to_clean = Vec::new();

        for group in duplicate_groups {
            let files: Vec<PathBuf> = if keep_originals {
                group.files.iter()
                    .filter(|f| !f.is_original)
                    .map(|f| PathBuf::from(&f.path))
                    .collect()
            } else {
                group.files.iter()
                    .skip(1)
                    .map(|f| PathBuf::from(&f.path))
                    .collect()
            };
            files_to_clean.extend(files);
        }

        self.clean(files_to_clean).await
    }
}

impl Default for CleanerExecutor {
    fn default() -> Self {
        Self::new()
    }
}
