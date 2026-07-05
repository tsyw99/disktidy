use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::modules::cleaner::CleanerExecutor;
use crate::models::CleanOptions;
use super::tool_error::ToolError;

#[derive(Debug, Deserialize)]
pub struct CleanerToolInput {
    pub files: Vec<String>,
    #[serde(default = "default_true")]
    pub move_to_recycle_bin: bool,
    #[serde(default)]
    pub secure_delete: bool,
    #[serde(default)]
    pub confirmed: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct CleanerToolOutput {
    pub cleaned_files: u64,
    pub freed_space: u64,
    pub errors: Vec<String>,
}

pub struct CleanerTool {
    #[allow(dead_code)]
    app: AppHandle,
}

impl CleanerTool {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Tool for CleanerTool {
    const NAME: &'static str = "cleaner";
    type Error = ToolError;
    type Args = CleanerToolInput;
    type Output = CleanerToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "清理指定文件。注意：永久删除（move_to_recycle_bin=false）需要用户明确确认（confirmed=true）。建议优先使用移动到回收站。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "files": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要清理的文件路径列表"
                    },
                    "move_to_recycle_bin": {
                        "type": "boolean",
                        "description": "是否移动到回收站（true）或永久删除（false），默认为 true"
                    },
                    "secure_delete": {
                        "type": "boolean",
                        "description": "是否安全删除（多次覆写），默认为 false"
                    },
                    "confirmed": {
                        "type": "boolean",
                        "description": "用户确认标志。永久删除时必须为 true，默认为 false"
                    }
                },
                "required": ["files"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 安全检查：永久删除必须确认
        if !args.move_to_recycle_bin && !args.confirmed {
            return Err(ToolError::RequiresConfirmation);
        }

        let options = CleanOptions {
            move_to_recycle_bin: args.move_to_recycle_bin,
            secure_delete: args.secure_delete,
            secure_pass_count: 3,
        };

        let cleaner = CleanerExecutor::with_options(options);
        let files: Vec<std::path::PathBuf> = args.files.iter().map(|f| std::path::PathBuf::from(f)).collect();
        
        let result = cleaner.clean(files).await.map_err(|e| ToolError::CleanFailed(format!("{:?}", e)))?;

        Ok(CleanerToolOutput {
            cleaned_files: result.cleaned_files,
            freed_space: result.cleaned_size,
            errors: result.errors.iter().map(|e| format!("{}: {} - {}", e.path, e.error_code, e.error_message)).collect(),
        })
    }
}
