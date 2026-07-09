use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::UNIX_EPOCH;

use super::tool_error::ToolError;
use super::ToolPrompt;

// ── input ──
#[derive(Debug, Deserialize)]
pub struct ScanDesktopInput {
    pub directory: String,
    #[serde(default)]
    pub extensions_filter: Vec<String>,
}

// ── output ──
#[derive(Debug, Serialize)]
pub struct ScanDesktopOutput {
    pub directory: String,
    pub total_files: usize,
    pub total_size_bytes: u64,
    /// 扩展名分布，如 {"txt": 30, "pdf": 12}
    pub extension_counts: HashMap<String, usize>,
    /// 当文件 ≤ 20 时，返回完整清单
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileEntry>>,
    /// 当文件 > 20 时，返回前缀聚类摘要（如 "log_2026": {count: 50, samples: [...]}）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clusters: Option<Vec<FileCluster>>,
    /// 少量文件特殊提示
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    pub _prompt: String,
}

#[derive(Debug, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub size_bytes: u64,
    /// 人类可读的大小
    pub size_human: String,
    /// 修改日期 YYYY-MM-DD
    pub modified_date: String,
}

#[derive(Debug, Serialize)]
pub struct FileCluster {
    /// 前缀（如 "log_", "weekly_report_"）
    pub prefix: String,
    /// 文件数量
    pub count: usize,
    /// 总大小
    pub total_size_bytes: u64,
    /// 最多 3 个样本文件名
    pub samples: Vec<String>,
}

// ── 内部扫描结果 ──
struct RawScan {
    total_size: u64,
    files: Vec<(String, u64, String)>, // (name, size_bytes, modified_date)
}

pub struct ScanDesktopTool;

impl ScanDesktopTool {
    pub fn new() -> Self { Self }
}

impl ToolPrompt for ScanDesktopTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 扫描完成
下一步：根据文件清单判断是否需要整理。
- 文件很少（≤3个）→ 直接告知用户"文件很少，不需要整理"
- 文件较多 → 自主决定分类维度（按类型/日期/场景），调用 organize_files 执行

分类维度示例：
- 全是日志类 → 按日期分：2026-01/ 2026-02/ ...
- 工作+个人 → 按场景分：工作/ 个人/
- 散乱命名 → 按修改时间分：本周/ 本月/ 更早/
- 无法判断 → 统一放进 "桌面整理归档/"
"#
    }
}

impl rig_core::tool::Tool for ScanDesktopTool {
    const NAME: &'static str = "scan_desktop";
    type Error = ToolError;
    type Args = ScanDesktopInput;
    type Output = ScanDesktopOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "扫描目录（仅根目录，不递归子文件夹），返回文件清单。文件多时自动按前缀聚类以减少上下文。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "要扫描的目录绝对路径"
                    },
                    "extensions_filter": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "可选，只扫描指定扩展名的文件，如 [\"txt\", \"pdf\"]"
                    }
                },
                "required": ["directory"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let directory = args.directory.clone();
        let dir_for_output = directory.clone();
        let ext_filter: Vec<String> = args.extensions_filter.iter()
            .map(|e| e.trim_start_matches('.').to_lowercase())
            .collect();

        let raw = tokio::task::spawn_blocking(move || -> Result<RawScan, String> {
            let dir = Path::new(&directory);
            if !dir.is_dir() {
                return Err(format!("目录不存在: {}", directory));
            }

            let mut entries: Vec<_> = std::fs::read_dir(dir)
                .map_err(|e| format!("读取目录失败: {}", e))?
                .filter_map(|e| e.ok())
                .filter(|e| {
                    // 跳过隐藏文件和目录
                    if let Some(name) = e.file_name().to_str() {
                        if name.starts_with('.') || name.starts_with('$') {
                            return false;
                        }
                    }
                    // 只处理文件（max_depth=1 硬编码）
                    if let Ok(ft) = e.file_type() {
                        ft.is_file()
                    } else {
                        false
                    }
                })
                .collect();

            // 扩展名过滤
            if !ext_filter.is_empty() {
                entries.retain(|e| {
                    e.path().extension()
                        .and_then(|ex| ex.to_str())
                        .map(|ex| ext_filter.contains(&ex.to_lowercase()))
                        .unwrap_or(false)
                });
            }

            let mut files = Vec::new();
            let mut total_size = 0u64;

            for entry in entries {
                let name = entry.file_name().to_string_lossy().to_string();
                let mut size = 0u64;
                let mut modified = String::new();

                if let Ok(meta) = entry.metadata() {
                    size = meta.len();
                    total_size += size;
                    modified = meta.modified()
                        .ok()
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| {
                            let secs = d.as_secs();
                            let days = secs / 86400;
                            // 简易时间戳转日期（足够精确）
                            let year = 1970 + (days / 365) as i32;
                            let remainder = days % 365;
                            let month = (remainder / 30 + 1) as u32;
                            let day = (remainder % 30 + 1) as u32;
                            format!("{:04}-{:02}-{:02}", year, month.min(12), day.min(28))
                        })
                        .unwrap_or_default();
                }

                files.push((name, size, modified));
            }

            Ok(RawScan { total_size, files })
        })
        .await
        .map_err(|e| ToolError::ExecutionError(format!("扫描失败: {}", e)))?
        .map_err(ToolError::ExecutionError)?;

        let total_files = raw.files.len();

        // 扩展名统计
        let mut extension_counts: HashMap<String, usize> = HashMap::new();
        for (name, ..) in &raw.files {
            let ext = Path::new(name).extension()
                .and_then(|e| e.to_str())
                .unwrap_or("无扩展名")
                .to_lowercase();
            *extension_counts.entry(ext).or_insert(0) += 1;
        }

        // 少量文件（≤20）→ 直接返回完整清单
        if total_files <= 20 {
            let files: Vec<FileEntry> = raw.files.iter().map(|(name, size, modified)| {
                FileEntry {
                    name: name.clone(),
                    size_bytes: *size,
                    size_human: format_size(*size),
                    modified_date: modified.clone(),
                }
            }).collect();

            let hint = if total_files <= 3 {
                Some(format!("只有 {} 个文件，可以判断是否需要整理。如果不需要，直接告知用户。", total_files))
            } else {
                None
            };

            return Ok(ScanDesktopOutput {
                directory: dir_for_output,
                total_files,
                total_size_bytes: raw.total_size,
                extension_counts,
                files: Some(files),
                clusters: None,
                hint,
                _prompt: Self.detailed_prompt().to_string(),
            });
        }

        // 多文件（>20）→ 按前缀聚类
        let clusters = build_clusters(&raw.files);

        Ok(ScanDesktopOutput {
            directory: dir_for_output,
            total_files,
            total_size_bytes: raw.total_size,
            extension_counts,
            files: None,
            clusters: Some(clusters),
            hint: None,
            _prompt: Self.detailed_prompt().to_string(),
        })
    }
}

/// 按文件名前缀聚类（取公共前缀前 3+ 字符，至少 3 个文件才聚类）
fn build_clusters(files: &[(String, u64, String)]) -> Vec<FileCluster> {
    // 按扩展名分组后，再按前缀聚类
    let mut by_ext: HashMap<String, Vec<&(String, u64, String)>> = HashMap::new();
    for f in files {
        let ext = Path::new(&f.0).extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        by_ext.entry(ext).or_default().push(f);
    }

    let mut clusters: Vec<FileCluster> = Vec::new();
    let mut clustered_names: std::collections::HashSet<String> = std::collections::HashSet::new();

    for (ext, group) in &by_ext {
        if group.len() < 3 {
            // 太少不聚类，作为独立文件
            continue;
        }

        // 按前缀分组（取前 3-10 个字符作为前缀）
        let mut prefix_map: HashMap<String, Vec<&(String, u64, String)>> = HashMap::new();
        for f in group {
            let stem = Path::new(&f.0).file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(&f.0);
            // 尝试逐步缩短前缀
            let prefix = find_best_prefix(stem, 3);
            prefix_map.entry(prefix).or_default().push(f);
        }

        for (prefix, items) in &prefix_map {
            if items.len() < 3 {
                continue;
            }

            let count = items.len();
            let total_size_bytes: u64 = items.iter().map(|f| f.1).sum();
            let samples: Vec<String> = items.iter().take(3).map(|f| f.0.clone()).collect();

            for f in items {
                clustered_names.insert(f.0.clone());
            }

            let display_prefix = if ext.is_empty() {
                format!("{}_*", prefix)
            } else {
                format!("{}_*.{}", prefix, ext)
            };

            clusters.push(FileCluster {
                prefix: display_prefix,
                count,
                total_size_bytes,
                samples,
            });
        }
    }

    // 未聚类的单独文件作为"其他"聚类
    let unclustered: Vec<&(String, u64, String)> = files.iter()
        .filter(|f| !clustered_names.contains(&f.0))
        .collect();
    if unclustered.len() >= 3 {
        let total_size_bytes: u64 = unclustered.iter().map(|f| f.1).sum();
        clusters.push(FileCluster {
            prefix: "其他".to_string(),
            count: unclustered.len(),
            total_size_bytes,
            samples: unclustered.iter().take(3).map(|f| f.0.clone()).collect(),
        });
    }

    clusters.sort_by(|a, b| b.count.cmp(&a.count));
    clusters
}

/// 找最佳前缀：从全名开始逐步缩短，直到找到一组中的公共前缀
fn find_best_prefix(name: &str, min_len: usize) -> String {
    // 分离字母/数字/其他部分
    let chars: Vec<char> = name.chars().collect();
    let mut pos = 0;

    // 跳过日期前缀 (2026-01-01_ etc)
    while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '-' || chars[pos] == '_') {
        pos += 1;
    }

    if pos >= chars.len() {
        // 纯数字/日期，用全名
        return name.chars().take(10.min(name.chars().count())).collect();
    }

    // 从跳过后位置开始，取到下一个分隔符或达到合理长度
    let start = pos;
    let mut end = start + 5.min(chars.len() - start);

    // 尝试在分隔符处截断
    for i in start..(start + 8).min(chars.len()) {
        if chars[i] == '_' || chars[i] == '-' || chars[i] == ' ' {
            end = i;
            break;
        }
    }

    let result: String = chars[start..end].iter().collect();
    if result.chars().count() >= min_len {
        result
    } else {
        chars[start..(start + min_len).min(chars.len())].iter().collect()
    }
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
