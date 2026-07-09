use serde::{Deserialize, Serialize};
use std::path::Path;
use log::debug;

use super::tool_error::ToolError;
use super::ToolPrompt;

// ── input ──
#[derive(Debug, Deserialize)]
pub struct OrganizeFilesInput {
    /// 源文件所在根目录（用于安全校验：所有源文件必须在此目录下）
    pub root_directory: String,
    /// 分类操作组：每个组有目标文件夹名和源文件路径列表
    pub groups: Vec<OperationGroup>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OperationGroup {
    /// 目标文件夹名（相对于 root_directory）
    pub target_folder: String,
    /// 源文件路径列表（绝对路径）
    pub files: Vec<String>,
}

// ── output ──
#[derive(Debug, Serialize)]
pub struct OrganizeFilesOutput {
    pub root_directory: String,
    pub total_moved: usize,
    pub total_skipped: usize,
    pub total_failed: usize,
    pub groups: Vec<GroupResult>,
    pub _prompt: String,
}

#[derive(Debug, Serialize)]
pub struct GroupResult {
    pub target_folder: String,
    pub moved: usize,
    pub skipped: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

pub struct OrganizeFilesTool;

impl OrganizeFilesTool {
    pub fn new() -> Self { Self }
}

impl ToolPrompt for OrganizeFilesTool {
    fn detailed_prompt(&self) -> &'static str {
        r#"## 整理完成
向用户汇报整理结果：每个文件夹移动了多少文件，是否有失败。
如果用户想要撤回：告知用户可以在桌面找到对应文件夹手动恢复（同一磁盘移动，文件不会丢）。
"#
    }
}

impl rig_core::tool::Tool for OrganizeFilesTool {
    const NAME: &'static str = "organize_files";
    type Error = ToolError;
    type Args = OrganizeFilesInput;
    type Output = OrganizeFilesOutput;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "按分类方案移动文件到指定文件夹。注意：仅支持同目录内移动，安全约束内置，执行前会提示用户确认。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "root_directory": {
                        "type": "string",
                        "description": "源文件所在的根目录绝对路径"
                    },
                    "groups": {
                        "type": "array",
                        "description": "分类操作组列表",
                        "items": {
                            "type": "object",
                            "properties": {
                                "target_folder": {
                                    "type": "string",
                                    "description": "目标文件夹名（将在 root_directory 下创建）"
                                },
                                "files": {
                                    "type": "array",
                                    "items": { "type": "string" },
                                    "description": "要移动的源文件绝对路径列表"
                                }
                            },
                            "required": ["target_folder", "files"]
                        }
                    }
                },
                "required": ["root_directory", "groups"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let root = args.root_directory.clone();

        let result = tokio::task::spawn_blocking(move || -> Result<OrganizeFilesOutput, String> {
            let root = Path::new(&root);
            if !root.is_dir() {
                return Err(format!("目录不存在: {}", root.display()));
            }
            let root_canonical = root.canonicalize()
                .unwrap_or_else(|_| root.to_path_buf());

            let mut total_moved = 0usize;
            let mut total_skipped = 0usize;
            let mut total_failed = 0usize;
            let mut group_results = Vec::new();

            // 保存执行计划（用于可能的回滚）
            let plan_path = root.join(".disk_tidy_organize_plan.json");
            let plan_json = serde_json::to_string_pretty(&args.groups).unwrap_or_default();
            let _ = std::fs::write(&plan_path, &plan_json);

            for group in &args.groups {
                let mut moved = 0usize;
                let mut skipped = 0usize;
                let mut failed = 0usize;
                let mut errors = Vec::new();

                // 安全检查：目标文件夹名不能是系统路径
                let folder_name = group.target_folder.trim();
                if folder_name.is_empty()
                    || folder_name == "."
                    || folder_name == ".."
                    || folder_name.contains('\\')
                    || folder_name.contains('/')
                {
                    return Err(format!("非法目标文件夹名: {}", folder_name));
                }

                let target_dir = root.join(folder_name);

                for source in &group.files {
                    let source_path = Path::new(source);

                    // ── 安全约束 1: 源文件必须在根目录下（max_depth=1） ──
                    let source_parent = source_path.parent()
                        .and_then(|p| p.canonicalize().ok())
                        .unwrap_or_else(|| Path::new("").to_path_buf());

                    if source_parent != root_canonical {
                        skipped += 1;
                        errors.push(format!("安全跳过（不在根目录下）: {}", source));
                        continue;
                    }

                    // ── 安全约束 2: 源文件必须存在且为文件 ──
                    if !source_path.is_file() {
                        skipped += 1;
                        errors.push(format!("跳过（不是文件或不存在）: {}", source));
                        continue;
                    }

                    let file_name = source_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown");

                    // ── 安全约束 3: 目标文件夹创建在根目录下 ──
                    if let Err(e) = std::fs::create_dir_all(&target_dir) {
                        failed += 1;
                        errors.push(format!("创建目录失败 {}: {}", target_dir.display(), e));
                        continue;
                    }

                    let dest = target_dir.join(file_name);

                    // ── 安全约束 4: 冲突处理（追加时间戳后缀） ──
                    let dest = resolve_conflict(&dest);

                    // ── 执行移动（同盘 rename = 原子操作，不复制不删除） ──
                    match std::fs::rename(source_path, &dest) {
                        Ok(_) => {
                            moved += 1;
                            log::debug!("移动: {} -> {}", source, dest.display());
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("移动失败 {}: {}", file_name, e));
                        }
                    }
                }

                total_moved += moved;
                total_skipped += skipped;
                total_failed += failed;

                group_results.push(GroupResult {
                    target_folder: folder_name.to_string(),
                    moved,
                    skipped,
                    failed,
                    errors,
                });
            }

            // 执行完成，删除计划文件
            let _ = std::fs::remove_file(&plan_path);

            Ok(OrganizeFilesOutput {
                root_directory: root.to_string_lossy().to_string(),
                total_moved,
                total_skipped,
                total_failed,
                groups: group_results,
                _prompt: String::new(),
            })
        })
        .await
        .map_err(|e| ToolError::ExecutionError(format!("整理操作崩溃: {}", e)))?
        .map_err(ToolError::ExecutionError)?;

        debug!(
            "organize_files: 移动 {} 成功, {} 跳过, {} 失败",
            result.total_moved, result.total_skipped, result.total_failed
        );

        Ok(OrganizeFilesOutput {
            _prompt: Self.detailed_prompt().to_string(),
            ..result
        })
    }
}

/// 冲突处理：已有同名文件时追加时间戳后缀
fn resolve_conflict(dest: &Path) -> std::path::PathBuf {
    if !dest.exists() {
        return dest.to_path_buf();
    }

    let stem = dest.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = dest.extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();
    let parent = dest.parent().unwrap_or(Path::new("."));

    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // 格式: filename_20260709_143022.ext
    let days = ts / 86400;
    let time_of_day = ts % 86400;
    let hour = time_of_day / 3600;
    let min = (time_of_day % 3600) / 60;
    let sec = time_of_day % 60;

    let year = 1970 + (days / 365) as i32;
    let remainder = days % 365;
    let month = (remainder / 30 + 1) as u32;
    let day = (remainder % 30 + 1) as u32;

    let ts_suffix = format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month.min(12), day.min(28), hour, min, sec
    );

    let new_name = format!("{}_{}{}", stem, ts_suffix, ext);
    parent.join(new_name)
}
