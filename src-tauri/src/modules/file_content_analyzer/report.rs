use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::analyzer::ContentAnalysis;
use super::reader::ExcelSheet;

/// 分析报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub report_id: String,
    pub files_analyzed: usize,
    pub total_size_bytes: u64,
    pub analyses: Vec<ContentAnalysis>,
    pub html_report: String,
}

pub struct ReportGenerator;

impl ReportGenerator {
    /// 生成综合分析报告
    pub fn generate(analyses: Vec<ContentAnalysis>) -> AnalysisReport {
        let report_id = uuid::Uuid::new_v4().to_string();
        let files_analyzed = analyses.len();
        let total_size_bytes = analyses.iter().map(|a| a.file_size_bytes).sum();
        let html_report = Self::generate_generic_html(&analyses, &report_id);

        AnalysisReport {
            report_id,
            files_analyzed,
            total_size_bytes,
            analyses,
            html_report,
        }
    }

    /// 生成 Excel 深度分析报告（带 ECharts 图表）
    pub fn generate_excel_report(
        analyses: Vec<ContentAnalysis>,
        sheets: &[ExcelSheet],
    ) -> AnalysisReport {
        let report_id = uuid::Uuid::new_v4().to_string();
        let files_analyzed = analyses.len();
        let total_size_bytes = analyses.iter().map(|a| a.file_size_bytes).sum();

        // 合并所有工作表的数据
        let mut all_rows: Vec<HashMap<String, String>> = Vec::new();
        for sheet in sheets {
            all_rows.extend(sheet.rows.clone());
        }

        // 如果没有结构化数据，降级到通用报告
        if all_rows.is_empty() {
            return Self::generate(analyses);
        }

        let headers = sheets.first().map(|s| s.headers.clone()).unwrap_or_default();
        let analysis = ExcelDataAnalysis::from_rows(&all_rows, &headers);

        let html_report = Self::generate_excel_html(&analysis);

        AnalysisReport {
            report_id,
            files_analyzed,
            total_size_bytes,
            analyses,
            html_report,
        }
    }

    // ==================== 通用 HTML 报告（非 Excel） ====================

    fn generate_generic_html(analyses: &[ContentAnalysis], report_id: &str) -> String {
        let chart_data = Self::build_chart_data(analyses);
        let aggregated_keywords = Self::aggregate_keywords(analyses);
        let keyword_cloud = Self::build_keyword_cloud(&aggregated_keywords);
        let file_summaries = Self::build_file_summaries(analyses);

        format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>文件内容分析报告</title>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'PingFang SC', 'Microsoft YaHei', sans-serif; background: #0f172a; color: #e2e8f0; line-height: 1.6; }}
.container {{ max-width: 1200px; margin: 0 auto; padding: 24px; }}
.header {{ text-align: center; padding: 48px 24px; background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%); border-radius: 16px; margin-bottom: 32px; border: 1px solid #334155; }}
.header h1 {{ font-size: 2rem; background: linear-gradient(135deg, #6366f1, #8b5cf6); -webkit-background-clip: text; -webkit-text-fill-color: transparent; margin-bottom: 8px; }}
.header p {{ color: #94a3b8; font-size: 0.95rem; }}
.stats-grid {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 32px; }}
.stat-card {{ background: #1e293b; border-radius: 12px; padding: 20px; border: 1px solid #334155; text-align: center; }}
.stat-card .value {{ font-size: 1.75rem; font-weight: 700; color: #6366f1; }}
.stat-card .label {{ font-size: 0.85rem; color: #94a3b8; margin-top: 4px; }}
.section {{ margin-bottom: 32px; }}
.section-title {{ font-size: 1.25rem; font-weight: 600; margin-bottom: 16px; color: #f1f5f9; display: flex; align-items: center; gap: 8px; }}
.section-title::before {{ content: ''; width: 4px; height: 20px; background: linear-gradient(135deg, #6366f1, #8b5cf6); border-radius: 2px; }}
.chart-container {{ background: #1e293b; border-radius: 12px; padding: 24px; border: 1px solid #334155; }}
.chart-container canvas {{ max-height: 400px; }}
.keyword-cloud {{ display: flex; flex-wrap: wrap; gap: 8px; padding: 16px; background: #1e293b; border-radius: 12px; border: 1px solid #334155; }}
.keyword-tag {{ padding: 6px 14px; border-radius: 20px; font-size: 0.9rem; color: #e2e8f0; background: rgba(99,102,241,0.15); border: 1px solid rgba(99,102,241,0.3); transition: all 0.2s; }}
.keyword-tag:hover {{ background: rgba(99,102,241,0.3); transform: scale(1.05); }}
.file-card {{ background: #1e293b; border-radius: 12px; padding: 20px; border: 1px solid #334155; margin-bottom: 16px; }}
.file-card h3 {{ font-size: 1.05rem; color: #e2e8f0; margin-bottom: 12px; display: flex; align-items: center; gap: 8px; }}
.file-card .badge {{ display: inline-block; padding: 2px 10px; border-radius: 12px; font-size: 0.75rem; font-weight: 500; }}
.badge-text {{ background: rgba(99,102,241,0.2); color: #a5b4fc; }}
.badge-md {{ background: rgba(34,197,94,0.2); color: #86efac; }}
.badge-pdf {{ background: rgba(239,68,68,0.2); color: #fca5a5; }}
.badge-docx {{ background: rgba(59,130,246,0.2); color: #93c5fd; }}
.badge-json {{ background: rgba(234,179,8,0.2); color: #fde68a; }}
.file-meta {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(120px, 1fr)); gap: 12px; margin: 12px 0; }}
.meta-item {{ text-align: center; }}
.meta-item .val {{ font-size: 1.1rem; font-weight: 600; color: #cbd5e1; }}
.meta-item .lbl {{ font-size: 0.75rem; color: #64748b; }}
.structure-list {{ margin-top: 12px; }}
.structure-item {{ padding: 6px 12px; font-size: 0.85rem; color: #94a3b8; border-left: 2px solid #334155; margin-bottom: 4px; }}
.structure-item.l1 {{ border-left-color: #6366f1; }}
.structure-item.l2 {{ border-left-color: #8b5cf6; margin-left: 16px; }}
.structure-item.l3 {{ border-left-color: #a78bfa; margin-left: 32px; }}
.footer {{ text-align: center; padding: 24px; color: #475569; font-size: 0.85rem; }}
@media (max-width: 768px) {{
  .header h1 {{ font-size: 1.5rem; }}
  .stats-grid {{ grid-template-columns: repeat(2, 1fr); }}
  .file-meta {{ grid-template-columns: repeat(2, 1fr); }}
}}
</style>
<script src="https://cdn.jsdelivr.net/npm/chart.js@4"></script>
</head>
<body>
<div class="container">
  <div class="header">
    <h1>文件内容分析报告</h1>
    <p>分析时间: {timestamp} | 报告ID: {report_id}</p>
  </div>

  <div class="stats-grid">
    <div class="stat-card"><div class="value">{files_count}</div><div class="label">分析文件数</div></div>
    <div class="stat-card"><div class="value">{total_size}</div><div class="label">总大小</div></div>
    <div class="stat-card"><div class="value">{total_words}</div><div class="label">总词数</div></div>
    <div class="stat-card"><div class="value">{total_lines}</div><div class="label">总行数</div></div>
  </div>

  <div class="section">
    <div class="section-title">文件大小分布</div>
    <div class="chart-container"><canvas id="sizeChart"></canvas></div>
  </div>

  <div class="section">
    <div class="section-title">字符数分布</div>
    <div class="chart-container"><canvas id="charChart"></canvas></div>
  </div>

  <div class="section">
    <div class="section-title">总关键词云</div>
    <div class="keyword-cloud">{keyword_cloud_html}</div>
  </div>

  {file_summaries_html}

  <div class="footer">
    <p>由 DiskTidy 文件内容分析模块生成</p>
  </div>
</div>

<script>
{chart_script}
</script>
</body>
</html>"#,
            timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
            report_id = report_id,
            files_count = analyses.len(),
            total_size = Self::format_size(chart_data.total_size),
            total_words = chart_data.total_words,
            total_lines = chart_data.total_lines,
            keyword_cloud_html = keyword_cloud,
            file_summaries_html = file_summaries,
            chart_script = chart_data.script,
        )
    }

    // ==================== Excel ECharts 报告 ====================

    fn generate_excel_html(analysis: &ExcelDataAnalysis) -> String {
        let data_json = serde_json::to_string(&analysis)
            .unwrap_or_else(|_| "{}".to_string());

        format!(
            r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>数据分析报告</title>
<script src="https://cdn.jsdelivr.net/npm/echarts@5.5.0/dist/echarts.min.js"></script>
<style>
* {{ margin: 0; padding: 0; box-sizing: border-box; }}
body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Microsoft YaHei', sans-serif; background: #f0f2f5; color: #333; }}
.header {{ background: linear-gradient(135deg, #1a73e8, #0d47a1); color: #fff; padding: 32px 40px; }}
.header h1 {{ font-size: 28px; font-weight: 600; margin-bottom: 8px; }}
.header p {{ font-size: 14px; opacity: 0.85; }}
.container {{ max-width: 1400px; margin: 0 auto; padding: 24px; }}
.stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 16px; margin-bottom: 24px; }}
.stat-card {{ background: #fff; border-radius: 12px; padding: 20px 16px; box-shadow: 0 2px 8px rgba(0,0,0,0.06); text-align: center; transition: transform 0.2s; }}
.stat-card:hover {{ transform: translateY(-2px); box-shadow: 0 4px 16px rgba(0,0,0,0.1); }}
.stat-card .value {{ font-size: 28px; font-weight: 700; color: #1a73e8; margin-bottom: 4px; }}
.stat-card .value.small {{ font-size: 22px; }}
.stat-card .label {{ font-size: 13px; color: #888; }}
.section {{ margin-bottom: 24px; }}
.section-title {{ font-size: 18px; font-weight: 600; color: #333; margin-bottom: 16px; padding-left: 12px; border-left: 4px solid #1a73e8; }}
.chart-row {{ display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }}
.chart-row.full {{ grid-template-columns: 1fr; }}
.chart-card {{ background: #fff; border-radius: 12px; padding: 20px; box-shadow: 0 2px 8px rgba(0,0,0,0.06); }}
.chart-card .chart-inner {{ width: 100%; height: 380px; }}
.chart-card .chart-inner.tall {{ height: 420px; }}
.footer {{ text-align: center; padding: 24px; color: #999; font-size: 12px; }}
@media (max-width: 900px) {{ .chart-row {{ grid-template-columns: 1fr; }} }}
</style>
</head>
<body>
<div class="header">
  <h1>{title}</h1>
  <p>{subtitle}</p>
</div>
<div class="container">
  <div class="stats">
    {stats_html}
  </div>
  {charts_html}
</div>
<div class="footer">{footer_text}</div>
<script>
var DATA = {data_json};

var colors = ['#5470c6','#91cc75','#fac858','#ee6666','#73c0de','#3ba272','#fc8452','#9a60b4','#ea7ccc','#48b8d0'];

{chart_scripts}

window.addEventListener('resize', function() {{
  document.querySelectorAll('.chart-inner').forEach(function(el) {{
    var instance = echarts.getInstanceByDom(el);
    if (instance) instance.resize();
  }});
}});
</script>
</body>
</html>"#,
            title = analysis.title,
            subtitle = analysis.subtitle,
            stats_html = Self::build_stats_html(analysis),
            charts_html = Self::build_charts_sections(analysis),
            footer_text = analysis.footer,
            data_json = data_json,
            chart_scripts = Self::build_chart_scripts(analysis),
        )
    }

    fn build_stats_html(analysis: &ExcelDataAnalysis) -> String {
        analysis.stats.iter().map(|s| {
            format!(
                r#"<div class="stat-card"><div class="value{}">{}</div><div class="label">{}</div></div>"#,
                if s.value.len() > 8 { " small" } else { "" },
                s.value,
                s.label,
            )
        }).collect::<Vec<_>>().join("\n")
    }

    fn build_charts_sections(analysis: &ExcelDataAnalysis) -> String {
        analysis.charts.iter().map(|c| {
            let row_class = if c.full_width { "chart-row full" } else { "chart-row" };
            let inner_class = if c.full_width { "chart-inner tall" } else { "chart-inner" };

            let cards_html: String = c.panels.iter().map(|p| {
                format!(
                    r#"<div class="chart-card"><div class="{}" id="chart-{}"></div></div>"#,
                    inner_class, p.id
                )
            }).collect::<Vec<_>>().join("\n");

            format!(
                r#"<div class="section"><div class="section-title">{title}</div><div class="{row_class}">{cards}</div></div>"#,
                title = c.title,
                row_class = row_class,
                cards = cards_html,
            )
        }).collect::<Vec<_>>().join("\n")
    }

    fn build_chart_scripts(analysis: &ExcelDataAnalysis) -> String {
        analysis.charts.iter().map(|c| {
            c.panels.iter().map(|p| {
                format!(
                    "var c{id} = echarts.init(document.getElementById('chart-{id}'));\nc{id}.setOption({option});\n",
                    id = p.id,
                    option = p.option
                )
            }).collect::<Vec<_>>().join("\n")
        }).collect::<Vec<_>>().join("\n")
    }

    // ==================== 通用报告辅助函数 ====================

    fn format_size(bytes: u64) -> String {
        if bytes >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", bytes as f64 / (1024 * 1024 * 1024) as f64)
        } else if bytes >= 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024 * 1024) as f64)
        } else if bytes >= 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{} B", bytes)
        }
    }

    fn build_chart_data(analyses: &[ContentAnalysis]) -> ChartData {
        let labels: Vec<String> = analyses.iter().map(|a| {
            let name = &a.file_name;
            if name.len() > 20 { format!("{}...", &name[..20]) } else { name.clone() }
        }).collect();
        let sizes: Vec<u64> = analyses.iter().map(|a| a.file_size_bytes).collect();
        let chars: Vec<usize> = analyses.iter().map(|a| a.total_chars).collect();
        let total_size: u64 = sizes.iter().sum();
        let total_words: usize = analyses.iter().map(|a| a.total_words).sum();
        let total_lines: usize = analyses.iter().map(|a| a.total_lines).sum();

        let labels_json = serde_json::to_string(&labels).unwrap_or_default();
        let sizes_json = serde_json::to_string(&sizes).unwrap_or_default();
        let chars_json = serde_json::to_string(&chars).unwrap_or_default();

        let script = format!(
            r#"
const labels = {labels};
const sizeData = {sizes};
const charData = {chars};

new Chart(document.getElementById('sizeChart'), {{
  type: 'bar',
  data: {{
    labels: labels,
    datasets: [{{
      label: '文件大小 (bytes)',
      data: sizeData,
      backgroundColor: labels.map((_, i) => {{
        const hue = (i * 360 / labels.length) % 360;
        return `hsla(${{hue}}, 70%, 60%, 0.7)`;
      }}),
      borderColor: labels.map((_, i) => {{
        const hue = (i * 360 / labels.length) % 360;
        return `hsla(${{hue}}, 70%, 60%, 1)`;
      }}),
      borderWidth: 1,
      borderRadius: 6,
    }}]
  }},
  options: {{
    responsive: true,
    maintainAspectRatio: true,
    plugins: {{ legend: {{ display: false }} }},
    scales: {{
      x: {{ ticks: {{ color: '#94a3b8', maxRotation: 45 }} }},
      y: {{ ticks: {{ color: '#94a3b8', callback: v => v >= 1e6 ? (v/1e6).toFixed(1)+'MB' : v >= 1e3 ? (v/1e3).toFixed(1)+'KB' : v+'B' }} }}
    }}
  }}
}});

new Chart(document.getElementById('charChart'), {{
  type: 'bar',
  data: {{
    labels: labels,
    datasets: [{{
      label: '字符数',
      data: charData,
      backgroundColor: labels.map((_, i) => {{
        const hue = (i * 360 / labels.length + 180) % 360;
        return `hsla(${{hue}}, 70%, 60%, 0.7)`;
      }}),
      borderColor: labels.map((_, i) => {{
        const hue = (i * 360 / labels.length + 180) % 360;
        return `hsla(${{hue}}, 70%, 60%, 1)`;
      }}),
      borderWidth: 1,
      borderRadius: 6,
    }}]
  }},
  options: {{
    responsive: true,
    maintainAspectRatio: true,
    plugins: {{ legend: {{ display: false }} }},
    scales: {{
      x: {{ ticks: {{ color: '#94a3b8', maxRotation: 45 }} }},
      y: {{ ticks: {{ color: '#94a3b8' }} }}
    }}
  }}
}});
"#,
            labels = labels_json,
            sizes = sizes_json,
            chars = chars_json,
        );

        ChartData {
            total_size,
            total_words,
            total_lines,
            script,
        }
    }

    fn aggregate_keywords(analyses: &[ContentAnalysis]) -> Vec<AggregatedKeyword> {
        let mut map: HashMap<String, (usize, f64)> = HashMap::new();
        for analysis in analyses {
            for kw in &analysis.top_keywords {
                let entry = map.entry(kw.word.clone()).or_insert((0, 0.0));
                entry.0 += kw.count;
                entry.1 += kw.score;
            }
        }

        let mut items: Vec<AggregatedKeyword> = map.into_iter()
            .map(|(word, (count, score))| AggregatedKeyword { word, count, score })
            .collect();
        items.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(30);
        items
    }

    fn build_keyword_cloud(keywords: &[AggregatedKeyword]) -> String {
        let max_score = keywords.first().map(|k| k.score).unwrap_or(1.0);
        keywords.iter().map(|kw| {
            let font_size = 0.8 + (kw.score / max_score.max(1.0)) * 1.4;
            format!(
                r#"<span class="keyword-tag" style="font-size:{:.2}rem;">{}</span>"#,
                font_size, kw.word
            )
        }).collect::<Vec<_>>().join("\n")
    }

    fn build_file_summaries(analyses: &[ContentAnalysis]) -> String {
        analyses.iter().map(|analysis| {
            let badge_class = match analysis.file_format.as_str() {
                "纯文本" => "badge-text",
                "Markdown" => "badge-md",
                "PDF" => "badge-pdf",
                "Word文档" => "badge-docx",
                "JSON" | "XML" | "CSV" => "badge-json",
                _ => "badge-text",
            };

            let keywords_html = analysis.top_keywords.iter().take(8)
                .map(|kw| format!("<span style='color:#6366f1;'>#{}</span>", kw.word))
                .collect::<Vec<_>>().join(" ");

            let structure_html = if analysis.structure.is_empty() {
                String::from("<p style='color:#64748b;font-size:0.85rem;'>无明显结构</p>")
            } else {
                analysis.structure.iter().map(|s| {
                    format!(
                        r#"<div class="structure-item l{}">{}</div>"#,
                        s.level, s.title
                    )
                }).collect::<Vec<_>>().join("\n")
            };

            format!(
                r#"<div class="file-card">
  <h3><span class="badge {}">{}</span> {}</h3>
  <div class="file-meta">
    <div class="meta-item"><div class="val">{}</div><div class="lbl">文件大小</div></div>
    <div class="meta-item"><div class="val">{}</div><div class="lbl">总词数</div></div>
    <div class="meta-item"><div class="val">{}</div><div class="lbl">行数</div></div>
    <div class="meta-item"><div class="val">{:.1}分钟</div><div class="lbl">阅读时间</div></div>
    <div class="meta-item"><div class="val">{}</div><div class="lbl">推测语言</div></div>
  </div>
  <p style="margin:8px 0;font-size:0.85rem;color:#94a3b8;">关键词: {}</p>
  <div class="structure-list">{}</div>
</div>"#,
                badge_class, analysis.file_format, analysis.file_name,
                Self::format_size(analysis.file_size_bytes),
                analysis.total_words,
                analysis.total_lines,
                analysis.reading_time_minutes,
                analysis.language_hint,
                keywords_html,
                structure_html,
            )
        }).collect::<Vec<_>>().join("\n")
    }
}

// ==================== Excel 数据分析模型 ====================

/// 序列化为 JSON 注入 HTML 的完整分析数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcelDataAnalysis {
    pub title: String,
    pub subtitle: String,
    pub footer: String,
    pub stats: Vec<StatItem>,
    pub charts: Vec<ChartSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatItem {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSection {
    pub title: String,
    pub full_width: bool,
    pub panels: Vec<ChartPanel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPanel {
    pub id: String,
    /// 完整的 ECharts option JSON（不含外层大括号）
    pub option: String,
}

impl ExcelDataAnalysis {
    /// 从 Excel 行数据中自动识别字段并生成分析
    pub fn from_rows(rows: &[HashMap<String, String>], headers: &[String]) -> Self {
        let col_map = ColumnMapping::detect(headers);
        let total = rows.len();

        // === 概览统计 ===
        let total_amount = Self::sum_f64(rows, &col_map.amount_col);
        let unique_suppliers = Self::count_unique(rows, &col_map.supplier_col);
        let unique_depts = Self::count_unique(rows, &col_map.dept_col);
        let unique_buyers = Self::count_unique(rows, &col_map.buyer_col);
        let unique_products = Self::count_unique(rows, &col_map.product_col);

        let title = "数据分析报告".to_string();
        let date_range = Self::date_range(rows, &col_map.date_col);
        let subtitle = format!("数据范围: {} | 共 {} 条记录 | 报告生成日期: {}",
            date_range, total,
            chrono::Local::now().format("%Y年%m月%d日").to_string(),
        );
        let footer = format!("数据来源: 分析文件 | 本报告由 DiskTidy 自动生成");

        let mut stats = vec![
            StatItem { value: format!("¥{:.2}万", total_amount / 10000.0), label: "总金额".to_string() },
            StatItem { value: total.to_string(), label: "总记录数".to_string() },
        ];
        if unique_suppliers > 0 {
            stats.push(StatItem { value: unique_suppliers.to_string(), label: "供应商数".to_string() });
        }
        if unique_products > 0 {
            stats.push(StatItem { value: unique_products.to_string(), label: "商品种类".to_string() });
        }
        if unique_buyers > 0 {
            stats.push(StatItem { value: unique_buyers.to_string(), label: "采购员数".to_string() });
        }
        if unique_depts > 0 {
            stats.push(StatItem { value: unique_depts.to_string(), label: "部门数".to_string() });
        }

        // === 图表 ===
        let mut charts = Vec::new();

        // 1. 月度趋势（如果有日期列且金额列）
        if !col_map.date_col.is_empty() && !col_map.amount_col.is_empty() {
            let (m_labels, m_values, m_counts) = Self::monthly_agg(rows, &col_map.date_col, &col_map.amount_col);
            if !m_labels.is_empty() {
                let ml = serde_json::to_string(&m_labels).unwrap_or_default();
                let mv = serde_json::to_string(&m_values).unwrap_or_default();
                let mc = serde_json::to_string(&m_counts).unwrap_or_default();
                charts.push(ChartSection {
                    title: "月度趋势".to_string(),
                    full_width: true,
                    panels: vec![ChartPanel {
                        id: "monthly".to_string(),
                        option: format!(r#"{{
  tooltip: {{ trigger: 'axis', axisPointer: {{ type: 'cross' }} }},
  legend: {{ data: ['金额', '记录数'], top: 5 }},
  grid: {{ left: 70, right: 70, top: 50, bottom: 40 }},
  xAxis: {{ type: 'category', data: {ml}, axisLabel: {{ rotate: 30 }} }},
  yAxis: [
    {{ type: 'value', name: '金额(元)', axisLabel: {{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
    {{ type: 'value', name: '记录数' }}
  ],
  series: [
    {{
      name: '金额', type: 'line', smooth: true, data: {mv},
      itemStyle: {{ color: '#5470c6' }},
      areaStyle: {{ color: new echarts.graphic.LinearGradient(0,0,0,1,[
        {{offset:0,color:'rgba(84,112,198,0.3)'}},{{offset:1,color:'rgba(84,112,198,0.02)'}}]) }},
      markLine: {{ silent: true, data: [{{ type:'average', name:'均值' }}], label: {{ formatter: '均值\\n¥{{c}}' }} }}
    }},
    {{
      name: '记录数', type: 'bar', yAxisIndex: 1, data: {mc},
      itemStyle: {{ color: '#91cc75', borderRadius: [4,4,0,0] }}
    }}
  ]
}}"#),
                    }],
                });
            }
        }

        // 2. 供应商排行
        if !col_map.supplier_col.is_empty() && !col_map.amount_col.is_empty() {
            let (s_names, s_values) = Self::group_sum_top(rows, &col_map.supplier_col, &col_map.amount_col, 10);
            if !s_names.is_empty() {
                let sn = serde_json::to_string(&s_names).unwrap_or_default();
                let sv = serde_json::to_string(&s_values).unwrap_or_default();
                let supplier_chart = ChartPanel {
                    id: "supplier".to_string(),
                    option: format!(r#"{{
  tooltip: {{ trigger: 'axis', axisPointer: {{ type: 'shadow' }} }},
  grid: {{ left: 140, right: 50, top: 20, bottom: 20 }},
  xAxis: {{ type: 'value', axisLabel: {{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
  yAxis: {{ type: 'category', data: {sn}, inverse: true, axisLabel: {{ fontSize: 11 }} }},
  series: [{{
    type: 'bar', data: {sv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
    label: {{ show: true, position: 'right', formatter: p => '¥'+(p.value/10000).toFixed(1)+'万', fontSize: 10 }},
    barMaxWidth: 26
  }}]
}}"#),
                };

                // 部门饼图（如果有部门列）
                if !col_map.dept_col.is_empty() {
                    let d_data = Self::group_sum_pie(rows, &col_map.dept_col, &col_map.amount_col);
                    let dd = serde_json::to_string(&d_data).unwrap_or_default();
                    let dept_chart = ChartPanel {
                        id: "dept".to_string(),
                        option: format!(r#"{{
  tooltip: {{ trigger: 'item', formatter: '{{b}}: ¥{{c}} ({{d}}%)' }},
  legend: {{ orient: 'vertical', right: 10, top: 'center', itemWidth: 12, itemHeight: 12 }},
  series: [{{
    type: 'pie', radius: ['40%','70%'], center: ['40%','50%'],
    data: {dd}, label: {{ formatter: '{{b}}\\n{{d}}%' }},
    itemStyle: {{ borderRadius: 4, borderColor: '#fff', borderWidth: 2 }},
    emphasis: {{ itemStyle: {{ shadowBlur: 10, shadowOffsetX: 0, shadowColor: 'rgba(0,0,0,0.3)' }} }}
  }}]
}}"#),
                    };
                    charts.push(ChartSection {
                        title: "供应商 / 部门分布".to_string(),
                        full_width: false,
                        panels: vec![supplier_chart, dept_chart],
                    });
                } else {
                    charts.push(ChartSection {
                        title: "Top 供应商".to_string(),
                        full_width: false,
                        panels: vec![supplier_chart],
                    });
                }
            }
        }

        // 3. 采购员/人员业绩
        if !col_map.buyer_col.is_empty() && !col_map.amount_col.is_empty() {
            let (b_names, b_values, b_counts) = Self::buyer_agg(rows, &col_map.buyer_col, &col_map.amount_col);
            if !b_names.is_empty() {
                let bn = serde_json::to_string(&b_names).unwrap_or_default();
                let bv = serde_json::to_string(&b_values).unwrap_or_default();
                let bc = serde_json::to_string(&b_counts).unwrap_or_default();
                charts.push(ChartSection {
                    title: "采购员业绩".to_string(),
                    full_width: true,
                    panels: vec![ChartPanel {
                        id: "buyer".to_string(),
                        option: format!(r#"{{
  tooltip: {{ trigger: 'axis', axisPointer: {{ type: 'shadow' }} }},
  legend: {{ data: ['金额', '记录数'], top: 5 }},
  grid: {{ left: 60, right: 60, top: 40, bottom: 30 }},
  xAxis: {{ type: 'category', data: {bn} }},
  yAxis: [
    {{ type: 'value', name: '金额(元)', axisLabel: {{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
    {{ type: 'value', name: '记录数' }}
  ],
  series: [
    {{
      name: '金额', type: 'bar', data: {bv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
      label: {{ show: true, position: 'top', formatter: p => '¥'+(p.value/10000).toFixed(1)+'万', fontSize: 9 }},
      barMaxWidth: 38
    }},
    {{
      name: '记录数', type: 'line', yAxisIndex: 1, data: {bc},
      itemStyle: {{ color: '#ee6666' }}, symbolSize: 8, lineStyle: {{ width: 2 }}
    }}
  ]
}}"#),
                    }],
                });
            }
        }

        // 4. 商品分析
        if !col_map.product_col.is_empty() && !col_map.amount_col.is_empty() {
            let (p_names, p_values) = Self::group_sum_top(rows, &col_map.product_col, &col_map.amount_col, 15);
            if !p_names.is_empty() {
                let pn = serde_json::to_string(&p_names).unwrap_or_default();
                let pv = serde_json::to_string(&p_values).unwrap_or_default();
                let product_chart = ChartPanel {
                    id: "product".to_string(),
                    option: format!(r#"{{
  tooltip: {{ trigger: 'axis', axisPointer: {{ type: 'shadow' }} }},
  grid: {{ left: 120, right: 50, top: 20, bottom: 20 }},
  xAxis: {{ type: 'value', axisLabel: {{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
  yAxis: {{ type: 'category', data: {pn}, inverse: true, axisLabel: {{ fontSize: 11 }} }},
  series: [{{
    type: 'bar', data: {pv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
    label: {{ show: true, position: 'right', formatter: p => '¥'+(p.value/10000).toFixed(1)+'万', fontSize: 10 }},
    barMaxWidth: 20
  }}]
}}"#),
                };

                // 备注/类型饼图
                if !col_map.note_col.is_empty() {
                    let n_data = Self::group_sum_pie(rows, &col_map.note_col, &col_map.amount_col);
                    let nd = serde_json::to_string(&n_data).unwrap_or_default();
                    let note_chart = ChartPanel {
                        id: "note".to_string(),
                        option: format!(r#"{{
  tooltip: {{ trigger: 'item', formatter: '{{b}}: ¥{{c}} ({{d}}%)' }},
  legend: {{ orient: 'vertical', right: 10, top: 'center', itemWidth: 12, itemHeight: 12 }},
  series: [{{
    type: 'pie', radius: ['40%','70%'], center: ['40%','50%'],
    data: {nd}, label: {{ formatter: '{{b}}\\n{{d}}%' }},
    itemStyle: {{ borderRadius: 4, borderColor: '#fff', borderWidth: 2 }}
  }}]
}}"#),
                    };
                    charts.push(ChartSection {
                        title: "商品分析 / 类型分布".to_string(),
                        full_width: false,
                        panels: vec![product_chart, note_chart],
                    });
                } else {
                    charts.push(ChartSection {
                        title: "Top 商品".to_string(),
                        full_width: false,
                        panels: vec![product_chart],
                    });
                }
            }
        }

        // 5. 数量分析（如果有数量列）
        if !col_map.qty_col.is_empty() && !col_map.product_col.is_empty() {
            let (q_names, q_values) = Self::group_sum_top(rows, &col_map.product_col, &col_map.qty_col, 12);
            if !q_names.is_empty() && q_values.iter().any(|&v| v > 0.0) {
                let qn = serde_json::to_string(&q_names).unwrap_or_default();
                let qv = serde_json::to_string(&q_values).unwrap_or_default();
                charts.push(ChartSection {
                    title: "采购数量排行".to_string(),
                    full_width: false,
                    panels: vec![ChartPanel {
                        id: "qty".to_string(),
                        option: format!(r#"{{
  tooltip: {{ trigger: 'axis', axisPointer: {{ type: 'shadow' }} }},
  grid: {{ left: 120, right: 50, top: 20, bottom: 20 }},
  xAxis: {{ type: 'value' }},
  yAxis: {{ type: 'category', data: {qn}, inverse: true, axisLabel: {{ fontSize: 11 }} }},
  series: [{{
    type: 'bar', data: {qv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
    label: {{ show: true, position: 'right', fontSize: 10 }},
    barMaxWidth: 22
  }}]
}}"#),
                    }],
                });
            }
        }

        ExcelDataAnalysis {
            title,
            subtitle,
            footer,
            stats,
            charts,
        }
    }

    // ===== 辅助方法 =====

    fn get_val(row: &HashMap<String, String>, col: &str) -> String {
        row.get(col).cloned().unwrap_or_default()
    }

    fn parse_f64(s: &str) -> f64 {
        let s = s.trim().trim_start_matches('¥').replace(',', "");
        s.parse::<f64>().unwrap_or(0.0)
    }

    fn sum_f64(rows: &[HashMap<String, String>], col: &str) -> f64 {
        if col.is_empty() { return 0.0; }
        rows.iter().map(|r| Self::parse_f64(&Self::get_val(r, col))).sum()
    }

    fn count_unique(rows: &[HashMap<String, String>], col: &str) -> usize {
        if col.is_empty() { return 0; }
        let set: std::collections::HashSet<String> = rows.iter()
            .map(|r| Self::get_val(r, col))
            .filter(|v| !v.is_empty())
            .collect();
        set.len()
    }

    fn date_range(rows: &[HashMap<String, String>], col: &str) -> String {
        if col.is_empty() { return "全部".to_string(); }
        let dates: Vec<String> = rows.iter()
            .map(|r| Self::get_val(r, col))
            .filter(|v| !v.is_empty())
            .collect();
        if dates.is_empty() { return "全部".to_string(); }
        let empty = String::new();
        let min_d = dates.iter().min().unwrap_or(&empty);
        let max_d = dates.iter().max().unwrap_or(&empty);
        format!("{} - {}", min_d, max_d)
    }

    fn monthly_agg(rows: &[HashMap<String, String>], date_col: &str, amount_col: &str)
        -> (Vec<String>, Vec<f64>, Vec<usize>) {
        let mut map: HashMap<String, (f64, usize)> = HashMap::new();
        for row in rows {
            let date_str = Self::get_val(row, date_col);
            let month_key = Self::extract_month(&date_str);
            let amount = Self::parse_f64(&Self::get_val(row, amount_col));
            let entry = map.entry(month_key).or_insert((0.0, 0));
            entry.0 += amount;
            entry.1 += 1;
        }
        let mut items: Vec<(String, f64, usize)> = map.into_iter()
            .map(|(k, (v, c))| (k, v, c))
            .collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        let labels: Vec<String> = items.iter().map(|i| i.0.clone()).collect();
        let values: Vec<f64> = items.iter().map(|i| i.1).collect();
        let counts: Vec<usize> = items.iter().map(|i| i.2).collect();
        (labels, values, counts)
    }

    fn extract_month(date_str: &str) -> String {
        // 支持多种日期格式: 2026-01-15, 2026/01/15, 20260115, 2026年01月
        let s = date_str.trim();
        // 取前7或8个包含年月信息的字符
        if s.len() >= 7 {
            if s.chars().nth(4) == Some('-') || s.chars().nth(4) == Some('/') {
                return s[..7].to_string();
            }
            if s.contains('年') && s.contains('月') {
                let y_end = s.find('年').unwrap_or(4);
                let m_end = s.find('月').unwrap_or(7);
                return format!("{}年{}月", &s[..y_end], &s[y_end+3..m_end]);
            }
        }
        s.chars().take(7).collect()
    }

    fn group_sum_top(rows: &[HashMap<String, String>], group_col: &str, value_col: &str, top_n: usize)
        -> (Vec<String>, Vec<f64>) {
        let mut map: HashMap<String, f64> = HashMap::new();
        for row in rows {
            let key = Self::get_val(row, group_col);
            if key.is_empty() { continue; }
            let val = Self::parse_f64(&Self::get_val(row, value_col));
            *map.entry(key).or_insert(0.0) += val;
        }
        let mut items: Vec<(String, f64)> = map.into_iter().collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items.truncate(top_n);
        let names: Vec<String> = items.iter().map(|i| i.0.clone()).collect();
        let values: Vec<f64> = items.iter().map(|i| i.1).collect();
        (names, values)
    }

    fn group_sum_pie(rows: &[HashMap<String, String>], group_col: &str, value_col: &str)
        -> Vec<Value> {
        let mut map: HashMap<String, f64> = HashMap::new();
        for row in rows {
            let key = Self::get_val(row, group_col);
            if key.is_empty() { continue; }
            let val = Self::parse_f64(&Self::get_val(row, value_col));
            *map.entry(key).or_insert(0.0) += val;
        }
        map.into_iter()
            .map(|(name, value)| serde_json::json!({ "name": name, "value": value }))
            .collect()
    }

    fn buyer_agg(rows: &[HashMap<String, String>], buyer_col: &str, amount_col: &str)
        -> (Vec<String>, Vec<f64>, Vec<usize>) {
        let mut map: HashMap<String, (f64, usize)> = HashMap::new();
        for row in rows {
            let key = Self::get_val(row, buyer_col);
            if key.is_empty() { continue; }
            let amount = Self::parse_f64(&Self::get_val(row, amount_col));
            let entry = map.entry(key).or_insert((0.0, 0));
            entry.0 += amount;
            entry.1 += 1;
        }
        let mut items: Vec<(String, f64, usize)> = map.into_iter()
            .map(|(k, (v, c))| (k, v, c))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let names: Vec<String> = items.iter().map(|i| i.0.clone()).collect();
        let values: Vec<f64> = items.iter().map(|i| i.1).collect();
        let counts: Vec<usize> = items.iter().map(|i| i.2).collect();
        (names, values, counts)
    }
}

// ==================== 列名智能匹配 ====================

struct ColumnMapping {
    date_col: String,
    amount_col: String,
    supplier_col: String,
    dept_col: String,
    buyer_col: String,
    product_col: String,
    note_col: String,
    qty_col: String,
}

impl ColumnMapping {
    fn detect(headers: &[String]) -> Self {
        ColumnMapping {
            date_col: Self::find_col(headers, &["日期", "date", "时间"]),
            amount_col: Self::find_col(headers, &["金额", "amount", "总价", "合计", "总金额"]),
            supplier_col: Self::find_col(headers, &["供应商", "supplier", "供货"]),
            dept_col: Self::find_col(headers, &["部门", "dept", "区域", "品类"]),
            buyer_col: Self::find_col(headers, &["采购员", "buyer", "经办人", "负责人", "员工"]),
            product_col: Self::find_col(headers, &["商品", "product", "名称", "品名", "货品", "物品"]),
            note_col: Self::find_col(headers, &["备注", "note", "类型", "说明", "分类"]),
            qty_col: Self::find_col(headers, &["数量", "qty", "quantity", "采购数量", "件数"]),
        }
    }

    fn find_col(headers: &[String], keywords: &[&str]) -> String {
        for h in headers {
            let h_lower = h.to_lowercase();
            for kw in keywords {
                if h_lower.contains(kw) {
                    return h.clone();
                }
            }
        }
        String::new()
    }
}

// ==================== 内部结构体 ====================

struct ChartData {
    total_size: u64,
    total_words: usize,
    total_lines: usize,
    script: String,
}

struct AggregatedKeyword {
    word: String,
    #[allow(dead_code)]
    count: usize,
    score: f64,
}
