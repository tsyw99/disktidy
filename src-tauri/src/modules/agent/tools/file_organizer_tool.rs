use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::modules::file_organizer::{
    FileOrganizerScanner, ContentClassifier, FileOrganizer,
};
use super::tool_error::ToolError;
use super::ToolPrompt;

#[derive(Debug, Deserialize)]
pub struct FileOrganizerToolInput {
    pub path: String,
    pub action: String, // "scan" | "preview" | "execute"
    #[serde(default)]
    pub include_hidden: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct FileOrganizerToolOutput {
    pub action: String,
    pub path: String,
    pub scan_summary: Option<ScanSummary>,
    pub preview: Option<PreviewSummary>,
    pub result: Option<ExecuteSummary>,
    /// 工具专属提示词
    pub _prompt: String,
}

#[derive(Debug, Serialize)]
pub struct ScanSummary {
    pub total_files: u64,
    pub total_size_bytes: u64,
    pub scan_duration_ms: u64,
    pub categories_found: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PreviewSummary {
    pub preview_id: String,
    pub total_operations: usize,
    pub files_to_move: usize,
    pub files_to_copy: usize,
    pub files_to_skip: usize,
    pub categories: Vec<CategoryInfo>,
}

#[derive(Debug, Serialize)]
pub struct CategoryInfo {
    pub name: String,
    pub file_count: usize,
    pub total_size_bytes: u64,
}

#[derive(Debug, Serialize)]
pub struct ExecuteSummary {
    pub moved_count: usize,
    pub copied_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

impl ToolPrompt for FileOrganizerTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 文件智能整理工作流
你正在使用 file_organizer 工具整理文件，请严格遵循以下流程：

1. 如果用户提到非绝对路径（如"桌面"、"文档"），先用 resolve_path 获取真实路径
2. 调用 file_organizer 设置 action="scan" 扫描目录
3. 向用户展示扫描结果（文件数量、大小分布）
4. 调用 file_organizer 设置 action="preview" 生成整理预览方案
5. 向用户展示预览方案（分类规则、将移动/复制的文件数、目标目录结构）
6. **必须等待用户确认**后才能执行
7. 用户确认后，调用 file_organizer 设置 action="execute" 和 dry_run=false
8. 报告整理结果
"#
    }
}

pub struct FileOrganizerTool;

impl FileOrganizerTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for FileOrganizerTool {
    const NAME: &'static str = "file_organizer";
    type Error = ToolError;
    type Args = FileOrganizerToolInput;
    type Output = FileOrganizerToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "智能整理文件：扫描目录、按类型/内容自动分类、预览整理方案、执行文件移动。支持自定义分类规则。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "要整理的目录路径"
                    },
                    "action": {
                        "type": "string",
                        "enum": ["scan", "preview", "execute"],
                        "description": "操作类型: scan(扫描), preview(预览整理方案), execute(执行整理)"
                    },
                    "include_hidden": {
                        "type": "boolean",
                        "description": "是否包含隐藏文件，默认为false"
                    },
                    "dry_run": {
                        "type": "boolean",
                        "description": "是否为演习模式（不实际移动文件），默认为false"
                    },
                    "max_depth": {
                        "type": "integer",
                        "description": "最大扫描深度"
                    }
                },
                "required": ["path", "action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.action.as_str() {
            "scan" => self.do_scan(&args),
            "preview" => self.do_preview(&args),
            "execute" => self.do_execute(&args),
            _ => Err(ToolError::ExecutionFailed(format!("未知操作: {}", args.action))),
        }
    }
}

impl FileOrganizerTool {
    fn do_scan(&self, args: &FileOrganizerToolInput) -> Result<FileOrganizerToolOutput, ToolError> {
        let result = FileOrganizerScanner::scan(
            &args.path,
            args.include_hidden,
            args.max_depth,
            None,
        );

        // 统计扩展名种类
        let extensions: std::collections::HashSet<String> = result.files.iter()
            .map(|f| f.extension.clone())
            .collect();
        let mut categories_found: Vec<String> = extensions.into_iter().collect();
        categories_found.sort();

        Ok(FileOrganizerToolOutput {
            action: "scan".to_string(),
            path: args.path.clone(),
            scan_summary: Some(ScanSummary {
                total_files: result.total_files,
                total_size_bytes: result.total_size_bytes,
                scan_duration_ms: result.scan_duration_ms,
                categories_found,
            }),
            preview: None,
            result: None,
            _prompt: self.detailed_prompt().to_string(),
        })
    }

    fn do_preview(&self, args: &FileOrganizerToolInput) -> Result<FileOrganizerToolOutput, ToolError> {
        // 扫描
        let scan_result = FileOrganizerScanner::scan(
            &args.path,
            args.include_hidden,
            args.max_depth,
            None,
        );

        // 使用默认规则分类
        let rules = ContentClassifier::default_rules();
        let classifier = ContentClassifier::new(rules);
        let classification = classifier.classify(&scan_result.files);

        // 生成预览
        let organizer = FileOrganizer::new(&args.path, vec![]);
        let preview = organizer.preview(&scan_result, &classification);

        let categories: Vec<CategoryInfo> = preview.categories.iter().map(|(name, files)| {
            let total_size = preview.operations.iter()
                .filter(|op| op.category == *name)
                .map(|op| op.file_size_bytes)
                .sum();
            CategoryInfo {
                name: name.clone(),
                file_count: files.len(),
                total_size_bytes: total_size,
            }
        }).collect();

        Ok(FileOrganizerToolOutput {
            action: "preview".to_string(),
            path: args.path.clone(),
            scan_summary: Some(ScanSummary {
                total_files: scan_result.total_files,
                total_size_bytes: scan_result.total_size_bytes,
                scan_duration_ms: scan_result.scan_duration_ms,
                categories_found: categories.iter().map(|c| c.name.clone()).collect(),
            }),
            preview: Some(PreviewSummary {
                preview_id: preview.preview_id,
                total_operations: preview.total_files,
                files_to_move: preview.summary_stats.files_to_move,
                files_to_copy: preview.summary_stats.files_to_copy,
                files_to_skip: preview.summary_stats.files_to_skip,
                categories,
            }),
            result: None,
            _prompt: self.detailed_prompt().to_string(),
        })
    }

    fn do_execute(&self, args: &FileOrganizerToolInput) -> Result<FileOrganizerToolOutput, ToolError> {
        // 扫描
        let scan_result = FileOrganizerScanner::scan(
            &args.path,
            args.include_hidden,
            args.max_depth,
            None,
        );

        // 分类
        let rules = ContentClassifier::default_rules();
        let classifier = ContentClassifier::new(rules);
        let classification = classifier.classify(&scan_result.files);

        // 预览
        let organizer = FileOrganizer::new(&args.path, vec![]);
        let preview = organizer.preview(&scan_result, &classification);

        // 执行
        let result = organizer.execute(&preview, None, args.dry_run);

        Ok(FileOrganizerToolOutput {
            action: "execute".to_string(),
            path: args.path.clone(),
            scan_summary: Some(ScanSummary {
                total_files: scan_result.total_files,
                total_size_bytes: scan_result.total_size_bytes,
                scan_duration_ms: scan_result.scan_duration_ms,
                categories_found: preview.categories.keys().cloned().collect(),
            }),
            preview: None,
            result: Some(ExecuteSummary {
                moved_count: result.moved_count,
                copied_count: result.copied_count,
                skipped_count: result.skipped_count,
                failed_count: result.failed_count,
                errors: result.errors,
            }),
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
