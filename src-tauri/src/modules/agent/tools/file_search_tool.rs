use std::path::Path;
use std::time::UNIX_EPOCH;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::tool_error::ToolError;

/// 文件查找分析工具输入
#[derive(Debug, Deserialize)]
pub struct FileSearchToolInput {
    /// 要扫描的目录路径
    pub path: String,
    /// 最大递归深度（0 = 仅当前目录，None = 无限）
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// 最多返回文件数（默认 500）
    #[serde(default = "default_max_results")]
    pub max_results: usize,
    /// 是否包含隐藏文件
    #[serde(default)]
    pub include_hidden: bool,
    /// 过滤扩展名（如 ["docx", "pdf"]，空表示不过滤）
    #[serde(default)]
    pub extensions: Vec<String>,
}

fn default_max_results() -> usize {
    500
}

/// 文件条目信息
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_display: String,
    pub modified: String,
    pub is_dir: bool,
    pub extension: String,
}

/// 文件查找分析结果
#[derive(Debug, Serialize)]
pub struct FileSearchToolOutput {
    pub directory: String,
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size_bytes: u64,
    pub total_size_display: String,
    pub files: Vec<FileEntry>,
    pub extension_stats: Vec<ExtensionStat>,
    pub truncated: bool,
}

/// 文件类型统计
#[derive(Debug, Serialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub count: usize,
    pub total_size_bytes: u64,
    pub total_size_display: String,
}

pub struct FileSearchTool;

impl FileSearchTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for FileSearchTool {
    const NAME: &'static str = "file_search";
    type Error = ToolError;
    type Args = FileSearchToolInput;
    type Output = FileSearchToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "查找并分析指定目录下的文件。递归扫描目录，返回完整的文件列表（包含名称、路径、大小、修改日期、类型等信息），并提供文件类型分布统计。用于了解目录内容、分析文件构成。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要扫描的目录路径，例如 'E:/2025' 或 'C:\\\\Users'"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "最大递归深度。0=仅当前目录，不设置=无限递归"
                    },
                    "max_results": {
                        "type": "integer",
                        "description": "最多返回的文件数，默认 500。如果超出将截断并在结果中标记"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "是否包含隐藏文件（.开头的文件），默认 false"
                    },
                    "extensions": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "过滤扩展名列表，如 ['docx', 'pdf']。空数组表示不过滤"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);
        if !path.exists() {
            return Err(ToolError::InvalidArgs(format!(
                "目录不存在: {}",
                args.path
            )));
        }
        if !path.is_dir() {
            return Err(ToolError::InvalidArgs(format!(
                "路径不是目录: {}",
                args.path
            )));
        }

        let mut files: Vec<FileEntry> = Vec::new();
        let mut total_dirs: usize = 0;
        let mut total_size: u64 = 0;
        let mut extension_stats: std::collections::HashMap<String, (usize, u64)> = std::collections::HashMap::new();

        let mut walker_builder = WalkDir::new(path)
            .follow_links(false)
            .sort_by_file_name();

        if let Some(max_depth) = args.max_depth {
            walker_builder = walker_builder.max_depth(max_depth + 1);
        }

        let walker = walker_builder.into_iter();

        for entry in walker {
            let entry = entry.map_err(|e| {
                ToolError::ExecutionFailed(format!("读取文件失败: {}", e))
            })?;

            // 跳过隐藏文件
            if !args.include_hidden {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with('.') {
                        continue;
                    }
                }
            }

            if entry.file_type().is_dir() {
                total_dirs += 1;
                continue; // 只统计目录，不加入文件列表
            }

            if files.len() >= args.max_results {
                let truncated = true;
                // 完成统计
                let ext_stats: Vec<ExtensionStat> = extension_stats
                    .into_iter()
                    .map(|(ext, (count, size))| ExtensionStat {
                        extension: ext,
                        count,
                        total_size_bytes: size,
                        total_size_display: format_file_size(size),
                    })
                    .collect();

                return Ok(FileSearchToolOutput {
                    directory: args.path.clone(),
                    total_files: files.len(),
                    total_dirs,
                    total_size_bytes: total_size,
                    total_size_display: format_file_size(total_size),
                    files,
                    extension_stats: ext_stats,
                    truncated,
                });
            }

            let ext = entry
                .path()
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("无扩展名")
                .to_lowercase();

            // 扩展名过滤
            if !args.extensions.is_empty() && !args.extensions.iter().any(|e| e.to_lowercase() == ext) {
                continue;
            }

            let metadata = entry.metadata().map_err(|e| {
                ToolError::ExecutionFailed(format!("读取文件元数据失败: {}", e))
            })?;

            let size = metadata.len();
            let modified = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| {
                    let secs = d.as_secs();
                    let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
                        .unwrap_or_default();
                    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|| "未知".to_string());

            total_size += size;

            // 更新扩展名统计
            let stat = extension_stats.entry(ext.clone()).or_insert((0, 0));
            stat.0 += 1;
            stat.1 += size;

            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path().to_string_lossy().to_string();

            files.push(FileEntry {
                name: file_name,
                path: file_path,
                size_bytes: size,
                size_display: format_file_size(size),
                modified,
                is_dir: false,
                extension: ext,
            });
        }

        let ext_stats: Vec<ExtensionStat> = extension_stats
            .into_iter()
            .map(|(ext, (count, size))| ExtensionStat {
                extension: ext,
                count,
                total_size_bytes: size,
                total_size_display: format_file_size(size),
            })
            .collect();

        Ok(FileSearchToolOutput {
            directory: args.path.clone(),
            total_files: files.len(),
            total_dirs,
            total_size_bytes: total_size,
            total_size_display: format_file_size(total_size),
            files,
            extension_stats: ext_stats,
            truncated: false,
        })
    }
}

fn format_file_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, UNITS[unit_idx])
}