use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

use crate::modules::software_residue::{
    ResidueScanOptions, ResidueScanProgress, ResidueScanResult, SoftwareResidueScanner,
};

pub struct ResidueScanState {
    scanner: Arc<RwLock<Option<SoftwareResidueScanner>>>,
    results: Arc<RwLock<Vec<ResidueScanResult>>>,
}

impl ResidueScanState {
    pub fn new() -> Self {
        Self {
            scanner: Arc::new(RwLock::new(None)),
            results: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for ResidueScanState {
    fn default() -> Self {
        Self::new()
    }
}

#[tauri::command]
pub async fn residue_scan_start(
    state: State<'_, ResidueScanState>,
    options: Option<ResidueScanOptions>,
) -> Result<Vec<ResidueScanResult>, String> {
    let scan_options = options.unwrap_or_default();

    let scanner = SoftwareResidueScanner::with_options(scan_options);

    {
        let mut scanner_lock = state.scanner.write().await;
        *scanner_lock = Some(scanner);
    }

    let scanner_lock = state.scanner.read().await;
    if let Some(scanner) = scanner_lock.as_ref() {
        let results = scanner.start_scan().await?;

        {
            let mut results_lock = state.results.write().await;
            *results_lock = results.clone();
        }

        Ok(results)
    } else {
        Err("扫描器未初始化".to_string())
    }
}

#[tauri::command]
pub async fn residue_scan_pause(state: State<'_, ResidueScanState>) -> Result<(), String> {
    let scanner_lock = state.scanner.read().await;
    if let Some(scanner) = scanner_lock.as_ref() {
        scanner.pause_scan();
        Ok(())
    } else {
        Err("没有正在进行的扫描".to_string())
    }
}

#[tauri::command]
pub async fn residue_scan_resume(state: State<'_, ResidueScanState>) -> Result<(), String> {
    let scanner_lock = state.scanner.read().await;
    if let Some(scanner) = scanner_lock.as_ref() {
        scanner.resume_scan();
        Ok(())
    } else {
        Err("没有正在进行的扫描".to_string())
    }
}

#[tauri::command]
pub async fn residue_scan_cancel(state: State<'_, ResidueScanState>) -> Result<(), String> {
    let scanner_lock = state.scanner.read().await;
    if let Some(scanner) = scanner_lock.as_ref() {
        scanner.cancel_scan();
        Ok(())
    } else {
        Err("没有正在进行的扫描".to_string())
    }
}

#[tauri::command]
pub async fn residue_scan_progress(
    state: State<'_, ResidueScanState>,
) -> Result<Option<ResidueScanProgress>, String> {
    let scanner_lock = state.scanner.read().await;
    if let Some(scanner) = scanner_lock.as_ref() {
        Ok(scanner.get_progress().await)
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn residue_scan_result(
    state: State<'_, ResidueScanState>,
) -> Result<Vec<ResidueScanResult>, String> {
    let results = state.results.read().await;
    Ok(results.clone())
}

#[tauri::command]
pub async fn residue_scan_clear(state: State<'_, ResidueScanState>) -> Result<(), String> {
    let mut scanner_lock = state.scanner.write().await;
    *scanner_lock = None;

    let mut results_lock = state.results.write().await;
    results_lock.clear();

    Ok(())
}

#[tauri::command]
pub async fn residue_delete_items(
    state: State<'_, ResidueScanState>,
    item_ids: Vec<String>,
    move_to_recycle_bin: bool,
) -> Result<DeleteResidueResult, String> {
    let results = state.results.read().await;

    let mut deleted_count = 0;
    let mut deleted_size: u64 = 0;
    let mut failed_items: Vec<FailedItem> = Vec::new();

    for result in results.iter() {
        for item in result.items.iter() {
            if item_ids.contains(&item.id) {
                if item.residue_type == crate::modules::software_residue::ResidueType::RegistryKey {
                    failed_items.push(FailedItem {
                        id: item.id.clone(),
                        path: item.path.clone(),
                        error: "注册表项暂不支持删除".to_string(),
                    });
                    continue;
                }

                let path = std::path::Path::new(&item.path);
                if path.exists() {
                    let delete_result = if move_to_recycle_bin {
                        move_to_recycle_bin_internal(path)
                    } else {
                        std::fs::remove_dir_all(path).map(|_| ())
                    };

                    match delete_result {
                        Ok(()) => {
                            deleted_count += 1;
                            deleted_size += item.size;
                        }
                        Err(e) => {
                            failed_items.push(FailedItem {
                                id: item.id.clone(),
                                path: item.path.clone(),
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(DeleteResidueResult {
        deleted_count,
        deleted_size,
        failed_count: failed_items.len() as u32,
        failed_items,
    })
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DeleteResidueResult {
    pub deleted_count: u32,
    pub deleted_size: u64,
    pub failed_count: u32,
    pub failed_items: Vec<FailedItem>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FailedItem {
    pub id: String,
    pub path: String,
    pub error: String,
}

#[cfg(windows)]
fn move_to_recycle_bin_internal(path: &std::path::Path) -> Result<(), std::io::Error> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::HWND;
    use windows::Win32::UI::Shell::{SHFileOperationW, SHFILEOPSTRUCTW};

    unsafe {
        let path_wide: Vec<u16> = OsStr::new(path)
            .encode_wide()
            .chain(std::iter::once(0))
            .chain(std::iter::once(0))
            .collect();

        let mut op = SHFILEOPSTRUCTW {
            hwnd: HWND(std::ptr::null_mut()),
            wFunc: 0x0003,
            pFrom: PCWSTR(path_wide.as_ptr()),
            pTo: PCWSTR::null(),
            fFlags: 0x0040 | 0x0100 | 0x0200,
            fAnyOperationsAborted: false.into(),
            hNameMappings: std::ptr::null_mut(),
            lpszProgressTitle: PCWSTR::null(),
        };

        let result = SHFileOperationW(&mut op);
        if result == 0 {
            Ok(())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("删除失败，错误码: {}", result),
            ))
        }
    }
}

#[cfg(not(windows))]
fn move_to_recycle_bin_internal(path: &std::path::Path) -> Result<(), std::io::Error> {
    std::fs::remove_dir_all(path)
}
