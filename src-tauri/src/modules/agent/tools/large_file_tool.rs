use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::modules::large_file_scanner::{start_scan, ScanConfig};
use super::tool_error::ToolError;
use super::ToolPrompt;

#[derive(Debug, Deserialize)]
pub struct LargeFileToolInput {
    pub path: String,
    #[serde(default = "default_min_size")]
    pub min_size_mb: u64,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_system: bool,
}

fn default_min_size() -> u64 {
    100
}

#[derive(Debug, Serialize)]
pub struct LargeFileToolOutput {
    pub scan_id: String,
    pub status: String,
    pub message: String,
    pub _prompt: String,
}

impl ToolPrompt for LargeFileTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 大文件扫描工作流
你正在使用 large_file_scanner 工具分析大文件，请遵循以下流程：

1. 扫描启动后告知用户当前阈值和扫描范围
2. 收到结果后，用表格展示 TOP 大文件：文件名 | 路径 | 大小
3. 分析大文件是否可以安全删除或移动：
   - 安装包（.exe/.msi/.dmg）→ 可删除或备份
   - 视频/大型文档 → 建议移到外置存储
   - 系统文件 → 警告不可删除
4. 对可清理的大文件，建议调用 cleaner 或 file_delete 处理
"#
    }
}

pub struct LargeFileTool {
    app: AppHandle,
}

impl LargeFileTool {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Tool for LargeFileTool {
    const NAME: &'static str = "large_file_scanner";
    type Error = ToolError;
    type Args = LargeFileToolInput;
    type Output = LargeFileToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "扫描指定路径的大文件（默认>=100MB）。返回扫描ID，可通过事件系统获取进度。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要扫描的目录路径"
                    },
                    "min_size_mb": {
                        "type": "integer",
                        "description": "最小文件大小（MB），默认为 100"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "是否包含隐藏文件，默认为 false"
                    },
                    "include_system": {
                        "type": "boolean",
                        "description": "是否包含系统文件，默认为 false"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let config = ScanConfig {
            path: args.path.clone(),
            min_size_bytes: args.min_size_mb * 1024 * 1024,
            exclude_paths: vec![],
            include_hidden: args.include_hidden,
            include_system: args.include_system,
        };

        match start_scan(self.app.clone(), config).await {
            Ok(scan_id) => Ok(LargeFileToolOutput {
                scan_id,
                status: "started".to_string(),
                message: "大文件扫描已启动，请通过事件系统获取进度".to_string(),
                _prompt: self.detailed_prompt().to_string(),
            }),
            Err(e) => Err(ToolError::ScanFailed(e)),
        }
    }
}
