use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::modules::cancellable_file_classifier::{CancellableFileClassifier, FileClassificationOptions, FileTypeStats, FileBriefInfo};
use super::tool_error::ToolError;

#[derive(Debug, Deserialize)]
pub struct FileClassifierToolInput {
    pub path: String,
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub include_system: bool,
}

#[derive(Debug, Serialize)]
pub struct FileClassifierToolOutput {
    pub scan_id: String,
    pub path: String,
    pub total_files: u64,
    pub total_size: u64,
    pub total_folders: u64,
    pub categories: Vec<FileTypeStats>,
    pub largest_files: Vec<FileBriefInfo>,
}

pub struct FileClassifierTool;

impl FileClassifierTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for FileClassifierTool {
    const NAME: &'static str = "file_classifier";
    type Error = ToolError;
    type Args = FileClassifierToolInput;
    type Output = FileClassifierToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "按类型分类文件，分析磁盘空间使用情况。返回文件分类统计、大文件列表等信息。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要分类的目录路径"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "最大扫描深度，默认为无限制"
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
        let options = FileClassificationOptions {
            max_depth: args.max_depth,
            include_hidden: args.include_hidden,
            include_system: args.include_system,
            ..Default::default()
        };

        let classifier = CancellableFileClassifier::with_options(options);
        let result = classifier.classify(&args.path);

        Ok(FileClassifierToolOutput {
            scan_id: result.scan_id,
            path: result.path,
            total_files: result.total_files,
            total_size: result.total_size,
            total_folders: result.total_folders,
            categories: result.categories,
            largest_files: result.largest_files,
        })
    }
}
