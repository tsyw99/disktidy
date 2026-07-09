use std::path::Path;
use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use log::{info, warn, debug};

use super::tool_error::ToolError;
use super::ToolPrompt;

/// 文件删除工具输入
#[derive(Debug, Deserialize)]
pub struct FileDeleteToolInput {
    /// 删除模式: "by_paths" 按路径删除, "by_pattern" 按类型删除
    pub mode: String,
    /// 要删除的文件路径列表（by_paths 模式）
    #[serde(default)]
    pub paths: Vec<String>,
    /// 目标目录（by_pattern 模式）
    #[serde(default)]
    pub directory: String,
    /// 文件扩展名模式（by_pattern 模式），如 "docx", "pdf"
    #[serde(default)]
    pub pattern: String,
    /// 是否递归子目录（by_pattern 模式），默认 false
    #[serde(default)]
    pub recursive: bool,
    /// 是否确认删除。false=预览模式（仅列出待删除文件），true=执行删除
    #[serde(default)]
    pub confirmed: bool,
}

/// 待删除文件信息
#[derive(Debug, Clone, Serialize)]
pub struct DeleteCandidate {
    pub name: String,
    pub path: String,
    pub size_bytes: u64,
    pub size_display: String,
}

/// 文件删除工具输出
#[derive(Debug, Serialize)]
pub struct FileDeleteToolOutput {
    /// 是否为预览模式
    pub preview: bool,
    /// 待删除文件列表
    pub candidates: Vec<DeleteCandidate>,
    /// 成功删除的文件数
    pub deleted_count: usize,
    /// 删除失败的文件数
    pub failed_count: usize,
    /// 错误列表
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    /// 提示消息
    pub message: String,
    /// 工具专属提示词
    pub _prompt: String,
}

impl ToolPrompt for FileDeleteTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 删除工作流（严格遵守）
你正在使用 file_delete 工具进行文件删除，必须严格遵守以下安全流程：

1. 调用 file_delete 设置 confirmed=false 获取待删除文件预览
2. 向用户展示完整的文件列表和影响分析（表格形式）
3. **必须等待用户明确确认**（"确认删除"、"是的"、"删除吧" 等）后才能继续
4. 用户确认后，再次调用 file_delete 设置 confirmed=true 执行删除
5. 报告删除结果（成功/失败统计）
"#
    }
}

pub struct FileDeleteTool;

impl FileDeleteTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for FileDeleteTool {
    const NAME: &'static str = "file_delete";
    type Error = ToolError;
    type Args = FileDeleteToolInput;
    type Output = FileDeleteToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "安全删除指定文件。支持两种模式：1) by_paths - 按文件路径删除；2) by_pattern - 按文件类型删除目录下所有匹配文件。采用两步确认机制：首次调用时设置 confirmed=false 预览待删除文件列表，用户确认后设置 confirmed=true 执行删除。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "mode": {
                        "type": "string",
                        "enum": ["by_paths", "by_pattern"],
                        "description": "删除模式：by_paths=按指定路径删除，by_pattern=按文件类型删除"
                    },
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要删除的文件路径列表（by_paths 模式使用）"
                    },
                    "directory": {
                        "type": "string",
                        "description": "目标目录（by_pattern 模式使用）"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "文件扩展名，如 'docx'、'pdf'（by_pattern 模式使用）"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "是否递归子目录（by_pattern 模式），默认 false"
                    },
                    "confirmed": {
                        "type": "boolean",
                        "description": "是否确认删除。false=仅预览待删除文件，true=执行删除。必须先在预览模式下展示文件列表并获得用户确认后，才能设置为 true"
                    }
                },
                "required": ["mode", "confirmed"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        info!(
            "file_delete 被调用: mode={}, confirmed={}, paths={:?}, directory={}, pattern={}",
            args.mode, args.confirmed, args.paths, args.directory, args.pattern
        );

        // 第一步：收集待删除文件列表
        let candidates = match args.mode.as_str() {
            "by_paths" => collect_by_paths(&args.paths)?,
            "by_pattern" => collect_by_pattern(&args.directory, &args.pattern, args.recursive)?,
            _ => {
                return Err(ToolError::InvalidArgs(format!(
                    "不支持的删除模式: {}。请使用 'by_paths' 或 'by_pattern'",
                    args.mode
                )));
            }
        };

        if candidates.is_empty() {
            warn!("file_delete: 没有找到匹配的文件");
            return Ok(FileDeleteToolOutput {
                preview: !args.confirmed,
                candidates: vec![],
                deleted_count: 0,
                failed_count: 0,
                errors: vec![],
                message: "没有找到匹配的文件".to_string(),
                _prompt: self.detailed_prompt().to_string(),
            });
        }

        // 预览模式：仅返回文件列表
        if !args.confirmed {
            info!("file_delete: 预览模式，返回 {} 个待删除文件", candidates.len());
            let total_display: u64 = candidates.iter().map(|c| c.size_bytes).sum();
            let preview_msg = format!(
                "⚠️ 预览模式 — 待删除 {} 个文件，总计 {}。\n请向用户展示以下文件列表，确认是否继续删除：",
                candidates.len(),
                format_file_size(total_display)
            );

            return Ok(FileDeleteToolOutput {
                preview: true,
                candidates,
                deleted_count: 0,
                failed_count: 0,
                errors: vec![],
                message: preview_msg,
                _prompt: self.detailed_prompt().to_string(),
            });
        }

        // 确认模式：执行删除
        info!("file_delete: 确认删除模式，开始删除 {} 个文件", candidates.len());
        let mut deleted = 0;
        let mut failed = 0;
        let mut errors: Vec<String> = Vec::new();

        for candidate in &candidates {
            debug!("file_delete: 正在删除 {}", candidate.path);
            match std::fs::remove_file(&candidate.path) {
                Ok(()) => {
                    deleted += 1;
                    info!("file_delete: 成功删除 {}", candidate.path);
                }
                Err(e) => {
                    failed += 1;
                    warn!("file_delete: 删除失败 {}: {}", candidate.path, e);
                    errors.push(format!("{}: {}", candidate.name, e));
                }
            }
        }

        info!("file_delete: 删除完成 — 成功 {} 个，失败 {} 个", deleted, failed);

        let result_msg = if failed == 0 {
            format!(
                "✅ 删除完成：成功删除 {} 个文件",
                deleted
            )
        } else {
            format!(
                "⚠️ 删除完成：成功 {} 个，失败 {} 个",
                deleted, failed
            )
        };

        Ok(FileDeleteToolOutput {
            preview: false,
            candidates,
            deleted_count: deleted,
            failed_count: failed,
            errors,
            message: result_msg,
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}

/// 按路径收集文件
fn collect_by_paths(paths: &[String]) -> Result<Vec<DeleteCandidate>, ToolError> {
    let mut candidates = Vec::new();

    for path_str in paths {
        let path = Path::new(path_str);
        if !path.exists() {
            warn!("file_delete: 路径不存在: {}", path_str);
            continue; // 跳过不存在的文件
        }
        if !path.is_file() {
            warn!("file_delete: 路径不是文件: {}", path_str);
            continue; // 跳过目录
        }

        let metadata = std::fs::metadata(path).map_err(|e| {
            ToolError::ExecutionFailed(format!("读取文件失败 {}: {}", path_str, e))
        })?;

        debug!("file_delete: 收集候选文件: {} ({} bytes)", path_str, metadata.len());

        candidates.push(DeleteCandidate {
            name: path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone()),
            path: path_str.clone(),
            size_bytes: metadata.len(),
            size_display: format_file_size(metadata.len()),
        });
    }

    info!("file_delete: collect_by_paths 找到 {} 个候选文件", candidates.len());
    Ok(candidates)
}

/// 按扩展名模式收集文件
fn collect_by_pattern(directory: &str, pattern: &str, recursive: bool) -> Result<Vec<DeleteCandidate>, ToolError> {
    let dir = Path::new(directory);
    if !dir.exists() {
        return Err(ToolError::InvalidArgs(format!("目录不存在: {}", directory)));
    }
    if !dir.is_dir() {
        return Err(ToolError::InvalidArgs(format!("路径不是目录: {}", directory)));
    }

    let pattern_lower = pattern.trim_start_matches('.').to_lowercase();

    let candidates = if recursive {
        // 递归读取
        collect_files_recursive(dir, &pattern_lower)?
    } else {
        // 仅当前目录
        collect_files_flat(dir, &pattern_lower)?
    };

    Ok(candidates)
}

fn collect_files_recursive(dir: &Path, pattern: &str) -> Result<Vec<DeleteCandidate>, ToolError> {
    let mut candidates = Vec::new();

    for entry in walkdir::WalkDir::new(dir).max_depth(10).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let ext = entry.path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != pattern {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| {
            ToolError::ExecutionFailed(format!("读取文件元数据失败: {}", e))
        })?;

        candidates.push(DeleteCandidate {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            size_bytes: metadata.len(),
            size_display: format_file_size(metadata.len()),
        });
    }

    Ok(candidates)
}

fn collect_files_flat(dir: &Path, pattern: &str) -> Result<Vec<DeleteCandidate>, ToolError> {
    let mut candidates = Vec::new();

    let entries = std::fs::read_dir(dir).map_err(|e| {
        ToolError::ExecutionFailed(format!("读取目录失败: {}", e))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            ToolError::ExecutionFailed(format!("读取目录项失败: {}", e))
        })?;

        if !entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
            continue;
        }

        let ext = entry.path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != pattern {
            continue;
        }

        let metadata = entry.metadata().map_err(|e| {
            ToolError::ExecutionFailed(format!("读取文件元数据失败: {}", e))
        })?;

        candidates.push(DeleteCandidate {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            size_bytes: metadata.len(),
            size_display: format_file_size(metadata.len()),
        });
    }

    Ok(candidates)
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