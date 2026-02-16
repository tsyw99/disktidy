use std::path::Path;
use crate::models::DiskTidyError;

pub struct RecycleBin;

impl RecycleBin {
    #[cfg(windows)]
    pub async fn move_to_recycle_bin(path: &Path) -> Result<(), DiskTidyError> {
        use std::ptr;
        use windows::Win32::UI::Shell::{
            SHFileOperationW,
            SHFILEOPSTRUCTW,
            FO_DELETE,
            FOF_ALLOWUNDO,
            FOF_NOCONFIRMATION,
            FOF_SILENT,
        };
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::{FALSE, HWND};

        let path_wide: Vec<u16> = path
            .to_string_lossy()
            .encode_utf16()
            .chain(std::iter::once(0))
            .chain(std::iter::once(0))
            .collect();

        unsafe {
            let mut op = SHFILEOPSTRUCTW {
                hwnd: HWND(ptr::null_mut()),
                wFunc: FO_DELETE,
                pFrom: PCWSTR(path_wide.as_ptr()),
                pTo: PCWSTR::null(),
                fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT).0 as u16,
                fAnyOperationsAborted: FALSE,
                hNameMappings: ptr::null_mut(),
                lpszProgressTitle: PCWSTR::null(),
            };

            let result = SHFileOperationW(&mut op);

            if result != 0 {
                return Err(DiskTidyError::SystemCallFailed {
                    api: "SHFileOperationW".to_string(),
                    message: format!("Error code: {}", result),
                });
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn move_to_recycle_bin(path: &Path) -> Result<(), DiskTidyError> {
        let trash_path = dirs::trash_dir().ok_or_else(|| DiskTidyError::SystemCallFailed {
            api: "trash_dir".to_string(),
            message: "Cannot find trash directory".to_string(),
        })?;

        let file_name = path.file_name().ok_or_else(|| DiskTidyError::InvalidParameter {
            message: "Invalid file name".to_string(),
        })?;

        let dest = trash_path.join(file_name);
        tokio::fs::rename(path, dest)
            .await
            .map_err(DiskTidyError::IoError)?;

        Ok(())
    }

    #[cfg(windows)]
    pub async fn empty_recycle_bin() -> Result<(), DiskTidyError> {
        use windows::Win32::UI::Shell::SHEmptyRecycleBinW;
        use windows::Win32::UI::Shell::SHERB_NOCONFIRMATION;
        use windows::Win32::UI::Shell::SHERB_NOPROGRESSUI;
        use windows::Win32::UI::Shell::SHERB_NOSOUND;
        use windows::core::PCWSTR;
        use windows::Win32::Foundation::HWND;

        unsafe {
            let result = SHEmptyRecycleBinW(
                Some(HWND(std::ptr::null_mut())),
                PCWSTR::null(),
                SHERB_NOCONFIRMATION | SHERB_NOPROGRESSUI | SHERB_NOSOUND,
            );

            if result.is_err() {
                return Err(DiskTidyError::SystemCallFailed {
                    api: "SHEmptyRecycleBinW".to_string(),
                    message: "Failed to empty recycle bin".to_string(),
                });
            }
        }

        Ok(())
    }

    #[cfg(not(windows))]
    pub async fn empty_recycle_bin() -> Result<(), DiskTidyError> {
        let trash_path = dirs::trash_dir().ok_or_else(|| DiskTidyError::SystemCallFailed {
            api: "trash_dir".to_string(),
            message: "Cannot find trash directory".to_string(),
        })?;

        if trash_path.exists() {
            tokio::fs::remove_dir_all(&trash_path)
                .await
                .map_err(DiskTidyError::IoError)?;
            tokio::fs::create_dir_all(&trash_path)
                .await
                .map_err(DiskTidyError::IoError)?;
        }

        Ok(())
    }

    #[cfg(windows)]
    pub async fn get_recycle_bin_info() -> Result<RecycleBinInfo, DiskTidyError> {
        use windows::Win32::UI::Shell::SHQueryRecycleBinW;
        use windows::Win32::UI::Shell::SHQUERYRBINFO;
        use windows::core::PCWSTR;

        let mut info = SHQUERYRBINFO {
            cbSize: std::mem::size_of::<SHQUERYRBINFO>() as u32,
            i64Size: 0,
            i64NumItems: 0,
        };

        unsafe {
            let result = SHQueryRecycleBinW(PCWSTR::null(), &mut info);

            if result.is_err() {
                return Err(DiskTidyError::SystemCallFailed {
                    api: "SHQueryRecycleBinW".to_string(),
                    message: "Failed to query recycle bin".to_string(),
                });
            }
        }

        Ok(RecycleBinInfo {
            total_size: info.i64Size as u64,
            total_items: info.i64NumItems as u64,
        })
    }

    #[cfg(not(windows))]
    pub async fn get_recycle_bin_info() -> Result<RecycleBinInfo, DiskTidyError> {
        let trash_path = dirs::trash_dir().ok_or_else(|| DiskTidyError::SystemCallFailed {
            api: "trash_dir".to_string(),
            message: "Cannot find trash directory".to_string(),
        })?;

        let mut total_size: u64 = 0;
        let mut total_items: u64 = 0;

        if trash_path.exists() {
            if let Ok(mut entries) = tokio::fs::read_dir(&trash_path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        if metadata.is_file() {
                            total_size += metadata.len();
                            total_items += 1;
                        } else if metadata.is_dir() {
                            total_items += 1;
                            total_size += Self::get_dir_size(&entry.path()).await;
                        }
                    }
                }
            }
        }

        Ok(RecycleBinInfo {
            total_size,
            total_items,
        })
    }

    #[cfg(not(windows))]
    async fn get_dir_size(path: &Path) -> u64 {
        let mut size = 0;
        if let Ok(mut entries) = tokio::fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(metadata) = entry.metadata().await {
                    if metadata.is_file() {
                        size += metadata.len();
                    } else if metadata.is_dir() {
                        size += Self::get_dir_size(&entry.path()).await;
                    }
                }
            }
        }
        size
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecycleBinInfo {
    pub total_size: u64,
    pub total_items: u64,
}
