use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::modules::app_cache::scanner::{start_app_cache_scan, AppCacheScanOptions};
use super::tool_error::ToolError;

#[derive(Debug, Deserialize)]
pub struct AppCacheToolInput {
    #[serde(default)]
    pub apps: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub incremental: bool,
    #[serde(default)]
    pub force_rescan: bool,
}

#[derive(Debug, Serialize)]
pub struct AppCacheToolOutput {
    pub scan_id: String,
    pub total_apps: u64,
    pub total_size: u64,
    pub apps: Vec<serde_json::Value>,
}

pub struct AppCacheTool {
    app: AppHandle,
}

impl AppCacheTool {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl Tool for AppCacheTool {
    const NAME: &'static str = "app_cache_scanner";
    type Error = ToolError;
    type Args = AppCacheToolInput;
    type Output = AppCacheToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "扫描应用缓存，识别可清理的缓存文件。返回应用列表及其缓存大小。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "apps": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要扫描的应用名称列表"
                    },
                    "categories": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要扫描的应用分类列表"
                    },
                    "incremental": {
                        "type": "boolean",
                        "description": "是否增量扫描，默认为 false"
                    },
                    "force_rescan": {
                        "type": "boolean",
                        "description": "是否强制重新扫描，默认为 false"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let options = AppCacheScanOptions {
            apps: args.apps,
            categories: args.categories,
            incremental: args.incremental,
            force_rescan: args.force_rescan,
        };

        let result: String = start_app_cache_scan(self.app.clone(), options)
            .await
            .map_err(|e| ToolError::ScanFailed(e.to_string()))?;

        Ok(AppCacheToolOutput {
            scan_id: result,
            total_apps: 0,
            total_size: 0,
            apps: vec![],
        })
    }
}
