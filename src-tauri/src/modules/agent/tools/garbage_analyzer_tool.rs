use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::modules::file_analyzer::garbage::{GarbageDetector, GarbageDetectorOptions};
use crate::models::cleaner::{RiskLevel};
use super::tool_error::ToolError;
use super::ToolPrompt;

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
    pub categories: Vec<CompressedCategory>,
    pub _prompt: String,
}

#[derive(Debug, Serialize)]
pub struct CompressedCategory {
    pub category: String,
    pub display_name: String,
    pub file_count: u64,
    pub total_size: u64,
    pub safe_to_delete_count: u64,
    pub risky_count: u64,
    pub largest_file_size: Option<u64>,
    pub sample_files: Vec<String>,
}

impl ToolPrompt for GarbageAnalyzerTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 垃圾文件分析工作流
你正在使用 garbage_analyzer 工具分析垃圾文件，请遵循以下流程：

1. 展示分析结果：垃圾文件类别、数量、总大小
2. 用表格展示各类垃圾：类别 | 文件数 | 大小 | 可否安全清理
3. 按安全级别排列建议：
   - 安全清理：系统临时文件、浏览器缓存 → 推荐优先清理
   - 需确认：应用缓存 → 先展示再清理
4. 建议调用 cleaner 工具执行清理
"#
    }
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

        let compressed_categories: Vec<CompressedCategory> = result.categories.values().map(|stats| {
            // Count safe/risky files
            let safe_to_delete_count = stats.files.iter()
                .filter(|f| f.safe_to_delete)
                .count() as u64;
            
            let risky_count = stats.files.iter()
                .filter(|f| f.risk_level >= RiskLevel::High)
                .count() as u64;

            // Find largest file
            let largest_file_size = stats.files.iter()
                .map(|f| f.size)
                .max();

            // Take up to 3 sample files
            let sample_files: Vec<String> = stats.files.iter()
                .take(3)
                .map(|f| {
                    std::path::Path::new(&f.path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&f.path)
                        .to_string()
                })
                .collect();

            CompressedCategory {
                category: stats.category.clone(),
                display_name: match stats.category.as_str() {
                    "systemTemp" => "系统临时文件".to_string(),
                    "browserCache" => "浏览器缓存".to_string(),
                    "appCache" => "应用缓存".to_string(),
                    "recycleBin" => "回收站".to_string(),
                    "logFile" => "日志文件".to_string(),
                    _ => stats.category.clone(),
                },
                file_count: stats.file_count,
                total_size: stats.total_size,
                safe_to_delete_count,
                risky_count,
                largest_file_size,
                sample_files,
            }
        }).collect();

        Ok(GarbageAnalyzerToolOutput {
            scan_id: result.scan_id,
            total_categories: result.categories.len() as u64,
            total_files: result.total_files,
            total_size: result.total_size,
            categories: compressed_categories,
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
