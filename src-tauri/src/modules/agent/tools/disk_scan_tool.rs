use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::modules::disk_scan;
use crate::models::ScanOptions;
use super::tool_error::ToolError;

#[derive(Debug, Deserialize)]
pub struct DiskScanToolInput {
    pub path: String,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_system: bool,
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DiskScanToolOutput {
    pub scan_id: String,
    pub status: String,
    pub message: String,
}

pub struct DiskScanTool {
    app: AppHandle,
}

impl DiskScanTool {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Tool for DiskScanTool {
    const NAME: &'static str = "disk_scan";
    type Error = ToolError;
    type Args = DiskScanToolInput;
    type Output = DiskScanToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "扫描指定磁盘路径，分析文件分布和磁盘使用情况。返回扫描ID，可通过事件系统获取扫描进度。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要扫描的磁盘路径，例如 'C:\\\\' 或 'D:\\\\'"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "是否包含隐藏文件，默认为 false"
                    },
                    "include_system": {
                        "type": "boolean",
                        "description": "是否包含系统文件，默认为 false"
                    },
                    "exclude_paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "排除的路径列表，默认为空"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let options = ScanOptions {
            paths: vec![args.path.clone()],
            mode: "full".to_string(),
            include_hidden: args.include_hidden,
            include_system: args.include_system,
            exclude_paths: args.exclude_paths,
        };

        match disk_scan::start_scan(self.app.clone(), options).await {
            Ok(scan_id) => Ok(DiskScanToolOutput {
                scan_id,
                status: "started".to_string(),
                message: "扫描已启动，请通过事件系统获取进度".to_string(),
            }),
            Err(e) => Err(ToolError::ScanFailed(e)),
        }
    }
}
