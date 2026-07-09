use std::path::Path;
use std::fs;
use std::io::Read;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 支持的文件格式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileFormat {
    PlainText,
    Markdown,
    Pdf,
    Docx,
    Xlsx,
    Xls,
    Json,
    Xml,
    Csv,
    Unknown,
}

impl std::fmt::Display for FileFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileFormat::PlainText => write!(f, "纯文本"),
            FileFormat::Markdown => write!(f, "Markdown"),
            FileFormat::Pdf => write!(f, "PDF"),
            FileFormat::Docx => write!(f, "Word文档"),
            FileFormat::Xlsx => write!(f, "Excel工作簿(.xlsx)"),
            FileFormat::Xls => write!(f, "Excel工作簿(.xls)"),
            FileFormat::Json => write!(f, "JSON"),
            FileFormat::Xml => write!(f, "XML"),
            FileFormat::Csv => write!(f, "CSV"),
            FileFormat::Unknown => write!(f, "未知"),
        }
    }
}

/// 文件内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub name: String,
    pub format: FileFormat,
    pub content: String,
    pub size_bytes: u64,
    pub line_count: usize,
    pub char_count: usize,
}

/// Excel 结构化数据：一张工作表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelSheet {
    pub name: String,
    pub headers: Vec<String>,
    /// 每行是一个 HashMap<列名, 单元格值>
    pub rows: Vec<HashMap<String, String>>,
}

impl ExcelSheet {
    /// 从 TSV 格式的 content 中解析出结构化数据
    /// content 格式为 "# 工作表: SheetName\nheader1\theader2\t...\ndata1\tdata2\t..."
    pub fn parse_from_content(content: &str) -> Vec<Self> {
        let mut sheets = Vec::new();
        let mut current_name = String::from("Sheet1");
        let mut current_headers: Vec<String> = Vec::new();
        let mut current_rows: Vec<HashMap<String, String>> = Vec::new();
        let mut in_header = false;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // 检测工作表分隔或标题
            if line.starts_with("# 工作表:") {
                if !current_headers.is_empty() {
                    sheets.push(ExcelSheet {
                        name: std::mem::take(&mut current_name),
                        headers: std::mem::take(&mut current_headers),
                        rows: std::mem::take(&mut current_rows),
                    });
                }
                current_name = line.trim_start_matches("# 工作表:").trim().to_string();
                current_headers.clear();
                current_rows.clear();
                in_header = true;
                continue;
            }

            // 检测分隔线
            if line == "---" {
                continue;
            }

            // 检测截断提示
            if line.starts_with("... (共 ") {
                continue;
            }

            let cells: Vec<&str> = line.split('\t').collect();

            if in_header {
                current_headers = cells.iter().map(|c| c.trim().to_string()).collect();
                in_header = false;
            } else if !current_headers.is_empty() {
                let mut row = HashMap::new();
                for (i, header) in current_headers.iter().enumerate() {
                    let value = cells.get(i).map(|c| c.trim().to_string()).unwrap_or_default();
                    row.insert(header.clone(), value);
                }
                // 跳过全空行
                if row.values().any(|v| !v.is_empty()) {
                    current_rows.push(row);
                }
            }
        }

        // 保存最后一张工作表
        if !current_headers.is_empty() {
            sheets.push(ExcelSheet {
                name: current_name,
                headers: current_headers,
                rows: current_rows,
            });
        }

        sheets
    }
}

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("文件不存在: {0}")]
    NotFound(String),
    #[error("文件读取失败: {0}")]
    ReadError(String),
    #[error("PDF解析失败: {0}")]
    PdfParseError(String),
    #[error("DOCX解析失败: {0}")]
    DocxParseError(String),
    #[error("Excel解析失败: {0}")]
    ExcelParseError(String),
    #[error("不支持的文件格式: {0}")]
    UnsupportedFormat(String),
}

pub struct FileContentReader;

impl FileContentReader {
    /// 根据文件扩展名判断格式
    fn detect_format(path: &Path) -> FileFormat {
        match path.extension().and_then(|e| e.to_str()).map(|s| s.to_lowercase()) {
            Some(ref ext) => match ext.as_str() {
                "txt" | "log" | "ini" | "cfg" | "conf" | "yaml" | "yml" | "toml" | "env" => FileFormat::PlainText,
                "md" | "markdown" => FileFormat::Markdown,
                "pdf" => FileFormat::Pdf,
                "docx" => FileFormat::Docx,
                "xlsx" => FileFormat::Xlsx,
                "xls" => FileFormat::Xls,
                "json" => FileFormat::Json,
                "xml" | "html" | "htm" | "svg" => FileFormat::Xml,
                "csv" => FileFormat::Csv,
                _ => {
                    // 尝试检测常见代码文件
                    match ext.as_str() {
                        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "java"
                        | "c" | "cpp" | "h" | "hpp" | "cs" | "go" | "rb"
                        | "php" | "swift" | "kt" | "scala" | "vue" | "css"
                        | "scss" | "less" | "sql" | "sh" | "bat" | "ps1"
                        | "lua" | "r" | "dart" | "ex" | "exs" | "elm"
                        | "hs" | "clj" | "edn" | "erl" | "hrl" | "fs" | "fsx" => FileFormat::PlainText,
                        _ => FileFormat::Unknown,
                    }
                }
            },
            None => FileFormat::Unknown,
        }
    }

    /// 读取文件内容
    pub fn read(path: &str) -> Result<FileContent, ReaderError> {
        let path = Path::new(path);
        if !path.exists() {
            return Err(ReaderError::NotFound(path.to_string_lossy().to_string()));
        }

        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        let format = Self::detect_format(path);
        let metadata = fs::metadata(path)
            .map_err(|e| ReaderError::ReadError(format!("获取文件元数据失败: {}", e)))?;
        let size_bytes = metadata.len();

        let content = match &format {
            FileFormat::PlainText | FileFormat::Markdown | FileFormat::Json
            | FileFormat::Xml | FileFormat::Csv => {
                fs::read_to_string(path)
                    .map_err(|e| ReaderError::ReadError(format!("读取文本文件失败: {}", e)))?
            }
            FileFormat::Pdf => {
                Self::read_pdf(path)?
            }
            FileFormat::Docx => {
                Self::read_docx(path)?
            }
            FileFormat::Xlsx | FileFormat::Xls => {
                Self::read_excel(path)?
            }
            FileFormat::Unknown => {
                // 尝试作为纯文本读取（可能是无扩展名或有内容的结构化文件）
                // 如果读取内容中包含空字符（二进制），尝试作为 Excel 解析
                match fs::read_to_string(path) {
                    Ok(text) => text,
                    Err(_) => {
                        // 二进制文件，尝试 Excel 降级解析
                        Self::read_excel(path)
                            .map_err(|_| ReaderError::UnsupportedFormat(name.clone()))?
                    }
                }
            }
        };

        let line_count = content.lines().count();
        let char_count = content.chars().count();

        Ok(FileContent {
            path: path.to_string_lossy().to_string(),
            name,
            format,
            content,
            size_bytes,
            line_count,
            char_count,
        })
    }

    /// 批量读取多个文件
    pub fn read_batch(paths: &[String]) -> Vec<Result<FileContent, ReaderError>> {
        paths.iter().map(|p| Self::read(p)).collect()
    }

    fn read_pdf(path: &Path) -> Result<String, ReaderError> {
        let bytes = fs::read(path)
            .map_err(|e| ReaderError::ReadError(format!("读取PDF文件失败: {}", e)))?;
        let text = pdf_extract::extract_text_from_mem(&bytes)
            .map_err(|e| ReaderError::PdfParseError(e.to_string()))?;
        Ok(text)
    }

    fn read_docx(path: &Path) -> Result<String, ReaderError> {
        let file = fs::File::open(path)
            .map_err(|e| ReaderError::ReadError(format!("打开DOCX文件失败: {}", e)))?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ReaderError::DocxParseError(format!("解析DOCX ZIP: {}", e)))?;

        let mut content = String::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)
                .map_err(|e| ReaderError::DocxParseError(format!("读取条目: {}", e)))?;
            if entry.name().ends_with(".xml") || entry.name().ends_with(".rels") {
                let mut buf = String::new();
                entry.read_to_string(&mut buf).ok();
                content.push_str(&buf);
                content.push('\n');
            }
        }

        // 从 XML 中提取文本内容
        let text = Self::extract_text_from_xml(&content);
        Ok(text)
    }

    fn read_excel(path: &Path) -> Result<String, ReaderError> {
        use calamine::{open_workbook_auto, Reader, Data};

        let mut workbook = open_workbook_auto(path)
            .map_err(|e| ReaderError::ExcelParseError(format!("打开Excel文件失败: {}", e)))?;

        let mut output = String::new();

        // 获取所有工作表名称
        let sheet_names = workbook.sheet_names().to_vec();
        let total_sheets = sheet_names.len();
        let _ = total_sheets; // 保留以备将来使用

        for (sheet_idx, sheet_name) in sheet_names.iter().enumerate() {
            if sheet_idx > 0 {
                output.push_str("\n---\n");
            }
            output.push_str(&format!("# 工作表: {}\n", sheet_name));

            if let Ok(range) = workbook.worksheet_range(sheet_name) {
                let rows = range.rows();
                let total_rows = range.get_size().0;
                let max_show_rows = 500usize;

                for (row_idx, row) in rows.enumerate() {
                    if row_idx >= max_show_rows {
                        output.push_str(&format!("\n... (共 {} 行，仅显示前 {} 行)\n", total_rows, max_show_rows));
                        break;
                    }

                    let cells: Vec<String> = row.iter()
                        .map(|cell| match cell {
                            Data::String(s) => s.clone(),
                            Data::Float(f) => {
                                if *f == f.floor() && f.is_finite() {
                                    format!("{}", *f as i64)
                                } else {
                                    format!("{}", f)
                                }
                            }
                            Data::Int(i) => format!("{}", i),
                            Data::Bool(b) => format!("{}", b),
                            Data::DateTime(d) => format!("{:?}", d),
                            Data::Empty => String::new(),
                            Data::Error(e) => format!("[ERR: {:?}]", e),
                            Data::DateTimeIso(d) => d.clone(),
                            Data::DurationIso(d) => d.clone(),
                        })
                        .collect();

                    if !cells.iter().all(|c: &String| c.is_empty()) {
                        output.push_str(&cells.join("\t"));
                        output.push('\n');
                    }
                }
            }
        }

        if output.trim().is_empty() {
            return Err(ReaderError::ExcelParseError("Excel文件中没有可读数据".to_string()));
        }

        Ok(output)
    }

    fn extract_text_from_xml(xml_content: &str) -> String {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(xml_content);
        reader.config_mut().trim_text(true);
        let mut text = String::new();
        let mut buf = Vec::new();
        let mut in_text = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    if name.contains("t") || name.contains("p") {
                        in_text = true;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_text {
                        let t = e.unescape().unwrap_or_default();
                        text.push_str(&t);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_lowercase();
                    if name.contains("t") || name.contains("p") {
                        in_text = false;
                    }
                    if name.contains("p") {
                        text.push('\n');
                    }
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        if text.trim().is_empty() {
            // 降级：移除所有 XML 标签
            let mut result = String::new();
            let mut in_tag = false;
            for ch in xml_content.chars() {
                match ch {
                    '<' => in_tag = true,
                    '>' => in_tag = false,
                    _ if !in_tag => result.push(ch),
                    _ => {}
                }
            }
            result
        } else {
            text
        }
    }
}
