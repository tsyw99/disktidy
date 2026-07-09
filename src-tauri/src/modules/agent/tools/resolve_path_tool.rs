use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::utils::path::SystemPaths;
use super::tool_error::ToolError;
use super::ToolPrompt;

#[derive(Debug, Deserialize)]
pub struct ResolvePathInput {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct ResolvePathOutput {
    pub original: String,
    pub resolved_path: Option<String>,
    pub is_alias: bool,
    pub desktop_path: Option<String>,
    pub desktop_candidates: Vec<String>,
    /// 工具专属提示词（此工具为纯工具型，无需额外流程提示）
    pub _prompt: String,
}

impl ToolPrompt for ResolvePathTool {
    fn detailed_prompt(&self) -> &'static str {
        // resolve_path 是纯工具，无需额外流程提示词
        ""
    }
}

pub struct ResolvePathTool;

impl ResolvePathTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for ResolvePathTool {
    const NAME: &'static str = "resolve_path";
    type Error = ToolError;
    type Args = ResolvePathInput;
    type Output = ResolvePathOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "智能解析路径别名（如桌面/desktop、文档/documents、下载/downloads等）为绝对路径。支持OneDrive Desktop等自定义桌面路径。当用户提到非绝对路径时，先调用此工具获取真实路径。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要解析的路径，可以是别名（如desktop、桌面、documents、文档、downloads、下载）或相对路径"
                    }
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let original = args.path.clone();
        let is_alias = !args.path.contains(':') && !args.path.contains('/') && !args.path.contains('\\');

        let resolved_path = SystemPaths::resolve_path(&args.path)
            .map(|p| p.to_string_lossy().to_string());

        let desktop_path = SystemPaths::desktop_dir()
            .map(|p| p.to_string_lossy().to_string());

        let desktop_candidates: Vec<String> = SystemPaths::get_desktop_candidates()
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

        Ok(ResolvePathOutput {
            original,
            resolved_path,
            is_alias,
            desktop_path,
            desktop_candidates,
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
