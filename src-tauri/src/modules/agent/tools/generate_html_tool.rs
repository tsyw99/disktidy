use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::modules::agent::tools::tool_error::ToolError;
use super::ToolPrompt;
use super::analyze_data_tool::ReportData;
use super::excel_cache::{cache_get_report, cache_clear};

#[derive(Debug, Deserialize)]
pub struct GenerateHtmlInput {
    /// analyze_data 返回的 report 字段（JSON 对象，兼容旧方式）
    #[serde(default)]
    pub report: Option<serde_json::Value>,
    /// 缓存键（与 read_excel/analyze_data 的 directory 相同，推荐使用此方式）
    #[serde(default)]
    pub report_key: Option<String>,
    /// HTML 保存路径（绝对路径，含文件名）
    pub output_path: String,
}

#[derive(Debug, Serialize)]
pub struct GenerateHtmlOutput {
    /// 报告标题
    pub title: String,
    /// 保存路径
    pub saved_path: String,
    /// 专属提示词
    pub _prompt: String,
}

impl ToolPrompt for GenerateHtmlTool {
    fn detailed_prompt(&self) -> &'static str {
        "报告已生成完毕并向用户展示保存路径。\n\n\
         请在回复中主动建议用户开启新对话（点击「清空对话」按钮），\
         并简要说明原因：当前对话中累积了大量文件分析数据，\
         会占用较多上下文，开启新对话可以让后续交互更快速稳定。"
    }
}

pub struct GenerateHtmlTool;

impl GenerateHtmlTool {
    pub fn new() -> Self { Self }
}

impl rig_core::tool::Tool for GenerateHtmlTool {
    const NAME: &'static str = "generate_html";
    type Error = ToolError;
    type Args = GenerateHtmlInput;
    type Output = GenerateHtmlOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "将分析结果渲染为包含 ECharts 图表的自包含 HTML 报告文件。推荐传入 report_key（目录路径）从缓存读取报告数据，避免传递大 JSON。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "report_key": {
                        "type": "string",
                        "description": "目录路径（与 read_excel 的 directory 相同），工具会从缓存读取报告数据。这是推荐方式。"
                    },
                    "report": {
                        "type": "object",
                        "description": "analyze_data 返回的 report JSON（仅在无法用 report_key 时使用）"
                    },
                    "output_path": {
                        "type": "string",
                        "description": "HTML 报告保存的绝对路径，含 .html 文件名"
                    }
                },
                "required": ["output_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        // 优先从缓存读取报告（推荐方式），否则从 JSON 参数解析（兼容旧方式）
        let report: ReportData = if let Some(ref key) = args.report_key {
            cache_get_report(key)
                .ok_or_else(|| ToolError::ExecutionError(
                    format!("缓存中未找到报告数据，请确保已先执行 analyze_data (key={})", key)
                ))?
        } else if let Some(ref json_value) = args.report {
            serde_json::from_value(json_value.clone())
                .map_err(|e| ToolError::ExecutionError(format!("report JSON 解析失败: {}", e)))?
        } else {
            return Err(ToolError::ExecutionError(
                "请提供 report_key（推荐）或 report JSON 参数".to_string()
            ));
        };

        let output_path = args.output_path.clone();
        let title = report.title.clone();

        // 在独立线程中执行同步操作（HTML 生成 + 文件写入），避免阻塞 tokio 运行时
        let saved_path = tokio::task::spawn_blocking(move || -> Result<String, ToolError> {
            let html = build_html(&report);

            let path = Path::new(&output_path);
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| ToolError::ExecutionError(format!("创建目录失败: {}", e)))?;
                }
            }

            std::fs::write(path, &html)
                .map_err(|e| ToolError::ExecutionError(format!("保存报告失败: {}", e)))?;

            let canonical = std::fs::canonicalize(path)
                .unwrap_or_else(|_| path.to_path_buf());

            Ok(canonical.to_string_lossy().to_string())
        })
        .await
        .map_err(|e| ToolError::ExecutionError(format!("HTML 生成线程异常: {}", e)))??;

        // 报告生成成功后，清除 Excel 缓存以释放内存
        if let Some(ref key) = args.report_key {
            cache_clear(key);
        }

        Ok(GenerateHtmlOutput {
            title,
            saved_path,
            _prompt: String::new(),
        })
    }
}

// ==================== HTML 生成 ====================

fn build_html(report: &ReportData) -> String {
    let data_json = serde_json::to_string(report).unwrap_or_default();

    let stats_html: String = report.stats.iter().map(|s| {
        format!(
            r#"<div class="stat-card"><div class="value{}">{}</div><div class="label">{}</div></div>"#,
            if s.value.len() > 8 { " small" } else { "" },
            s.value, s.label
        )
    }).collect::<Vec<_>>().join("\n");

    let sections_html = build_chart_sections(report);

    let scripts_html = build_chart_scripts(report);

    format!(
        r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title}</title>
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
  <div class="stats">{stats_html}</div>
  {sections_html}
</div>
<div class="footer">{footer}</div>
<script>
var DATA = {data_json};
var colors = ['#5470c6','#91cc75','#fac858','#ee6666','#73c0de','#3ba272','#fc8452','#9a60b4','#ea7ccc','#48b8d0'];
{scripts_html}
window.addEventListener('resize', function() {{
  document.querySelectorAll('.chart-inner').forEach(function(el) {{
    var instance = echarts.getInstanceByDom(el);
    if (instance) instance.resize();
  }});
}});
</script>
</body>
</html>"#,
        title = report.title,
        subtitle = report.subtitle,
        stats_html = stats_html,
        sections_html = sections_html,
        data_json = data_json,
        footer = report.footer,
        scripts_html = scripts_html,
    )
}

fn build_chart_sections(report: &ReportData) -> String {
    let mut sections = Vec::new();

    // 月度趋势
    if report.monthly_trend.is_some() {
        sections.push(chart_section("月度趋势", true, vec![
            ("monthly", "chart-inner tall")
        ]));
    }

    // 供应商 + 部门
    let has_supplier = report.supplier_ranking.is_some();
    let has_dept = report.dept_distribution.is_some();
    if has_supplier && has_dept {
        sections.push(chart_section("供应商排行 / 部门分布", false, vec![
            ("supplier", "chart-inner"),
            ("dept", "chart-inner"),
        ]));
    } else if has_supplier {
        sections.push(chart_section("供应商排行", false, vec![
            ("supplier", "chart-inner"),
        ]));
    }

    // 采购员业绩
    if report.buyer_performance.is_some() {
        sections.push(chart_section("采购员业绩", true, vec![
            ("buyer", "chart-inner tall")
        ]));
    }

    // 商品 + 备注
    let has_product = report.top_products.is_some();
    let has_note = report.note_analysis.is_some();
    if has_product && has_note {
        sections.push(chart_section("商品分析 / 类型分布", false, vec![
            ("product", "chart-inner"),
            ("note", "chart-inner"),
        ]));
    } else if has_product {
        sections.push(chart_section("商品 Top 15", false, vec![
            ("product", "chart-inner"),
        ]));
    }

    // 数量分析
    if report.quantity_analysis.is_some() {
        sections.push(chart_section("采购数量排行", false, vec![
            ("qty", "chart-inner"),
        ]));
    }

    sections.join("\n")
}

fn chart_section(title: &str, full: bool, panels: Vec<(&str, &str)>) -> String {
    let row_class = if full { "chart-row full" } else { "chart-row" };
    let cards: String = panels.iter().map(|(id, cls)| {
        format!(r#"<div class="chart-card"><div class="{}" id="chart-{}"></div></div>"#, cls, id)
    }).collect::<Vec<_>>().join("\n");

    format!(
        r#"<div class="section"><div class="section-title">{}</div><div class="{}">{}</div></div>"#,
        title, row_class, cards
    )
}

fn build_chart_scripts(report: &ReportData) -> String {
    let mut scripts = Vec::new();

    // Monthly trend
    if let Some(ref m) = report.monthly_trend {
        let ml = serde_json::to_string(&m.labels).unwrap_or_default();
        let mv = serde_json::to_string(&m.values).unwrap_or_default();
        let mc = serde_json::to_string(&m.counts).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-monthly')).setOption({{
  tooltip: {{ trigger:'axis', axisPointer:{{ type:'cross' }} }},
  legend: {{ data:['金额','记录数'], top:5 }},
  grid: {{ left:70, right:70, top:50, bottom:40 }},
  xAxis: {{ type:'category', data:{ml}, axisLabel:{{ rotate:30 }} }},
  yAxis: [
    {{ type:'value', name:'金额(元)', axisLabel:{{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
    {{ type:'value', name:'记录数' }}
  ],
  series: [
    {{ name:'金额', type:'line', smooth:true, data:{mv},
       itemStyle:{{ color:'#5470c6' }},
       areaStyle:{{ color: new echarts.graphic.LinearGradient(0,0,0,1,[{{offset:0,color:'rgba(84,112,198,0.3)'}},{{offset:1,color:'rgba(84,112,198,0.02)'}}]) }},
       markLine:{{ silent:true, data:[{{ type:'average', name:'均值' }}], label:{{ formatter:'均值\\n¥{{c}}' }} }}
    }},
    {{ name:'记录数', type:'bar', yAxisIndex:1, data:{mc},
       itemStyle:{{ color:'#91cc75', borderRadius:[4,4,0,0] }}
    }}
  ]
}});"#));
    }

    // Supplier ranking
    if let Some(ref s) = report.supplier_ranking {
        let sn = serde_json::to_string(&s.names).unwrap_or_default();
        let sv = serde_json::to_string(&s.values).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-supplier')).setOption({{
  tooltip: {{ trigger:'axis', axisPointer:{{ type:'shadow' }} }},
  grid: {{ left:140, right:50, top:20, bottom:20 }},
  xAxis: {{ type:'value', axisLabel:{{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
  yAxis: {{ type:'category', data:{sn}, inverse:true, axisLabel:{{ fontSize:11 }} }},
  series: [{{ type:'bar',
    data: {sv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
    label: {{ show:true, position:'right', formatter: p => '¥'+(p.value/10000).toFixed(1)+'万', fontSize:10 }},
    barMaxWidth: 26
  }}]
}});"#));
    }

    // Department pie
    if let Some(ref d) = report.dept_distribution {
        let dd = serde_json::to_string(&d.data).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-dept')).setOption({{
  tooltip: {{ trigger:'item', formatter:'{{b}}: ¥{{c}} ({{d}}%)' }},
  legend: {{ orient:'vertical', right:10, top:'center', itemWidth:12, itemHeight:12 }},
  series: [{{ type:'pie', radius:['40%','70%'], center:['40%','50%'],
    data:{dd}, label:{{ formatter:'{{b}}\\n{{d}}%' }},
    itemStyle:{{ borderRadius:4, borderColor:'#fff', borderWidth:2 }},
    emphasis:{{ itemStyle:{{ shadowBlur:10, shadowOffsetX:0, shadowColor:'rgba(0,0,0,0.3)' }} }}
  }}]
}});"#));
    }

    // Buyer performance
    if let Some(ref b) = report.buyer_performance {
        let bn = serde_json::to_string(&b.names).unwrap_or_default();
        let bv = serde_json::to_string(&b.values).unwrap_or_default();
        let bc = serde_json::to_string(&b.counts).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-buyer')).setOption({{
  tooltip: {{ trigger:'axis', axisPointer:{{ type:'shadow' }} }},
  legend: {{ data:['金额','记录数'], top:5 }},
  grid: {{ left:60, right:60, top:40, bottom:30 }},
  xAxis: {{ type:'category', data:{bn} }},
  yAxis: [
    {{ type:'value', name:'金额(元)', axisLabel:{{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
    {{ type:'value', name:'记录数' }}
  ],
  series: [
    {{ name:'金额', type:'bar',
       data: {bv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
       label:{{ show:true, position:'top', formatter: p => '¥'+(p.value/10000).toFixed(1)+'万', fontSize:9 }},
       barMaxWidth:38
    }},
    {{ name:'记录数', type:'line', yAxisIndex:1, data:{bc},
       itemStyle:{{ color:'#ee6666' }}, symbolSize:8, lineStyle:{{ width:2 }}
    }}
  ]
}});"#));
    }

    // Product top
    if let Some(ref p) = report.top_products {
        let pn = serde_json::to_string(&p.names).unwrap_or_default();
        let pv = serde_json::to_string(&p.values).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-product')).setOption({{
  tooltip: {{ trigger:'axis', axisPointer:{{ type:'shadow' }} }},
  grid: {{ left:120, right:50, top:20, bottom:20 }},
  xAxis: {{ type:'value', axisLabel:{{ formatter: v => (v/10000).toFixed(0)+'万' }} }},
  yAxis: {{ type:'category', data:{pn}, inverse:true, axisLabel:{{ fontSize:11 }} }},
  series: [{{ type:'bar',
    data: {pv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
    label:{{ show:true, position:'right', formatter: p => '¥'+(p.value/10000).toFixed(1)+'万', fontSize:10 }},
    barMaxWidth:20
  }}]
}});"#));
    }

    // Note pie
    if let Some(ref n) = report.note_analysis {
        let nd = serde_json::to_string(&n.data).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-note')).setOption({{
  tooltip: {{ trigger:'item', formatter:'{{b}}: ¥{{c}} ({{d}}%)' }},
  legend: {{ orient:'vertical', right:10, top:'center', itemWidth:12, itemHeight:12 }},
  series: [{{ type:'pie', radius:['40%','70%'], center:['40%','50%'],
    data:{nd}, label:{{ formatter:'{{b}}\\n{{d}}%' }},
    itemStyle:{{ borderRadius:4, borderColor:'#fff', borderWidth:2 }}
  }}]
}});"#));
    }

    // Quantity
    if let Some(ref q) = report.quantity_analysis {
        let qn = serde_json::to_string(&q.names).unwrap_or_default();
        let qv = serde_json::to_string(&q.values).unwrap_or_default();
        scripts.push(format!(r#"echarts.init(document.getElementById('chart-qty')).setOption({{
  tooltip: {{ trigger:'axis', axisPointer:{{ type:'shadow' }} }},
  grid: {{ left:120, right:50, top:20, bottom:20 }},
  xAxis: {{ type:'value' }},
  yAxis: {{ type:'category', data:{qn}, inverse:true, axisLabel:{{ fontSize:11 }} }},
  series: [{{ type:'bar',
    data: {qv}.map(function(v,i){{ return {{ value:v, itemStyle:{{color:colors[i%colors.length]}} }}; }}),
    label:{{ show:true, position:'right', fontSize:10 }},
    barMaxWidth:22
  }}]
}});"#));
    }

    scripts.join("\n")
}
