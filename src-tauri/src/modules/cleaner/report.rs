use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use crate::models::{CleanError, CleanResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanReportData {
    pub clean_id: String,
    pub start_time: u64,
    pub end_time: u64,
    pub duration_ms: u64,
    pub total_files: u64,
    pub cleaned_files: u64,
    pub failed_files: u64,
    pub skipped_files: u64,
    pub total_size: u64,
    pub cleaned_size: u64,
    pub errors: Vec<CleanError>,
    pub summary: CleanSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanSummary {
    pub success_rate: f32,
    pub average_speed: u64,
    pub error_categories: Vec<ErrorCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorCategory {
    pub error_code: String,
    pub count: u64,
    pub examples: Vec<String>,
}

pub struct CleanReportGenerator {
    clean_id: String,
    start_time: u64,
}

impl CleanReportGenerator {
    pub fn new() -> Self {
        Self {
            clean_id: uuid::Uuid::new_v4().to_string(),
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn with_id(clean_id: String) -> Self {
        Self {
            clean_id,
            start_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    pub fn generate(&self, result: &CleanResult) -> CleanReportData {
        let end_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let summary = self.generate_summary(result);

        CleanReportData {
            clean_id: self.clean_id.clone(),
            start_time: self.start_time,
            end_time,
            duration_ms: result.duration_ms,
            total_files: result.total_files,
            cleaned_files: result.cleaned_files,
            failed_files: result.failed_files,
            skipped_files: result.skipped_files,
            total_size: result.total_size,
            cleaned_size: result.cleaned_size,
            errors: result.errors.clone(),
            summary,
        }
    }

    fn generate_summary(&self, result: &CleanResult) -> CleanSummary {
        let success_rate = if result.total_files > 0 {
            (result.cleaned_files as f32 / result.total_files as f32) * 100.0
        } else {
            0.0
        };

        let average_speed = if result.duration_ms > 0 {
            (result.cleaned_size * 1000) / result.duration_ms as u64
        } else {
            0
        };

        let error_categories = self.categorize_errors(&result.errors);

        CleanSummary {
            success_rate,
            average_speed,
            error_categories,
        }
    }

    fn categorize_errors(&self, errors: &[CleanError]) -> Vec<ErrorCategory> {
        let mut categories: std::collections::HashMap<String, (u64, Vec<String>)> =
            std::collections::HashMap::new();

        for error in errors {
            let entry = categories.entry(error.error_code.clone()).or_insert((0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 3 {
                entry.1.push(error.path.clone());
            }
        }

        categories
            .into_iter()
            .map(|(code, (count, examples))| ErrorCategory {
                error_code: code,
                count,
                examples,
            })
            .collect()
    }

    pub fn export_json(&self, report: &CleanReportData) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(report)
    }

    pub fn export_html(&self, report: &CleanReportData) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>DiskTidy 清理报告</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            background: #1a1a2e;
            color: #e0e0e0;
        }}
        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: #8b5cf6;
            margin-bottom: 10px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 15px;
            margin-bottom: 30px;
        }}
        .summary-card {{
            background: rgba(139, 92, 246, 0.1);
            border-radius: 8px;
            padding: 15px;
            text-align: center;
        }}
        .summary-card .value {{
            font-size: 24px;
            font-weight: bold;
            color: #8b5cf6;
        }}
        .summary-card .label {{
            font-size: 12px;
            color: #888;
            margin-top: 5px;
        }}
        .section {{
            background: rgba(255, 255, 255, 0.05);
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
        }}
        .section h2 {{
            color: #8b5cf6;
            margin-top: 0;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
        }}
        th, td {{
            padding: 10px;
            text-align: left;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }}
        th {{
            color: #8b5cf6;
        }}
        .success {{ color: #52c41a; }}
        .error {{ color: #ff4d4f; }}
        .footer {{
            text-align: center;
            margin-top: 30px;
            color: #666;
            font-size: 12px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>DiskTidy 清理报告</h1>
        <p>清理ID: {}</p>
    </div>

    <div class="summary">
        <div class="summary-card">
            <div class="value">{}</div>
            <div class="label">已清理文件</div>
        </div>
        <div class="summary-card">
            <div class="value">{}</div>
            <div class="label">已释放空间</div>
        </div>
        <div class="summary-card">
            <div class="value">{:.1}%</div>
            <div class="label">成功率</div>
        </div>
    </div>

    <div class="section">
        <h2>清理统计</h2>
        <table>
            <tr>
                <th>项目</th>
                <th>数值</th>
            </tr>
            <tr>
                <td>总文件数</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>已清理</td>
                <td class="success">{}</td>
            </tr>
            <tr>
                <td>失败</td>
                <td class="error">{}</td>
            </tr>
            <tr>
                <td>跳过</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>总大小</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>已清理大小</td>
                <td>{}</td>
            </tr>
            <tr>
                <td>耗时</td>
                <td>{} ms</td>
            </tr>
        </table>
    </div>

    {}

    <div class="footer">
        <p>由 DiskTidy 生成 - {}</p>
    </div>
</body>
</html>"#,
            report.clean_id,
            report.cleaned_files,
            format_size(report.cleaned_size),
            report.summary.success_rate,
            report.total_files,
            report.cleaned_files,
            report.failed_files,
            report.skipped_files,
            format_size(report.total_size),
            format_size(report.cleaned_size),
            report.duration_ms,
            self.generate_error_section(report),
            chrono::DateTime::from_timestamp(report.end_time as i64, 0)
                .map(|t| t.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        )
    }

    fn generate_error_section(&self, report: &CleanReportData) -> String {
        if report.errors.is_empty() {
            return String::new();
        }

        let mut html = String::from(r#"<div class="section">
        <h2>错误详情</h2>
        <table>
            <tr>
                <th>文件路径</th>
                <th>错误代码</th>
                <th>错误信息</th>
            </tr>"#);

        for error in &report.errors {
            html.push_str(&format!(
                r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td class="error">{}</td>
            </tr>"#,
                error.path,
                error.error_code,
                error.error_message
            ));
        }

        html.push_str("</table></div>");
        html
    }
}

impl Default for CleanReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}
