use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::tool_error::ToolError;
use super::ToolPrompt;

use log;

// Helper function to scan a directory for potential garbage files
fn scan_directory_for_garbage(
    dir: &std::path::Path,
    files: &mut Vec<crate::models::cleaner::GarbageFile>,
) -> Result<(), String> {
    use std::fs;
    use std::time::UNIX_EPOCH;
    use crate::models::cleaner::{GarbageCategory, GarbageFile, RiskLevel};

    // Common temporary file extensions
    const TEMP_EXTENSIONS: &[&str] = &["tmp", "temp", "bak", "old", "swp", "log", "cache"];

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            // 如果没有权限访问目录，记录日志并跳过
            log::warn!("无法读取目录 {}: {}", dir.display(), e);
            return Ok(());
        }
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        
        if path.is_file() {
            if let Ok(metadata) = entry.metadata() {
                let size = metadata.len();
                let modified_time = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);

                // Check if it's a potential garbage file based on extension
                if let Some(ext_os) = path.extension() {
                    if let Some(ext) = ext_os.to_str() {
                        let ext_lower = ext.to_lowercase();
                        if TEMP_EXTENSIONS.iter().any(|&e| e == ext_lower) {
                            // Determine risk level based on file characteristics
                            let risk_level = if size > 1_000_000_000 { // > 1GB
                                RiskLevel::High
                            } else if ext_lower == "exe" || ext_lower == "dll" || ext_lower == "sys" {
                                RiskLevel::Critical
                            } else if ext_lower == "tmp" || ext_lower == "temp" {
                                RiskLevel::Low
                            } else {
                                RiskLevel::Medium
                            };

                            let safe_to_delete = risk_level < RiskLevel::Critical;

                            files.push(GarbageFile {
                                path: path.to_string_lossy().to_string(),
                                size,
                                category: GarbageCategory::Other,
                                safe_to_delete,
                                risk_level,
                                modified_time,
                                accessed_time: modified_time, // Simplified
                            });
                        }
                    }
                }
            }
        } else if path.is_dir() {
            // Recursively scan subdirectories, but skip if permission denied
            if let Err(e) = scan_directory_for_garbage(&path, files) {
                log::warn!("无法扫描子目录 {}: {}", path.display(), e);
                // Continue with other directories
            }
        }
    }

    Ok(())
}

// ── input ──
#[derive(Debug, Deserialize)]
pub struct ScanGarbageInput {
    /// 要扫描的目录路径，如果为空则扫描系统默认垃圾目录
    #[serde(default)]
    pub directory: Option<String>,
    /// 是否包含系统临时文件
    #[serde(default)]
    pub include_system_temp: bool,
    /// 是否包含浏览器缓存
    #[serde(default)]
    pub include_browser_cache: bool,
    /// 是否包含应用缓存
    #[serde(default)]
    pub include_app_cache: bool,
    /// 是否包含回收站
    #[serde(default)]
    pub include_recycle_bin: bool,
    /// 是否包含日志文件
    #[serde(default)]
    pub include_log_files: bool,
    /// 最小文件年龄（天），只扫描超过此天数的文件
    #[serde(default)]
    pub min_file_age_days: Option<u32>,
}

// ── output ──
#[derive(Debug, Serialize)]
pub struct ScanGarbageOutput {
    pub scan_id: String,
    pub total_files: u64,
    pub total_size: u64,
    /// 按目录聚合的摘要
    pub by_directory: Vec<DirectorySummary>,
    /// 按扩展名聚合的摘要
    pub by_extension: Vec<ExtensionSummary>,
    /// 按时间聚合的摘要
    pub by_time: Vec<TimeSummary>,
    /// 高风险文件数量
    pub high_risk_count: u64,
    pub _prompt: String,
}

#[derive(Debug, Serialize)]
pub struct DirectorySummary {
    pub directory: String,
    pub file_count: u64,
    pub total_size: u64,
    pub primary_extensions: Vec<String>, // 前3个主要扩展名
    pub last_modified_desc: String,      // 最后修改描述
}

#[derive(Debug, Serialize)]
pub struct ExtensionSummary {
    pub extension: String,
    pub file_count: u64,
    pub total_size: u64,
    pub directory_count: u64, // 分布在多少个目录中
}

#[derive(Debug, Serialize)]
pub struct TimeSummary {
    pub time_period: String, // "over_2_years", "over_1_year", "over_6_months", etc.
    pub file_count: u64,
    pub total_size: u64,
}

pub struct ScanGarbageTool;

impl ScanGarbageTool {
    pub fn new() -> Self { Self }
}

impl ToolPrompt for ScanGarbageTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 垃圾文件扫描完成
分析扫描结果，按以下维度评估清理优先级：
- 安全清理：系统临时文件、浏览器缓存 → 可直接建议清理
- 需确认：下载目录的安装包、备份文件 → 建议用户确认
- 谨慎处理：日志文件、配置备份 → 询问用户是否需要

下一步：根据分析结果制定清理策略，调用 clean_garbage 执行清理。
"#
    }
}

impl rig_core::tool::Tool for ScanGarbageTool {
    const NAME: &'static str = "scan_garbage";
    type Error = ToolError;
    type Args = ScanGarbageInput;
    type Output = ScanGarbageOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "扫描系统垃圾文件并生成聚合摘要。按目录、扩展名、时间三个维度聚合，避免返回大量原始文件列表。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "directory": {
                        "type": "string",
                        "description": "要扫描的目录路径，如果为空则扫描系统默认垃圾目录"
                    },
                    "include_system_temp": {
                        "type": "boolean",
                        "description": "是否包含系统临时文件，默认为 false"
                    },
                    "include_browser_cache": {
                        "type": "boolean",
                        "description": "是否包含浏览器缓存，默认为 false"
                    },
                    "include_app_cache": {
                        "type": "boolean",
                        "description": "是否包含应用缓存，默认为 false"
                    },
                    "include_recycle_bin": {
                        "type": "boolean",
                        "description": "是否包含回收站，默认为 false"
                    },
                    "include_log_files": {
                        "type": "boolean",
                        "description": "是否包含日志文件，默认为 false"
                    },
                    "min_file_age_days": {
                        "type": "integer",
                        "description": "最小文件年龄（天），只扫描超过此天数的文件"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let scan_result = tokio::task::spawn_blocking(move || -> Result<ScanGarbageOutput, String> {
            // 如果指定了特定目录，则只扫描该目录，否则使用默认垃圾扫描
            if let Some(directory) = &args.directory {
                // 扫描指定目录下的垃圾文件
                let path = std::path::Path::new(directory);
                if !path.exists() {
                    return Err(format!("目录不存在: {}", directory));
                }
                
                // 扫描指定目录及其子目录中的常见垃圾文件
                let mut files: Vec<crate::models::cleaner::GarbageFile> = Vec::new();
                scan_directory_for_garbage(path, &mut files)?;
                
                // 聚合统计信息
                let mut dir_stats: HashMap<String, DirectorySummary> = HashMap::new();
                let mut ext_stats: HashMap<String, ExtensionSummary> = HashMap::new();
                let mut time_stats: HashMap<String, TimeSummary> = HashMap::new();
                
                let mut total_files = 0u64;
                let mut total_size = 0u64;
                let mut high_risk_count = 0u64;
                
                for file in &files {
                    total_files += 1;
                    total_size += file.size;
                    
                    if file.risk_level >= crate::models::cleaner::RiskLevel::High {
                        high_risk_count += 1;
                    }
                    
                    let path = Path::new(&file.path);
                    
                    // 按目录聚合
                    if let Some(parent) = path.parent() {
                        let dir = parent.to_string_lossy().to_string();
                        let entry = dir_stats.entry(dir.clone()).or_insert_with(|| DirectorySummary {
                            directory: dir,
                            file_count: 0,
                            total_size: 0,
                            primary_extensions: Vec::new(),
                            last_modified_desc: String::new(),
                        });
                        
                        entry.file_count += 1;
                        entry.total_size += file.size;
                    }

                    // 按扩展名聚合
                    if let Some(ext) = path.extension() {
                        let ext_str = format!(".{}", ext.to_string_lossy());
                        let ext_entry = ext_stats.entry(ext_str.clone()).or_insert_with(|| ExtensionSummary {
                            extension: ext_str,
                            file_count: 0,
                            total_size: 0,
                            directory_count: 0,
                        });
                        
                        ext_entry.file_count += 1;
                        ext_entry.total_size += file.size;
                    }

                    // 按时间聚合
                    if file.modified_time > 0 {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64;
                        let age_days = (now - file.modified_time) / (24 * 3600);
                        
                        let time_period = if age_days >= 365 * 2 {
                            "over_2_years".to_string()
                        } else if age_days >= 365 {
                            "over_1_year".to_string()
                        } else if age_days >= 180 {
                            "over_6_months".to_string()
                        } else if age_days >= 90 {
                            "over_3_months".to_string()
                        } else if age_days >= 30 {
                            "over_1_month".to_string()
                        } else {
                            "recent".to_string()
                        };
                        
                        let period_clone = time_period.clone();
                        let time_entry = time_stats.entry(time_period).or_insert_with(|| TimeSummary {
                            time_period: period_clone,
                            file_count: 0,
                            total_size: 0,
                        });
                        
                        time_entry.file_count += 1;
                        time_entry.total_size += file.size;
                    }
                }

                // 计算每个目录的主要扩展名
                for (_, dir_summary) in dir_stats.iter_mut() {
                    let mut ext_counts: HashMap<String, u64> = HashMap::new();
                    
                    for file in &files {
                        let path = Path::new(&file.path);
                        if let Some(parent) = path.parent() {
                            if parent.to_string_lossy() == dir_summary.directory {
                                if let Some(ext) = path.extension() {
                                    let ext_str = format!(".{}", ext.to_string_lossy());
                                    *ext_counts.entry(ext_str).or_insert(0) += 1;
                                }
                            }
                        }
                    }
                    
                    // 按数量排序，取前3个
                    let mut ext_vec: Vec<(String, u64)> = ext_counts.into_iter().collect();
                    ext_vec.sort_by(|a, b| b.1.cmp(&a.1));
                    dir_summary.primary_extensions = ext_vec
                        .into_iter()
                        .take(3)
                        .map(|(ext, _)| ext)
                        .collect();
                    
                    dir_summary.last_modified_desc = "近期".to_string();
                }

                // 更新目录计数到扩展名统计
                for (_, ext_summary) in ext_stats.iter_mut() {
                    let mut dirs: std::collections::HashSet<String> = std::collections::HashSet::new();
                    
                    for file in &files {
                        let path = Path::new(&file.path);
                        if let Some(ext) = path.extension() {
                            let ext_str = format!(".{}", ext.to_string_lossy());
                            if ext_str == ext_summary.extension {
                                if let Some(parent) = path.parent() {
                                    dirs.insert(parent.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    
                    ext_summary.directory_count = dirs.len() as u64;
                }

                // 转换为输出格式
                let by_directory: Vec<DirectorySummary> = dir_stats.into_values().collect();
                let by_extension: Vec<ExtensionSummary> = ext_stats.into_values().collect();
                let by_time: Vec<TimeSummary> = time_stats.into_values().collect();

                Ok(ScanGarbageOutput {
                    scan_id: uuid::Uuid::new_v4().to_string(),
                    total_files,
                    total_size,
                    by_directory,
                    by_extension,
                    by_time,
                    high_risk_count,
                    _prompt: String::new(),
                })
            } else {
                // 使用默认的垃圾扫描选项
                let options = crate::modules::file_analyzer::garbage::GarbageDetectorOptions {
                    include_system_temp: args.include_system_temp,
                    include_browser_cache: args.include_browser_cache,
                    include_app_cache: args.include_app_cache,
                    include_recycle_bin: args.include_recycle_bin,
                    include_log_files: args.include_log_files,
                    min_file_age_days: args.min_file_age_days.unwrap_or(0),
                    max_files_per_category: Some(10000), // 限制每类最大文件数，避免内存溢出
                };

                let detector = crate::modules::file_analyzer::garbage::GarbageDetector::with_options(options);
                let result = detector.detect_all();

                // 聚合统计信息
                let mut dir_stats: HashMap<String, DirectorySummary> = HashMap::new();
                let mut ext_stats: HashMap<String, ExtensionSummary> = HashMap::new();
                let mut time_stats: HashMap<String, TimeSummary> = HashMap::new();

                // 遍历所有类别和文件进行聚合
                for (_, category_stats) in &result.categories {
                    for file in &category_stats.files {
                        let path = Path::new(&file.path);
                        
                        // 按目录聚合
                        if let Some(parent) = path.parent() {
                            let dir = parent.to_string_lossy().to_string();
                            let entry = dir_stats.entry(dir.clone()).or_insert_with(|| DirectorySummary {
                                directory: dir,
                                file_count: 0,
                                total_size: 0,
                                primary_extensions: Vec::new(),
                                last_modified_desc: String::new(),
                            });
                            
                            entry.file_count += 1;
                            entry.total_size += file.size;
                        }

                        // 按扩展名聚合
                        if let Some(ext) = path.extension() {
                            let ext_str = format!(".{}", ext.to_string_lossy());
                            let ext_entry = ext_stats.entry(ext_str.clone()).or_insert_with(|| ExtensionSummary {
                                extension: ext_str,
                                file_count: 0,
                                total_size: 0,
                                directory_count: 0,
                            });
                            
                            ext_entry.file_count += 1;
                            ext_entry.total_size += file.size;
                        }

                        // 按时间聚合
                        if file.modified_time > 0 {
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs() as i64;
                            let age_days = (now - file.modified_time) / (24 * 3600);
                            
                            let time_period = if age_days >= 365 * 2 {
                                "over_2_years".to_string()
                            } else if age_days >= 365 {
                                "over_1_year".to_string()
                            } else if age_days >= 180 {
                                "over_6_months".to_string()
                            } else if age_days >= 90 {
                                "over_3_months".to_string()
                            } else if age_days >= 30 {
                                "over_1_month".to_string()
                            } else {
                                "recent".to_string()
                            };
                            
                            let period_clone = time_period.clone();
                            let time_entry = time_stats.entry(time_period).or_insert_with(|| TimeSummary {
                                time_period: period_clone,
                                file_count: 0,
                                total_size: 0,
                            });
                            
                            time_entry.file_count += 1;
                            time_entry.total_size += file.size;
                        }
                    }
                }

                // 计算每个目录的主要扩展名
                for (_, dir_summary) in dir_stats.iter_mut() {
                    let mut ext_counts: HashMap<String, u64> = HashMap::new();
                    
                    // 遍历所有文件来统计扩展名
                    for (_, category_stats) in &result.categories {
                        for file in &category_stats.files {
                            let path = Path::new(&file.path);
                            if let Some(parent) = path.parent() {
                                if parent.to_string_lossy() == dir_summary.directory {
                                    if let Some(ext) = path.extension() {
                                        let ext_str = format!(".{}", ext.to_string_lossy());
                                        *ext_counts.entry(ext_str).or_insert(0) += 1;
                                    }
                                }
                            }
                        }
                    }
                    
                    // 按数量排序，取前3个
                    let mut ext_vec: Vec<(String, u64)> = ext_counts.into_iter().collect();
                    ext_vec.sort_by(|a, b| b.1.cmp(&a.1));
                    dir_summary.primary_extensions = ext_vec
                        .into_iter()
                        .take(3)
                        .map(|(ext, _)| ext)
                        .collect();
                    
                    dir_summary.last_modified_desc = "近期".to_string();
                }

                // 更新目录计数到扩展名统计
                for (_, ext_summary) in ext_stats.iter_mut() {
                    let mut dirs: std::collections::HashSet<String> = std::collections::HashSet::new();
                    
                    for (_, category_stats) in &result.categories {
                        for file in &category_stats.files {
                            let path = Path::new(&file.path);
                            if let Some(ext) = path.extension() {
                                let ext_str = format!(".{}", ext.to_string_lossy());
                                if ext_str == ext_summary.extension {
                                    if let Some(parent) = path.parent() {
                                        dirs.insert(parent.to_string_lossy().to_string());
                                    }
                                }
                            }
                        }
                    }
                    
                    ext_summary.directory_count = dirs.len() as u64;
                }

                // 转换为输出格式
                let by_directory: Vec<DirectorySummary> = dir_stats.into_values().collect();
                let by_extension: Vec<ExtensionSummary> = ext_stats.into_values().collect();
                let by_time: Vec<TimeSummary> = time_stats.into_values().collect();

                Ok(ScanGarbageOutput {
                    scan_id: result.scan_id,
                    total_files: result.total_files,
                    total_size: result.total_size,
                    by_directory,
                    by_extension,
                    by_time,
                    high_risk_count: result.high_risk_count,
                    _prompt: String::new(),
                })
            }
        })
        .await
        .map_err(|e| ToolError::ExecutionError(format!("扫描操作崩溃: {}", e)))?
        .map_err(ToolError::ExecutionError)?;

        Ok(ScanGarbageOutput {
            _prompt: Self.detailed_prompt().to_string(),
            ..scan_result
        })
    }
}