use std::fs;
use std::path::{Path, PathBuf};

use super::path::SystemPaths;

pub struct AppPathResolver;

#[derive(Debug, Clone)]
pub struct ResolvedAppPath {
    pub path: PathBuf,
    pub source: PathSource,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PathSource {
    UserConfig,
    Default,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub id: String,
    pub name: String,
    pub data_path: PathBuf,
}

impl AppPathResolver {
    pub fn resolve_wechat_paths(user_path: Option<&PathBuf>) -> Vec<ResolvedAppPath> {
        let mut paths = Vec::new();

        if let Some(configured_path) = user_path {
            if configured_path.exists() {
                for account_path in Self::find_wechat_accounts(configured_path) {
                    paths.push(ResolvedAppPath {
                        path: account_path.data_path,
                        source: PathSource::UserConfig,
                        account_id: Some(account_path.id),
                    });
                }
            }
        }

        paths
    }

    pub fn resolve_qq_paths(user_path: Option<&PathBuf>) -> Vec<ResolvedAppPath> {
        let mut paths: Vec<ResolvedAppPath> = Vec::new();

        if let Some(configured_path) = user_path {
            if configured_path.exists() {
                for account_path in Self::find_qq_accounts(configured_path) {
                    paths.push(ResolvedAppPath {
                        path: account_path.data_path,
                        source: PathSource::UserConfig,
                        account_id: Some(account_path.id),
                    });
                }
            }
        }

        paths
    }

    pub fn resolve_dingtalk_paths(user_path: Option<&PathBuf>) -> Vec<ResolvedAppPath> {
        let mut paths = Vec::new();

        if let Some(configured_path) = user_path {
            if configured_path.exists() {
                for account_path in Self::find_dingtalk_accounts(configured_path) {
                    paths.push(ResolvedAppPath {
                        path: account_path.data_path,
                        source: PathSource::UserConfig,
                        account_id: Some(account_path.id),
                    });
                }
            }
        }

        paths
    }

    pub fn resolve_wework_paths(user_path: Option<&PathBuf>) -> Vec<ResolvedAppPath> {
        let mut paths: Vec<ResolvedAppPath> = Vec::new();

        if let Some(configured_path) = user_path {
            if configured_path.exists() {
                for account_path in Self::find_wework_accounts(configured_path) {
                    paths.push(ResolvedAppPath {
                        path: account_path.data_path,
                        source: PathSource::UserConfig,
                        account_id: Some(account_path.id),
                    });
                }
            }
        }

        paths
    }

    fn find_wechat_accounts(base_path: &Path) -> Vec<AccountInfo> {
        let mut accounts = Vec::new();
        println!("[find_wechat_accounts] Scanning base_path: {:?}", base_path);

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if name == "All Users"
                        || name == "Applet"
                        || name == "WMPF"
                        || name == "XPlugin"
                    {
                        continue;
                    }

                    let file_storage = path.join("FileStorage");
                    let msg_dir = path.join("Msg");

                    if file_storage.exists() || msg_dir.exists() {
                        accounts.push(AccountInfo {
                            id: name.to_string(),
                            name: name.to_string(),
                            data_path: path,
                        });
                    }
                }
            }
        }

        println!("[find_wechat_accounts] Found {} accounts", accounts.len());
        accounts
    }

    fn find_qq_accounts(base_path: &Path) -> Vec<AccountInfo> {
        let mut accounts = Vec::new();

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if name == "All Users" || name == "Registry" {
                        continue;
                    }

                    if name.chars().all(|c| c.is_numeric()) || name.starts_with("QQ") {
                        let msg_db = path.join("Msg3.0.db");
                        let image_dir = path.join("Image");

                        if msg_db.exists() || image_dir.exists() {
                            accounts.push(AccountInfo {
                                id: name.to_string(),
                                name: name.to_string(),
                                data_path: path,
                            });
                        }
                    }
                }
            }
        }

        accounts
    }

    fn find_dingtalk_accounts(base_path: &Path) -> Vec<AccountInfo> {
        let mut accounts = Vec::new();

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if name == "All Users" || name.starts_with(".") {
                        continue;
                    }

                    accounts.push(AccountInfo {
                        id: name.to_string(),
                        name: name.to_string(),
                        data_path: path,
                    });
                }
            }
        }

        accounts
    }

    fn find_wework_accounts(base_path: &Path) -> Vec<AccountInfo> {
        let mut accounts = Vec::new();

        if let Ok(entries) = fs::read_dir(base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                    if name == "All Users" || name.starts_with(".") {
                        continue;
                    }

                    if path.join("FileStorage").exists() || path.join("Image").exists() {
                        accounts.push(AccountInfo {
                            id: name.to_string(),
                            name: name.to_string(),
                            data_path: path,
                        });
                    }
                }
            }
        }

        accounts
    }
}

pub fn is_hidden_file(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = fs::metadata(path) {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
            let attrs = metadata.file_attributes();
            return (attrs & FILE_ATTRIBUTE_HIDDEN) != 0 || (attrs & FILE_ATTRIBUTE_SYSTEM) != 0;
        }
    }
    #[cfg(not(windows))]
    {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            return name.starts_with('.');
        }
    }
    false
}

pub fn should_skip_file(path: &Path) -> bool {
    if is_hidden_file(path) {
        return true;
    }

    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

    let skip_names = ["desktop.ini", "thumbs.db", ".ds_store", "icon\r"];
    if skip_names.contains(&name.to_lowercase().as_str()) {
        return true;
    }

    false
}

pub fn get_default_config_dir() -> Option<PathBuf> {
    SystemPaths::app_data_local().map(|p| p.join("DiskTidy"))
}
