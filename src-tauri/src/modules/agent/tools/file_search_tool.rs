use std::path::Path;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use super::tool_error::ToolError;
use super::ToolPrompt;

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
    200
}

/// 文件查找分析结果（精简版，避免 LLM token 超限和超时）
#[derive(Debug, Serialize)]
pub struct FileSearchToolOutput {
    pub directory: String,
    pub total_files: usize,
    pub total_dirs: usize,
    pub total_size_bytes: u64,
    pub total_size_display: String,
    /// 文件列表（仅包含名称、大小显示、扩展名，最多 200 条）
    pub files: Vec<FileSummary>,
    pub extension_stats: Vec<ExtensionStat>,
    pub truncated: bool,
    /// 可读摘要
    pub summary: String,
    /// 工具专属提示词
    pub _prompt: String,
}

/// 精简文件摘要
#[derive(Debug, Clone, Serialize)]
pub struct FileSummary {
    pub name: String,
    pub size: String,
    pub ext: String,
}

/// 文件类型统计
#[derive(Debug, Serialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub count: usize,
    pub total_size_bytes: u64,
    pub total_size_display: String,
}

impl ToolPrompt for FileSearchTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 文件查找分析工作流
你正在使用 file_search 工具查找和分析文件，请遵循以下流程：

1. 展示查找结果：目录路径、文件总数、子目录数、总大小
2. 用表格展示文件类型分布：类型 | 数量 | 总大小
3. 列出代表性文件（最多 10 个）
4. 根据文件类型建议后续操作：
   - Excel表格(.xlsx/.xls) → 使用三步骤工作流：read_excel → analyze_data → generate_html
   - 其他文档类(TXT/DOCX/PDF/MD) → 调用 file_content_analyzer 分析内容
   - 大文件 → 调用 large_file_scanner 深度分析
   - 杂乱文件 → 调用 file_organizer 整理
"#
    }
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

        let mut files: Vec<FileSummary> = Vec::new();
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

                let summary = build_summary(&args.path, files.len(), total_dirs, total_size, &ext_stats, truncated);

                return Ok(FileSearchToolOutput {
                    directory: args.path.clone(),
                    total_files: files.len(),
                    total_dirs,
                    total_size_bytes: total_size,
                    total_size_display: format_file_size(total_size),
                    files,
                    extension_stats: ext_stats,
                    truncated,
                    summary,
                    _prompt: self.detailed_prompt().to_string(),
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

            total_size += size;

            // 更新扩展名统计
            let stat = extension_stats.entry(ext.clone()).or_insert((0, 0));
            stat.0 += 1;
            stat.1 += size;

            let file_name = entry.file_name().to_string_lossy().to_string();

            files.push(FileSummary {
                name: file_name,
                size: format_file_size(size),
                ext,
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

        let summary = build_summary(&args.path, files.len(), total_dirs, total_size, &ext_stats, false);

        Ok(FileSearchToolOutput {
            directory: args.path.clone(),
            total_files: files.len(),
            total_dirs,
            total_size_bytes: total_size,
            total_size_display: format_file_size(total_size),
            files,
            extension_stats: ext_stats,
            truncated: false,
            summary,
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}

/// 生成可读摘要
fn build_summary(
    path: &str,
    file_count: usize,
    dir_count: usize,
    total_size: u64,
    ext_stats: &[ExtensionStat],
    truncated: bool,
) -> String {
    let top_exts: Vec<String> = ext_stats
        .iter()
        .take(5)
        .map(|s| format!("{}({}个)", s.extension, s.count))
        .collect();

    format!(
        "目录: {}\n文件总数: {}{}\n子目录: {}\n总大小: {}\n主要类型: {}",
        path,
        file_count,
        if truncated { "(已截断)" } else { "" },
        dir_count,
        format_file_size(total_size),
        if top_exts.is_empty() { "无".to_string() } else { top_exts.join(", ") },
    )
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