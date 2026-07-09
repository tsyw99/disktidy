use rig_core::tool::Tool;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::modules::file_content_analyzer::{
    FileContentReader, ContentAnalyzer, ReportGenerator,
    FileFormat, ExcelSheet,
};
use super::tool_error::ToolError;
use super::ToolPrompt;

#[derive(Debug, Deserialize)]
pub struct FileContentToolInput {
    pub paths: Vec<String>,
    #[serde(default)]
    pub generate_report: bool,
    /// 可选：HTML报告的保存路径（绝对路径，含文件名）
    /// 如 C:\Users\xxx\Downloads\20260709_分析报告.html
    /// 设置后将自动保存HTML到该路径，无需单独调用 file_write
    #[serde(default)]
    pub output_path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileContentToolOutput {
    pub report_id: String,
    pub files_analyzed: usize,
    pub total_size_bytes: u64,
    pub analyses: Vec<ContentAnalysisSummary>,
    /// HTML报告内容（仅当 output_path 未设置时返回）
    pub html_report: Option<String>,
    /// HTML报告保存路径（仅当 output_path 设置且保存成功时）
    pub saved_path: Option<String>,
    /// 工具专属提示词
    pub _prompt: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ContentAnalysisSummary {
    pub file_name: String,
    pub file_format: String,
    pub file_size_bytes: u64,
    pub total_chars: usize,
    pub total_lines: usize,
    pub total_words: usize,
    pub reading_time_minutes: f64,
    pub language_hint: String,
    pub top_keywords: Vec<String>,
    pub structure_count: usize,
}

impl ToolPrompt for FileContentTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 文件内容分析工作流

### Excel 文件（.xlsx/.xls）
1. 调用 `resolve_path` 传入 "下载" 获取下载目录真实路径
2. 调用 `file_content_analyzer`：
   - `paths`: 所有 Excel 文件路径
   - `generate_report`: true
   - `output_path`: `{下载目录}/{YYYYMMDD}_{简短描述}_分析报告.html`
3. 报告自动保存后，向用户汇总关键发现：总金额、总记录数、Top供应商、月度趋势要点

### 其他文件（TXT/PDF/DOCX/MD）
同上流程，`output_path` 保存即可。

**要点**：报告通过 `output_path` 自动保存，**禁止**再调用 `file_write` 写入。
**禁止**：不要对用户说"我来帮您生成"等客套话，直接执行。
"#
    }
}

pub struct FileContentTool;

impl FileContentTool {
    pub fn new() -> Self {
        Self
    }
}

impl Tool for FileContentTool {
    const NAME: &'static str = "file_content_analyzer";
    type Error = ToolError;
    type Args = FileContentToolInput;
    type Output = FileContentToolOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "分析文件内容，提取关键词、统计信息、内容结构。支持TXT、PDF、DOCX(Word)、XLSX/XLS(Excel)、MD、JSON、XML、CSV等格式。可通过output_path参数直接保存HTML可视化报告到指定路径。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "paths": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "要分析的文件路径列表"
                    },
                    "generate_report": {
                        "type": "boolean",
                        "description": "是否生成HTML可视化报告，默认为false"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "可选：HTML报告的保存路径（绝对路径，含.html文件名）。设置后HTML会自动写入该路径。"
                    }
                },
                "required": ["paths"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut analyses = Vec::new();
        let mut read_errors = Vec::new();
        let mut all_sheets: Vec<ExcelSheet> = Vec::new();
        let mut has_excel = false;

        for path in &args.paths {
            match FileContentReader::read(path) {
                Ok(content) => {
                    let is_excel = matches!(content.format, FileFormat::Xlsx | FileFormat::Xls);
                    if is_excel {
                        has_excel = true;
                        let sheets = ExcelSheet::parse_from_content(&content.content);
                        all_sheets.extend(sheets);
                    }
                    let analysis = ContentAnalyzer::analyze(&content);
                    analyses.push(analysis);
                }
                Err(e) => {
                    read_errors.push(format!("{}: {}", path, e));
                }
            }
        }

        let files_analyzed = analyses.len();
        let total_size_bytes: u64 = analyses.iter().map(|a| a.file_size_bytes).sum();
        let report_id = uuid::Uuid::new_v4().to_string();

        let summaries: Vec<ContentAnalysisSummary> = analyses.iter().map(|a| {
            ContentAnalysisSummary {
                file_name: a.file_name.clone(),
                file_format: a.file_format.clone(),
                file_size_bytes: a.file_size_bytes,
                total_chars: a.total_chars,
                total_lines: a.total_lines,
                total_words: a.total_words,
                reading_time_minutes: a.reading_time_minutes,
                language_hint: a.language_hint.clone(),
                top_keywords: a.top_keywords.iter().map(|k| k.word.clone()).collect(),
                structure_count: a.structure.len(),
            }
        }).collect();

        let mut html_report: Option<String> = None;
        let mut saved_path: Option<String> = None;

        if args.generate_report && !analyses.is_empty() {
            let report = if has_excel && !all_sheets.is_empty() {
                // Excel 文件：使用 ECharts 深度分析报告
                ReportGenerator::generate_excel_report(analyses, &all_sheets)
            } else {
                // 非 Excel 文件：使用通用报告
                ReportGenerator::generate(analyses)
            };

            if let Some(output_path) = &args.output_path {
                // 直接保存到指定路径
                let path = Path::new(output_path);

                // 确保父目录存在
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent).map_err(|e| {
                            ToolError::ExecutionError(format!("创建目录失败: {}", e))
                        })?;
                    }
                }

                std::fs::write(path, &report.html_report).map_err(|e| {
                    ToolError::ExecutionError(format!("保存HTML报告失败: {}", e))
                })?;

                let canonical = std::fs::canonicalize(path)
                    .unwrap_or_else(|_| path.to_path_buf());
                saved_path = Some(canonical.to_string_lossy().to_string());

                // 不返回html_report到LLM（太大了），只返回保存路径
            } else {
                // 没有 output_path，返回HTML内容到工具结果中（用于流式展示）
                html_report = Some(report.html_report);
            }
        }

        Ok(FileContentToolOutput {
            report_id,
            files_analyzed,
            total_size_bytes,
            analyses: summaries,
            html_report,
            saved_path,
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}
