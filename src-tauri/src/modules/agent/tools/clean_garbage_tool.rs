use serde::{Deserialize, Serialize};
use std::path::Path;

use super::tool_error::ToolError;
use super::ToolPrompt;

// ── input ──
#[derive(Debug, Deserialize)]
pub struct CleanGarbageInput {
    /// 要清理的文件路径列表
    pub files_to_delete: Vec<String>,
    /// 要移动到回收站的文件路径列表
    #[serde(default)]
    pub files_to_trash: Vec<String>,
    /// 是否确认执行（安全检查）
    pub confirmed: bool,
}

// ── output ──
#[derive(Debug, Serialize)]
pub struct CleanGarbageOutput {
    pub success: bool,
    pub deleted_count: usize,
    pub trashed_count: usize,
    pub failed_count: usize,
    pub total_size_freed: u64,
    pub errors: Vec<String>,
    pub _prompt: String,
}

pub struct CleanGarbageTool;

impl CleanGarbageTool {
    pub fn new() -> Self { Self }
}

impl ToolPrompt for CleanGarbageTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 垃圾文件清理完成
向用户报告清理结果：
- 共清理 X 个文件，释放 Y GB 空间
- 如有失败项，说明原因
- 建议用户定期执行垃圾清理
"#
    }
}

impl rig_core::tool::Tool for CleanGarbageTool {
    const NAME: &'static str = "clean_garbage";
    type Error = ToolError;
    type Args = CleanGarbageInput;
    type Output = CleanGarbageOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "根据用户确认的清理策略执行垃圾文件删除操作。支持直接删除或移至回收站。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "files_to_delete": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要直接删除的文件路径列表"
                    },
                    "files_to_trash": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要移至回收站的文件路径列表"
                    },
                    "confirmed": {
                        "type": "boolean",
                        "description": "是否确认执行清理操作（必须为 true 才能执行）"
                    }
                },
                "required": ["files_to_delete", "files_to_trash", "confirmed"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        if !args.confirmed {
            return Err(ToolError::ExecutionFailed("操作未确认，无法执行清理".to_string()));
        }

        let result = tokio::task::spawn_blocking(move || -> Result<CleanGarbageOutput, String> {
            let mut deleted_count = 0;
            let mut trashed_count = 0;
            let mut failed_count = 0;
            let mut total_size_freed = 0u64;
            let mut errors = Vec::new();

            // 删除文件
            for file_path in &args.files_to_delete {
                let path = Path::new(file_path);
                
                // 安全检查：确保路径在用户目录下，防止误删系统文件
                if let Err(e) = is_safe_path(path) {
                    errors.push(format!("安全检查失败，跳过文件: {} ({})", file_path, e));
                    failed_count += 1;
                    continue;
                }

                if !path.exists() {
                    errors.push(format!("文件不存在: {}", file_path));
                    failed_count += 1;
                    continue;
                }

                match std::fs::remove_file(path) {
                    Ok(()) => {
                        if let Ok(metadata) = path.metadata() {
                            total_size_freed += metadata.len();
                        }
                        deleted_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("删除失败 {}: {}", file_path, e));
                        failed_count += 1;
                    }
                }
            }

            // 移至回收站（使用 trash 库）
            for file_path in &args.files_to_trash {
                let path = Path::new(file_path);
                
                // 安全检查
                if let Err(e) = is_safe_path(path) {
                    errors.push(format!("安全检查失败，跳过文件: {} ({})", file_path, e));
                    failed_count += 1;
                    continue;
                }

                if !path.exists() {
                    errors.push(format!("文件不存在: {}", file_path));
                    failed_count += 1;
                    continue;
                }

                match trash::delete(path) {
                    Ok(()) => {
                        if let Ok(metadata) = path.metadata() {
                            total_size_freed += metadata.len();
                        }
                        trashed_count += 1;
                    }
                    Err(e) => {
                        errors.push(format!("移至回收站失败 {}: {}", file_path, e));
                        failed_count += 1;
                    }
                }
            }

            Ok(CleanGarbageOutput {
                success: failed_count == 0,
                deleted_count,
                trashed_count,
                failed_count,
                total_size_freed,
                errors,
                _prompt: String::new(),
            })
        })
        .await
        .map_err(|e| ToolError::ExecutionError(format!("清理操作崩溃: {}", e)))?
        .map_err(ToolError::ExecutionError)?;

        Ok(CleanGarbageOutput {
            _prompt: Self.detailed_prompt().to_string(),
            ..result
        })
    }
}

/// 安全检查：确保路径在用户可控范围内，不删除系统关键路径
fn is_safe_path(path: &Path) -> Result<(), String> {
    let abs_path = path.canonicalize()
        .map_err(|e| format!("无法解析路径: {}", e))?;

    let abs_str = abs_path.to_string_lossy().to_lowercase();

    // 检查是否在系统关键目录中
    let dangerous_paths = [
        "c:\\windows",
        "c:\\program files",
        "c:\\program files (x86)",
        "c:\\programdata",
        "\\system volume information",
        "\\config.msi",
    ];

    for dangerous in &dangerous_paths {
        if abs_str.starts_with(dangerous) {
            return Err(format!("路径位于危险区域，拒绝操作: {}", path.display()));
        }
    }

    Ok(())
}