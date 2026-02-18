use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use futures::future::BoxFuture;

use crate::models::{
    CleanOptions, CleanPreview, CleanProgress, CleanResult, CleanStatus,
    ProtectedFile, DiskTidyError, ErrorResponse,
    GarbageFile, DuplicateGroup,
    EVENT_CLEAN_PROGRESS, EVENT_CLEAN_COMPLETE,
};
use crate::modules::cleaner::{
    CleanerExecutor, SafetyChecker, RecycleBin, RecycleBinInfo,
    CleanReportGenerator, CleanReportData,
};

pub struct CleanManager {
    cleans: Arc<RwLock<std::collections::HashMap<String, CleanState>>>,
}

struct CleanState {
    status: CleanStatus,
    cancelled: bool,
}

impl CleanManager {
    pub fn new() -> Self {
        Self {
            cleans: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl Default for CleanManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOptionsJson {
    pub move_to_recycle_bin: bool,
    pub secure_delete: bool,
    pub secure_pass_count: u8,
    #[serde(default)]
    pub confirmed: bool,
}

impl Default for CleanOptionsJson {
    fn default() -> Self {
        Self {
            move_to_recycle_bin: true,
            secure_delete: false,
            secure_pass_count: 3,
            confirmed: false,
        }
    }
}

impl CleanOptionsJson {
    pub fn validate(&self) -> Result<(), String> {
        if !self.move_to_recycle_bin && !self.confirmed {
            return Err("永久删除操作需要用户确认。请设置 confirmed: true 以确认永久删除文件。".to_string());
        }
        if self.secure_delete && self.secure_pass_count == 0 {
            return Err("安全删除的覆盖次数必须大于 0".to_string());
        }
        if self.secure_pass_count > 35 {
            return Err("安全删除的覆盖次数不能超过 35 次".to_string());
        }
        Ok(())
    }
}

impl From<CleanOptionsJson> for CleanOptions {
    fn from(opts: CleanOptionsJson) -> Self {
        CleanOptions {
            move_to_recycle_bin: opts.move_to_recycle_bin,
            secure_delete: opts.secure_delete,
            secure_pass_count: opts.secure_pass_count,
        }
    }
}

#[tauri::command]
pub async fn clean_preview(
    files: Vec<String>,
) -> Result<CleanPreview, ErrorResponse> {
    let safety_checker = SafetyChecker::new();
    let mut total_files: u64 = 0;
    let mut total_size: u64 = 0;
    let mut protected_files = Vec::new();
    let mut warnings = Vec::new();

    for file_path in &files {
        let path = std::path::Path::new(file_path);

        if !path.exists() {
            warnings.push(format!("文件不存在: {}", file_path));
            continue;
        }

        let safety_result = safety_checker.check(path);
        if !safety_result.safe_to_delete {
            protected_files.push(ProtectedFile {
                path: file_path.clone(),
                reason: safety_result.reason.unwrap_or_else(|| "受保护文件".to_string()),
            });
            continue;
        }

        if let Ok(metadata) = tokio::fs::metadata(path).await {
            if metadata.is_file() {
                total_files += 1;
                total_size += metadata.len();
            } else if metadata.is_dir() {
                let (count, size) = get_dir_info(path).await;
                total_files += count;
                total_size += size;
            }
        }
    }

    Ok(CleanPreview {
        total_files,
        total_size,
        protected_files,
        warnings,
    })
}

fn get_dir_info<'a>(path: &'a std::path::Path) -> BoxFuture<'a, (u64, u64)> {
    Box::pin(async move {
        let mut count = 0u64;
        let mut size = 0u64;

        if let Ok(mut entries) = tokio::fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        count += 1;
                        size += metadata.len();
                    } else if metadata.is_dir() {
                        let (sub_count, sub_size) = get_dir_info(entry.path().as_path()).await;
                        count += sub_count;
                        size += sub_size;
                    }
                }
            }
        }

        (count, size)
    })
}

#[tauri::command]
pub async fn clean_files(
    files: Vec<String>,
    options: Option<CleanOptionsJson>,
    app: AppHandle,
    manager: State<'_, CleanManager>,
) -> Result<String, ErrorResponse> {
    let opts = options.unwrap_or_default();
    
    if let Err(e) = opts.validate() {
        return Err(ErrorResponse::from(DiskTidyError::InvalidParameter { message: e }));
    }
    
    let clean_id = Uuid::new_v4().to_string();
    let opts: CleanOptions = opts.into();

    {
        let mut cleans = manager.cleans.write().await;
        cleans.insert(clean_id.clone(), CleanState {
            status: CleanStatus::Running,
            cancelled: false,
        });
    }

    let clean_id_clone = clean_id.clone();
    let cleans = manager.cleans.clone();
    let app_clone = app.clone();
    let path_bufs: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();

    tokio::spawn(async move {
        if let Err(e) = perform_clean(
            clean_id_clone.clone(),
            path_bufs,
            opts,
            cleans,
            app_clone,
        ).await {
            eprintln!("Clean error: {}", e);
        }
    });

    Ok(clean_id)
}

#[tauri::command]
pub async fn clean_garbage_files(
    garbage_files: Vec<GarbageFile>,
    options: Option<CleanOptionsJson>,
    app: AppHandle,
    manager: State<'_, CleanManager>,
) -> Result<String, ErrorResponse> {
    let files: Vec<String> = garbage_files
        .iter()
        .filter(|f| f.safe_to_delete)
        .map(|f| f.path.clone())
        .collect();

    clean_files(files, options, app, manager).await
}

#[tauri::command]
pub async fn clean_duplicates(
    duplicate_groups: Vec<DuplicateGroup>,
    keep_originals: bool,
    options: Option<CleanOptionsJson>,
    app: AppHandle,
    manager: State<'_, CleanManager>,
) -> Result<String, ErrorResponse> {
    let mut files = Vec::new();

    for group in duplicate_groups {
        let group_files: Vec<String> = if keep_originals {
            group.files.iter()
                .filter(|f| !f.is_original)
                .map(|f| f.path.clone())
                .collect()
        } else {
            group.files.iter()
                .skip(1)
                .map(|f| f.path.clone())
                .collect()
        };
        files.extend(group_files);
    }

    clean_files(files, options, app, manager).await
}

#[tauri::command]
pub async fn clean_cancel(
    clean_id: String,
    manager: State<'_, CleanManager>,
) -> Result<(), ErrorResponse> {
    let mut cleans = manager.cleans.write().await;
    if let Some(state) = cleans.get_mut(&clean_id) {
        state.cancelled = true;
        state.status = CleanStatus::Cancelled;
    }
    Ok(())
}

#[tauri::command]
pub async fn clean_status(
    clean_id: String,
    manager: State<'_, CleanManager>,
) -> Result<CleanStatus, ErrorResponse> {
    let cleans = manager.cleans.read().await;
    if let Some(state) = cleans.get(&clean_id) {
        Ok(state.status.clone())
    } else {
        Err(DiskTidyError::CleanFileNotFound(clean_id).into())
    }
}

#[tauri::command]
pub async fn empty_recycle_bin() -> Result<(), String> {
    RecycleBin::empty_recycle_bin()
        .await
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn get_recycle_bin_info() -> Result<RecycleBinInfo, String> {
    RecycleBin::get_recycle_bin_info()
        .await
        .map_err(|e| format!("{}: {}", e.error_code(), e))
}

#[tauri::command]
pub async fn check_file_safety(
    path: String,
) -> Result<SafetyCheckResultJson, String> {
    let checker = SafetyChecker::new();
    let result = checker.check(PathBuf::from(&path).as_path());

    Ok(SafetyCheckResultJson {
        safe_to_delete: result.safe_to_delete,
        risk_level: format!("{:?}", result.risk_level),
        reason: result.reason,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCheckResultJson {
    pub safe_to_delete: bool,
    pub risk_level: String,
    pub reason: Option<String>,
}

#[tauri::command]
pub async fn generate_clean_report(
    result: CleanResult,
) -> CleanReportData {
    let generator = CleanReportGenerator::with_id(result.scan_id.clone());
    generator.generate(&result)
}

#[tauri::command]
pub async fn export_report_json(
    report: CleanReportData,
) -> Result<String, String> {
    let generator = CleanReportGenerator::with_id(report.clean_id.clone());
    generator.export_json(&report)
        .map_err(|e| format!("JSON export error: {}", e))
}

#[tauri::command]
pub async fn export_report_html(
    report: CleanReportData,
) -> String {
    let generator = CleanReportGenerator::with_id(report.clean_id.clone());
    generator.export_html(&report)
}

/// 将文件移动到回收站
#[tauri::command]
pub async fn move_files_to_recycle_bin(
    paths: Vec<String>,
) -> Result<(), String> {
    for path_str in paths {
        let path = PathBuf::from(&path_str);
        if path.exists() {
            RecycleBin::move_to_recycle_bin(&path)
                .await
                .map_err(|e| format!("{}: {}", e.error_code(), e))?;
        }
    }
    Ok(())
}

async fn perform_clean(
    clean_id: String,
    files: Vec<PathBuf>,
    options: CleanOptions,
    cleans: Arc<RwLock<std::collections::HashMap<String, CleanState>>>,
    app: AppHandle,
) -> Result<(), DiskTidyError> {
    let executor = CleanerExecutor::with_options(options.clone());
    let app_for_callback = app.clone();
    let cleans_for_callback = cleans.clone();
    let clean_id_for_callback = clean_id.clone();

    let progress_callback = Arc::new(move |progress: CleanProgress| {
        let app = app_for_callback.clone();
        let cleans = cleans_for_callback.clone();
        let clean_id = clean_id_for_callback.clone();

        tokio::spawn(async move {
            let cancelled = {
                let cleans = cleans.read().await;
                if let Some(state) = cleans.get(&clean_id) {
                    state.cancelled
                } else {
                    false
                }
            };

            if !cancelled {
                let _ = app.emit(EVENT_CLEAN_PROGRESS, &progress);
            }
        });
    });

    {
        let cleans = cleans.read().await;
        if let Some(state) = cleans.get(&clean_id) {
            if state.cancelled {
                return Ok(());
            }
        }
    }

    let result = executor.clean_with_progress(files, Some(progress_callback)).await?;

    {
        let mut cleans = cleans.write().await;
        if let Some(state) = cleans.get_mut(&clean_id) {
            state.status = CleanStatus::Completed;
        }
    }

    let _ = app.emit(EVENT_CLEAN_COMPLETE, &result);

    Ok(())
}
