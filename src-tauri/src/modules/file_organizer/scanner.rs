use std::path::Path;
use std::time::UNIX_EPOCH;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// 扫描到的文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub size_bytes: u64,
    pub modified_time: u64,
    pub is_hidden: bool,
    pub category: String,
}

/// 扫描结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub root_path: String,
    pub files: Vec<ScannedFile>,
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub scan_duration_ms: u64,
}

pub struct FileOrganizerScanner;

impl FileOrganizerScanner {
    /// 扫描指定目录下的所有文件
    pub fn scan(
        root_path: &str,
        include_hidden: bool,
        max_depth: Option<usize>,
        extensions_filter: Option<Vec<String>>,
    ) -> ScanResult {
        let start = std::time::Instant::now();
        let scan_id = uuid::Uuid::new_v4().to_string();
        let root = Path::new(root_path);

        let mut files = Vec::new();
        let walker = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(move |e| {
                if !include_hidden {
                    if let Some(name) = e.file_name().to_str() {
                        !name.starts_with('.') && !name.starts_with('$')
                    } else {
                        true
                    }
                } else {
                    true
                }
            });

        for entry in walker {
            if let Ok(entry) = entry {
                // 深度限制
                if let Some(max_d) = max_depth {
                    if entry.depth() > max_d {
                        continue;
                    }
                }

                let path = entry.path();
                if !entry.file_type().is_file() {
                    continue;
                }

                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let extension = path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                // 扩展名过滤
                if let Some(ref filter) = extensions_filter {
                    if !filter.is_empty() && !extension.is_empty() {
                        if !filter.contains(&extension) {
                            continue;
                        }
                    }
                }

                if let Ok(metadata) = entry.metadata() {
                    let size_bytes = metadata.len();
                    let modified_time = metadata.modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);

                    let is_hidden = name.starts_with('.');

                    files.push(ScannedFile {
                        path: path.to_string_lossy().to_string(),
                        name,
                        extension,
                        size_bytes,
                        modified_time,
                        is_hidden,
                        category: String::new(), // 稍后由分类器填充
                    });
                }
            }
        }

        let total_files = files.len() as u64;
        let total_size_bytes = files.iter().map(|f| f.size_bytes).sum();
        let scan_duration_ms = start.elapsed().as_millis() as u64;

        ScanResult {
            scan_id,
            root_path: root_path.to_string(),
            files,
            total_files,
            total_size_bytes,
            scan_duration_ms,
        }
    }

    /// 快速扫描（仅统计，不返回全部文件列表）
    pub fn quick_scan(root_path: &str, include_hidden: bool) -> (u64, u64) {
        let root = Path::new(root_path);
        let mut file_count = 0u64;
        let mut total_size = 0u64;

        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if !include_hidden && name.starts_with('.') {
                        continue;
                    }
                }
                if path.is_file() {
                    if let Ok(meta) = path.metadata() {
                        file_count += 1;
                        total_size += meta.len();
                    }
                }
            }
        }

        (file_count, total_size)
    }
}
