use calamine::{open_workbook_auto, Reader, Data};
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::modules::agent::tools::tool_error::ToolError;
use super::ToolPrompt;
use super::excel_cache::{self, CachedExcelData};

#[derive(Debug, Deserialize)]
pub struct ReadExcelInput {
    pub directory: String,
}

#[derive(Debug, Serialize)]
pub struct ReadExcelOutput {
    pub file_count: usize,
    pub file_names: Vec<String>,
    pub row_counts: Vec<usize>,
    pub total_rows: usize,
    pub columns: Vec<String>,
    pub column_count: usize,
    pub _prompt: String,
}

impl ToolPrompt for ReadExcelTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## Excel 数据读取完成
下一步调用 `analyze_data` 对数据进行多维度分析。
可用维度：`monthly_trend` `supplier_ranking` `dept_distribution` `buyer_performance` `top_products` `note_analysis` `quantity_analysis`
根据列名选择相关维度传入。
"#
    }
}

/// 内部返回结构（含缓存数据），不暴露给 LLM
struct ReadResult {
    file_count: usize,
    file_names: Vec<String>,
    row_counts: Vec<usize>,
    columns: Vec<String>,
    all_rows: Vec<HashMap<String, String>>,
}

pub struct ReadExcelTool;

impl ReadExcelTool {
    pub fn new() -> Self { Self }

    fn is_title_row(cells: &[String]) -> bool {
        if cells.is_empty() { return false; }
        let non_empty_count = cells.iter().filter(|c| !c.trim().is_empty()).count();
        non_empty_count <= 2 && !cells[0].trim().is_empty()
    }
}

impl rig_core::tool::Tool for ReadExcelTool {
    const NAME: &'static str = "read_excel";
    type Error = ToolError;
    type Args = ReadExcelInput;
    type Output = ReadExcelOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "读取指定目录下所有 .xlsx/.xls 文件，返回列名清单、文件数和总行数。不返回原始数据，仅供确认数据结构。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "包含 Excel 文件的目录路径"
                    }
                },
                "required": ["directory"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let directory = args.directory.clone();
        let dir_for_cache = directory.clone();

        // 阻塞 I/O 放入 spawn_blocking，返回内部结构（不含缓存数据在 Output 中）
        let result: ReadResult = tokio::task::spawn_blocking(move || -> Result<ReadResult, String> {
            let dir = Path::new(&directory);
            if !dir.is_dir() {
                return Err(format!("目录不存在: {}", directory));
            }

            let mut excel_files: Vec<_> = std::fs::read_dir(dir)
                .map_err(|e| format!("读取目录失败: {}", e))?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|e| e.to_str())
                        .map(|ext| matches!(ext.to_lowercase().as_str(), "xlsx" | "xls"))
                        .unwrap_or(false)
                })
                .collect();
            excel_files.sort();

            let mut file_names = Vec::new();
            let mut row_counts = Vec::new();
            let mut all_columns: Vec<String> = Vec::new();
            let mut all_rows: Vec<HashMap<String, String>> = Vec::new();

            for file_path in &excel_files {
                let file_name = file_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let mut wb = match open_workbook_auto(file_path) {
                    Ok(w) => w,
                    Err(e) => {
                        log::warn!("跳过无法打开的文件 {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let sheet_names = wb.sheet_names().to_vec();
                let mut file_rows = 0usize;

                for sheet_name in &sheet_names {
                    let range = match wb.worksheet_range(sheet_name) {
                        Ok(r) => r,
                        Err(_) => continue,
                    };
                    let rows_iter = range.rows();
                    let mut header_found = false;
                    let mut columns: Vec<String> = Vec::new();

                    for row in rows_iter {
                        let cells: Vec<String> = row.iter().map(|c| match c {
                            Data::String(s) => s.clone(),
                            Data::Float(f) => {
                                if *f == f.floor() && f.is_finite() && *f < 1e15 {
                                    format!("{}", *f as i64)
                                } else {
                                    format!("{}", *f)
                                }
                            }
                            Data::Int(i) => format!("{}", i),
                            Data::Bool(b) => format!("{}", b),
                            Data::DateTime(d) => format!("{:?}", d),
                            Data::DateTimeIso(d) => d.clone(),
                            Data::DurationIso(d) => d.clone(),
                            Data::Empty => String::new(),
                            Data::Error(e) => format!("[ERR:{:?}]", e),
                        }).collect();

                        if cells.iter().all(|c| c.trim().is_empty()) {
                            continue;
                        }
                        if !header_found {
                            if Self::is_title_row(&cells) {
                                continue;
                            }
                            columns = cells.iter().map(|c| c.trim().to_string()).collect();
                            if all_columns.is_empty() {
                                all_columns = columns.clone();
                            }
                            header_found = true;
                            continue;
                        }
                        let mut row_map = HashMap::new();
                        for (i, col) in columns.iter().enumerate() {
                            let val = cells.get(i).map(|c| c.trim().to_string()).unwrap_or_default();
                            row_map.insert(col.clone(), val);
                        }
                        if row_map.values().any(|v: &String| !v.is_empty()) {
                            all_rows.push(row_map);
                            file_rows += 1;
                        }
                    }
                }
                file_names.push(file_name);
                row_counts.push(file_rows);
            }

            Ok(ReadResult {
                file_count: excel_files.len(),
                file_names,
                row_counts,
                columns: all_columns,
                all_rows,
            })
        })
        .await
        .map_err(|e| ToolError::ExecutionError(format!("读取线程崩溃: {}", e)))?
        .map_err(ToolError::ExecutionError)?;

        let total_rows = result.all_rows.len();

        // 存入缓存（原始数据仅在此处使用，不暴露给 LLM）
        excel_cache::cache_put(&dir_for_cache, CachedExcelData {
            directory: dir_for_cache.clone(),
            file_names: result.file_names.clone(),
            row_counts: result.row_counts.clone(),
            columns: result.columns.clone(),
            rows: result.all_rows,
        });

        debug!(
            "read_excel: {} 文件, {} 行, {} 列",
            result.file_count, total_rows, result.columns.len()
        );

        Ok(ReadExcelOutput {
            file_count: result.file_count,
            file_names: result.file_names,
            row_counts: result.row_counts,
            total_rows,
            column_count: result.columns.len(),
            columns: result.columns,
            _prompt: Self.detailed_prompt().to_string(),
        })
    }
}
