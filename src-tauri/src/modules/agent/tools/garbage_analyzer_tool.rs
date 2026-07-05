use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::modules::file_analyzer::garbage::{GarbageDetector, GarbageDetectorOptions};
use super::tool_error::ToolError;

#[derive(Debug, Deserialize)]
pub struct GarbageAnalyzerToolInput {
    #[serde(default)]
    pub include_system_temp: bool,
    #[serde(default)]
    pub include_browser_cache: bool,
    #[serde(default)]
    pub include_app_cache: bool,
}

#[derive(Debug, Serialize)]
pub struct GarbageAnalyzerToolOutput {
    pub scan_id: String,
    pub total_categories: u64,
    pub total_files: u64,
    pub total_size: u64,
    pub categories: Vec<serde_json::Value>,
}

pub struct GarbageAnalyzerTool;

impl GarbageAnalyzerTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for GarbageAnalyzerTool {
    const NAME: &'static str = "garbage_analyzer";
    type Error = ToolError;
    type Args = GarbageAnalyzerToolInput;
    type Output = GarbageAnalyzerToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "分析系统垃圾文件（临时文件、浏览器缓存、应用缓存等）。返回垃圾文件分类统计。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "include_system_temp": {
                        "type": "boolean",
                        "description": "是否包含系统临时文件，默认为 false"
                    },
                    "include_browser_cache": {
                        "type": "boolean",
                        "description": "是否包含浏览器缓存，默认为 false"
                    },
                    "include_app_cache": {
                        "type": "boolean",
                        "description": "是否包含应用缓存，默认为 false"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let options = GarbageDetectorOptions {
            include_system_temp: args.include_system_temp,
            include_browser_cache: args.include_browser_cache,
            include_app_cache: args.include_app_cache,
            ..Default::default()
        };

        let detector = GarbageDetector::with_options(options);
        let result = detector.detect_all();

        Ok(GarbageAnalyzerToolOutput {
            scan_id: result.scan_id,
            total_categories: result.categories.len() as u64,
            total_files: result.total_files,
            total_size: result.total_size,
            categories: result.categories.iter().map(|c| serde_json::to_value(c).unwrap_or_default()).collect(),
        })
    }
}
