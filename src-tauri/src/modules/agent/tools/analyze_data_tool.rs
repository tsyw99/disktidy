use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modules::agent::tools::tool_error::ToolError;
use super::ToolPrompt;
use super::excel_cache;
use super::excel_cache::cache_set_report;

#[derive(Debug, Deserialize)]
pub struct AnalyzeDataInput {
    /// 之前 read_excel 使用的目录路径（用于从缓存取数据）
    pub directory: String,
    /// 分析维度列表
    pub dimensions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeDataOutput {
    /// 分析报告（不再传给 LLM，仅用于命令行展示；大 JSON 已存入缓存）
    #[serde(skip)]
    pub report: Option<ReportData>,
    /// 汇总摘要（给 LLM 看的）
    pub summary: String,
    pub dimension_count: usize,
    pub total_rows: usize,
    pub _prompt: String,
}

impl ToolPrompt for AnalyzeDataTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 数据分析完成
分析报告已生成并缓存。共处理 {total_rows} 条记录，覆盖 {dimension_count} 个维度。

下一步调用 `generate_html` 生成可视化报告：
- `report_key`: 传入目录路径（与 read_excel 的 directory 相同），工具会从缓存读取报告数据
- `output_path`: 传入下载目录路径，加上 "数据分析报告.html" 作为文件名

重要：不要尝试在函数调用中传递完整的 JSON 数据，只需传递 `report_key` (字符串) 即可。
"#
    }
}

pub struct AnalyzeDataTool;

impl AnalyzeDataTool {
    pub fn new() -> Self { Self }
}

impl rig_core::tool::Tool for AnalyzeDataTool {
    const NAME: &'static str = "analyze_data";
    type Error = ToolError;
    type Args = AnalyzeDataInput;
    type Output = AnalyzeDataOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "对已读取的 Excel 数据按指定维度做聚合统计。可用维度: monthly_trend(月度趋势), supplier_ranking(供应商排行), dept_distribution(部门分布), buyer_performance(采购员业绩), top_products(商品Top15), note_analysis(备注/类型分析), quantity_analysis(数量分析)。传入字符串数组。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "之前 read_excel 使用的目录路径"
                    },
                    "dimensions": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "分析维度，如 [\"monthly_trend\", \"supplier_ranking\", \"top_products\"]"
                    }
                },
                "required": ["directory", "dimensions"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let cached = excel_cache::cache_get(&args.directory)
            .ok_or_else(|| ToolError::ExecutionError(
                format!("未找到缓存数据。请先调用 read_excel 读取目录: {}", args.directory)
            ))?;

        if cached.rows.is_empty() {
            return Err(ToolError::ExecutionError("Excel 数据为空".to_string()));
        }

        let col_map = ColumnMapping::detect(&cached.columns);
        let total = cached.rows.len();
        let total_amount = sum_f64(&cached.rows, &col_map.amount);

        let mut report = ReportData {
            title: "数据分析报告".to_string(),
            subtitle: format!(
                "数据范围: {} | 共 {} 条记录 | {} 个文件 | 报告生成日期: {}",
                date_range(&cached.rows, &col_map.date),
                total,
                cached.file_names.len(),
                chrono::Local::now().format("%Y年%m月%d日"),
            ),
            footer: "本报告由 DiskTidy 自动生成".to_string(),
            stats: vec![],
            monthly_trend: None,
            supplier_ranking: None,
            dept_distribution: None,
            buyer_performance: None,
            top_products: None,
            note_analysis: None,
            quantity_analysis: None,
        };

        // Stats
        let mut stats = vec![
            StatItem { value: format!("¥{:.2}万", total_amount / 10000.0), label: "总金额".to_string() },
            StatItem { value: total.to_string(), label: "总记录数".to_string() },
        ];
        if !col_map.supplier.is_empty() {
            let n = count_unique(&cached.rows, &col_map.supplier);
            stats.push(StatItem { value: n.to_string(), label: "供应商数".to_string() });
        }
        if !col_map.product.is_empty() {
            let n = count_unique(&cached.rows, &col_map.product);
            stats.push(StatItem { value: n.to_string(), label: "商品种类".to_string() });
        }
        if !col_map.buyer.is_empty() {
            let n = count_unique(&cached.rows, &col_map.buyer);
            stats.push(StatItem { value: n.to_string(), label: "采购员数".to_string() });
        }
        if !col_map.dept.is_empty() {
            let n = count_unique(&cached.rows, &col_map.dept);
            stats.push(StatItem { value: n.to_string(), label: "部门数".to_string() });
        }
        report.stats = stats;

        // 按维度分析
        for dim in &args.dimensions {
            match dim.as_str() {
                "monthly_trend" => {
                    if !col_map.date.is_empty() && !col_map.amount.is_empty() {
                        let (labels, values, counts) = monthly_agg(&cached.rows, &col_map.date, &col_map.amount);
                        report.monthly_trend = Some(TrendData { labels, values, counts });
                    }
                }
                "supplier_ranking" => {
                    if !col_map.supplier.is_empty() && !col_map.amount.is_empty() {
                        let (names, values) = group_sum_top(&cached.rows, &col_map.supplier, &col_map.amount, 10);
                        report.supplier_ranking = Some(RankingData { names, values });
                    }
                }
                "dept_distribution" => {
                    if !col_map.dept.is_empty() && !col_map.amount.is_empty() {
                        let data = group_sum_pie(&cached.rows, &col_map.dept, &col_map.amount);
                        report.dept_distribution = Some(PieData { data });
                    }
                }
                "buyer_performance" => {
                    if !col_map.buyer.is_empty() && !col_map.amount.is_empty() {
                        let (names, values, counts) = buyer_agg(&cached.rows, &col_map.buyer, &col_map.amount);
                        report.buyer_performance = Some(PerfData { names, values, counts });
                    }
                }
                "top_products" => {
                    if !col_map.product.is_empty() && !col_map.amount.is_empty() {
                        let (names, values) = group_sum_top(&cached.rows, &col_map.product, &col_map.amount, 15);
                        report.top_products = Some(RankingData { names, values });
                    }
                }
                "note_analysis" => {
                    if !col_map.note.is_empty() && !col_map.amount.is_empty() {
                        let data = group_sum_pie(&cached.rows, &col_map.note, &col_map.amount);
                        report.note_analysis = Some(PieData { data });
                    }
                }
                "quantity_analysis" => {
                    if !col_map.qty.is_empty() && !col_map.product.is_empty() {
                        let (names, values) = group_sum_top(&cached.rows, &col_map.product, &col_map.qty, 12);
                        report.quantity_analysis = Some(RankingData { names, values });
                    }
                }
                _ => {}
            }
        }

        // 生成摘要
        let mut summary_parts = vec![format!("共 {} 条记录，总金额 ¥{:.2}万", total, total_amount / 10000.0)];
        if let Some(ref m) = report.monthly_trend {
            if m.values.iter().any(|&v| v > 0.0) {
                let max_v = m.values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                if let Some(max_l) = m.labels.iter().max_by_key(|l| {
                    let idx = m.labels.iter().position(|x| x == *l).unwrap_or(0);
                    (m.values.get(idx).copied().unwrap_or(0.0) * 100.0) as i64
                }) {
                    summary_parts.push(format!("{} 金额最高: ¥{:.2}万", max_l, max_v / 10000.0));
                }
            }
        }
        if let Some(ref s) = report.supplier_ranking {
            if let (Some(first), Some(val)) = (s.names.first(), s.values.first()) {
                summary_parts.push(format!("Top1 供应商: {} (¥{:.2}万)", first, val / 10000.0));
            }
        }

        let summary = summary_parts.join("；");

        // 将完整的 report 存入缓存，避免通过 LLM 传递大 JSON
        cache_set_report(&args.directory, report.clone());
        let dimension_count = args.dimensions.len();

        Ok(AnalyzeDataOutput {
            report: Some(report),
            summary,
            dimension_count,
            total_rows: total,
            _prompt: self.detailed_prompt().to_string(),
        })
    }
}

// ==================== 报告数据结构 ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportData {
    pub title: String,
    pub subtitle: String,
    pub footer: String,
    pub stats: Vec<StatItem>,
    pub monthly_trend: Option<TrendData>,
    pub supplier_ranking: Option<RankingData>,
    pub dept_distribution: Option<PieData>,
    pub buyer_performance: Option<PerfData>,
    pub top_products: Option<RankingData>,
    pub note_analysis: Option<PieData>,
    pub quantity_analysis: Option<RankingData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatItem {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendData {
    pub labels: Vec<String>,
    pub values: Vec<f64>,
    pub counts: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingData {
    pub names: Vec<String>,
    pub values: Vec<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PieData {
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfData {
    pub names: Vec<String>,
    pub values: Vec<f64>,
    pub counts: Vec<usize>,
}

// ==================== 列名匹配 ====================

struct ColumnMapping {
    date: String,
    amount: String,
    supplier: String,
    dept: String,
    buyer: String,
    product: String,
    note: String,
    qty: String,
}

impl ColumnMapping {
    fn detect(headers: &[String]) -> Self {
        ColumnMapping {
            date: find_col(headers, &["日期", "date", "时间"]),
            amount: find_col(headers, &["金额", "amount", "总价", "合计", "总金额"]),
            supplier: find_col(headers, &["供应商", "supplier", "供货"]),
            dept: find_col(headers, &["部门", "dept", "品类", "区域"]),
            buyer: find_col(headers, &["采购员", "buyer", "经办人", "负责人", "员工"]),
            product: find_col(headers, &["商品", "product", "名称", "品名", "货品", "物品"]),
            note: find_col(headers, &["备注", "note", "类型", "说明", "分类"]),
            qty: find_col(headers, &["数量", "qty", "quantity", "采购数量", "件数"]),
        }
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

// ==================== 聚合函数 ====================

fn get_val(row: &HashMap<String, String>, col: &str) -> String {
    row.get(col).cloned().unwrap_or_default()
}

fn parse_f64(s: &str) -> f64 {
    let s = s.trim().trim_start_matches('¥').replace(',', "");
    s.parse::<f64>().unwrap_or(0.0)
}

fn sum_f64(rows: &[HashMap<String, String>], col: &str) -> f64 {
    if col.is_empty() { return 0.0; }
    rows.iter().map(|r| parse_f64(&get_val(r, col))).sum()
}

fn count_unique(rows: &[HashMap<String, String>], col: &str) -> usize {
    if col.is_empty() { return 0; }
    rows.iter()
        .map(|r| get_val(r, col))
        .filter(|v| !v.is_empty())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

fn date_range(rows: &[HashMap<String, String>], col: &str) -> String {
    if col.is_empty() { return "全部".to_string(); }
    let dates: Vec<String> = rows.iter()
        .map(|r| get_val(r, col))
        .filter(|v| !v.is_empty())
        .collect();
    if dates.is_empty() { return "全部".to_string(); }
    let empty = String::new();
    let min_d = dates.iter().min().unwrap_or(&empty);
    let max_d = dates.iter().max().unwrap_or(&empty);
    format!("{} - {}", min_d, max_d)
}

fn extract_month(date_str: &str) -> String {
    let s = date_str.trim();
    if s.len() >= 7 {
        if s.chars().nth(4) == Some('-') || s.chars().nth(4) == Some('/') {
            return s[..7].to_string();
        }
        if s.contains('年') && s.contains('月') {
            let y_end = s.find('年').unwrap_or(4);
            let m_end = s.find('月').unwrap_or(y_end + 4);
            return format!("{}年{}月", &s[..y_end], &s[y_end+3..m_end]);
        }
    }
    s.chars().take(7).collect()
}

fn monthly_agg(rows: &[HashMap<String, String>], date_col: &str, amount_col: &str)
    -> (Vec<String>, Vec<f64>, Vec<usize>) {
    let mut map: HashMap<String, (f64, usize)> = HashMap::new();
    for row in rows {
        let month_key = extract_month(&get_val(row, date_col));
        let amount = parse_f64(&get_val(row, amount_col));
        let e = map.entry(month_key).or_insert((0.0, 0));
        e.0 += amount;
        e.1 += 1;
    }
    let mut items: Vec<_> = map.into_iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    (items.iter().map(|i| i.0.clone()).collect(),
     items.iter().map(|i| i.1.0).collect(),
     items.iter().map(|i| i.1.1).collect())
}

fn group_sum_top(rows: &[HashMap<String, String>], group_col: &str, value_col: &str, top_n: usize)
    -> (Vec<String>, Vec<f64>) {
    let mut map: HashMap<String, f64> = HashMap::new();
    for row in rows {
        let key = get_val(row, group_col);
        if key.is_empty() { continue; }
        *map.entry(key).or_insert(0.0) += parse_f64(&get_val(row, value_col));
    }
    let mut items: Vec<_> = map.into_iter().collect();
    items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    items.truncate(top_n);
    (items.iter().map(|i| i.0.clone()).collect(),
     items.iter().map(|i| i.1).collect())
}

fn group_sum_pie(rows: &[HashMap<String, String>], group_col: &str, value_col: &str)
    -> Vec<serde_json::Value> {
    let mut map: HashMap<String, f64> = HashMap::new();
    for row in rows {
        let key = get_val(row, group_col);
        if key.is_empty() { continue; }
        *map.entry(key).or_insert(0.0) += parse_f64(&get_val(row, value_col));
    }
    map.into_iter()
        .map(|(name, value)| serde_json::json!({ "name": name, "value": value }))
        .collect()
}

fn buyer_agg(rows: &[HashMap<String, String>], buyer_col: &str, amount_col: &str)
    -> (Vec<String>, Vec<f64>, Vec<usize>) {
    let mut map: HashMap<String, (f64, usize)> = HashMap::new();
    for row in rows {
        let key = get_val(row, buyer_col);
        if key.is_empty() { continue; }
        let amount = parse_f64(&get_val(row, amount_col));
        let e = map.entry(key).or_insert((0.0, 0));
        e.0 += amount;
        e.1 += 1;
    }
    let mut items: Vec<_> = map.into_iter().collect();
    items.sort_by(|a, b| b.1.0.partial_cmp(&a.1.0).unwrap_or(std::cmp::Ordering::Equal));
    (items.iter().map(|i| i.0.clone()).collect(),
     items.iter().map(|i| i.1.0).collect(),
     items.iter().map(|i| i.1.1).collect())
}
