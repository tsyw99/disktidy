use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::tool_error::ToolError;
use super::ToolPrompt;

#[derive(Debug, Deserialize)]
pub struct FileWriteToolInput {
    /// 写入内容
    pub content: String,
    /// 写入路径（绝对路径，含文件名）
    pub path: String,
    /// 如果文件已存在是否覆盖
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Serialize)]
pub struct FileWriteToolOutput {
    pub success: bool,
    pub file_path: String,
    pub file_size_bytes: u64,
    pub message: String,
    /// 工具专属提示词
    pub _prompt: String,
}

impl ToolPrompt for FileWriteTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 文件写入使用场景
你正在使用 file_write 工具写入文件，请注意：

- 适用于用户明确要求"导出"、"保存为文件"等场景
- 支持将文本/Markdown/CSV/JSON/HTML 内容写入到指定路径
- 会自动创建不存在的父目录
- 注：HTML 分析报告应通过 file_content_analyzer 的 output_path 参数直接生成，不需要单独使用 file_write
"#
    }
}

pub struct FileWriteTool;

impl Tool for FileWriteTool {
    const NAME: &'static str = "file_write";

    type Error = ToolError;
    type Args = FileWriteToolInput;
    type Output = FileWriteToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "写入内容到指定路径的文件。支持文本文件(txt/md/csv/json/xml)和HTML文件。会自动创建不存在的父目录。用于保存分析报告、导出数据、写入配置文件等场景。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "要写入文件的完整内容"
                    },
                    "path": {
                        "type": "string",
                        "description": "目标文件绝对路径（含文件名），如C:\\Users\\xxx\\Downloads\\报告.html"
                    },
                    "overwrite": {
                        "type": "boolean",
                        "description": "如果文件已存在是否覆盖，默认false"
                    }
                },
                "required": ["content", "path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);

        // 安全检查：拒绝写入到系统关键目录
        let path_str = args.path.to_lowercase();
        let blocked_prefixes = [
            "c:\\windows",
            "c:\\windows\\system32",
            "/system",
            "/etc",
            "/boot",
            "/sys",
        ];
        for prefix in &blocked_prefixes {
            if path_str.starts_with(prefix) {
                return Err(ToolError::PermissionDenied(format!(
                    "无权写入系统目录: {}",
                    &args.path
                )));
            }
        }

        // 检查文件是否已存在
        if path.exists() && !args.overwrite {
            return Err(ToolError::AlreadyExists(format!(
                "文件已存在: {}，如需覆盖请设置 overwrite=true",
                &args.path
            )));
        }

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    ToolError::ExecutionError(format!("创建目录失败: {}", e))
                })?;
            }
        }

        // 写入文件
        let content_bytes = args.content.as_bytes();
        std::fs::write(path, content_bytes).map_err(|e| {
            ToolError::ExecutionError(format!("写入文件失败: {}", e))
        })?;

        let size = content_bytes.len() as u64;
        let canonical = std::fs::canonicalize(path)
            .unwrap_or_else(|_| path.to_path_buf());

        Ok(FileWriteToolOutput {
            success: true,
            file_path: canonical.to_string_lossy().to_string(),
            file_size_bytes: size,
            message: format!(
                "文件已成功写入: {} ({} bytes)",
                canonical.display(),
                size
            ),
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
