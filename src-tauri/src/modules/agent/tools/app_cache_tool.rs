use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::modules::app_cache::scanner::{start_app_cache_scan, AppCacheScanOptions};
use super::tool_error::ToolError;
use super::ToolPrompt;

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
    /// 工具专属提示词，在工具被调用时随结果一起返回给 LLM
    pub _prompt: String,
}

impl ToolPrompt for AppCacheTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 应用缓存清理工作流
你正在使用 app_cache_scanner 工具进行应用缓存清理，请严格遵循以下流程：

1. 分析扫描结果，按缓存大小降序排列
2. 区分安全级别：
   - 可安全清理：浏览器缓存、临时文件、日志缓存
   - 需谨慎清理：用户数据、应用配置、聊天记录
3. 用表格展示：应用名 | 缓存大小 | 安全级别 | 建议操作
4. 推荐使用 cleaner 工具执行清理（确保 confirmed=true）
5. 清理完成后报告释放空间大小
"#
    }
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
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
