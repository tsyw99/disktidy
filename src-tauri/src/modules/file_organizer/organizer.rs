use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};

use super::classifier::{CategoryRule, ClassificationResult};

/// 整理操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizeAction {
    Move,
    Copy,
    Skip,
}

/// 单个文件的整理操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeOperation {
    pub source_path: String,
    pub dest_path: String,
    pub file_name: String,
    pub file_size_bytes: u64,
    pub action: OrganizeAction,
    pub category: String,
    pub reason: String,
}

/// 整理预览结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizePreview {
    pub preview_id: String,
    pub root_path: String,
    pub operations: Vec<OrganizeOperation>,
    pub total_files: usize,
    pub total_size_bytes: u64,
    pub categories: HashMap<String, Vec<String>>,
    pub summary_stats: PreviewStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewStats {
    pub files_to_move: usize,
    pub files_to_copy: usize,
    pub files_to_skip: usize,
    pub total_categories: usize,
    pub size_to_move_bytes: u64,
}

/// 执行整理后的结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeResult {
    pub success: bool,
    pub moved_count: usize,
    pub copied_count: usize,
    pub skipped_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

pub struct FileOrganizer {
    pub rules: Vec<CategoryRule>,
    base_path: String,
    preserve_structure: bool,
}

impl FileOrganizer {
    pub fn new(base_path: &str, rules: Vec<CategoryRule>) -> Self {
        Self {
            rules,
            base_path: base_path.to_string(),
            preserve_structure: false,
        }
    }

    pub fn with_preserve_structure(mut self, preserve: bool) -> Self {
        self.preserve_structure = preserve;
        self
    }

    /// 生成整理预览
    pub fn preview(
        &self,
        scan_result: &super::scanner::ScanResult,
        classification: &ClassificationResult,
    ) -> OrganizePreview {
        let preview_id = uuid::Uuid::new_v4().to_string();
        let mut operations = Vec::new();
        let mut categories: HashMap<String, Vec<String>> = HashMap::new();

        // 1. 处理规则分类的文件
        for (target_folder, files) in &classification.rule_groups {
            let dest_dir = Path::new(&self.base_path).join(target_folder);
            let dest_dir_str = dest_dir.to_string_lossy().to_string();

            for file in files {
                let dest_path = if self.preserve_structure {
                    // 保留相对目录结构
                    let rel_path = file.path
                        .strip_prefix(&scan_result.root_path)
                        .unwrap_or(&file.name);
                    let rel_dir = Path::new(rel_path).parent()
                        .map(|p| p.to_string_lossy().to_string())
                        .unwrap_or_default();
                    Path::new(&dest_dir_str).join(&rel_dir).join(&file.name).to_string_lossy().to_string()
                } else {
                    Path::new(&dest_dir_str).join(&file.name).to_string_lossy().to_string()
                };

                operations.push(OrganizeOperation {
                    source_path: file.path.clone(),
                    dest_path: dest_path.clone(),
                    file_name: file.name.clone(),
                    file_size_bytes: file.size_bytes,
                    action: OrganizeAction::Move,
                    category: target_folder.clone(),
                    reason: format!("匹配分类规则: {}", target_folder),
                });

                categories.entry(target_folder.clone())
                    .or_default()
                    .push(file.name.clone());
            }
        }

        // 2. 处理内容分组（相似度归类）
        for group in &classification.content_groups {
            let dest_dir = Path::new(&self.base_path)
                .join("相似文件")
                .join(&group.group_name);
            let dest_dir_str = dest_dir.to_string_lossy().to_string();

            for file in &group.files {
                if operations.iter().any(|op| op.source_path == file.path) {
                    continue;
                }

                let dest_path = Path::new(&dest_dir_str)
                    .join(&file.name)
                    .to_string_lossy()
                    .to_string();

                operations.push(OrganizeOperation {
                    source_path: file.path.clone(),
                    dest_path: dest_path.clone(),
                    file_name: file.name.clone(),
                    file_size_bytes: file.size_bytes,
                    action: OrganizeAction::Copy,
                    category: group.group_name.clone(),
                    reason: format!("内容相似度: {:.1}%", group.similarity_score * 100.0),
                });

                categories.entry("相似文件".to_string())
                    .or_default()
                    .push(file.name.clone());
            }
        }

        // 3. 处理未分类文件
        if !classification.unclassified.is_empty() {
            let dest_dir = Path::new(&self.base_path).join("其他");
            let dest_dir_str = dest_dir.to_string_lossy().to_string();

            for file in &classification.unclassified {
                operations.push(OrganizeOperation {
                    source_path: file.path.clone(),
                    dest_path: Path::new(&dest_dir_str).join(&file.name).to_string_lossy().to_string(),
                    file_name: file.name.clone(),
                    file_size_bytes: file.size_bytes,
                    action: OrganizeAction::Skip,
                    category: "其他".to_string(),
                    reason: "未能自动分类".to_string(),
                });

                categories.entry("其他".to_string())
                    .or_default()
                    .push(file.name.clone());
            }
        }

        let total_files = operations.len();
        let total_size_bytes = operations.iter().map(|op| op.file_size_bytes).sum();
        let files_to_move = operations.iter().filter(|op| matches!(op.action, OrganizeAction::Move)).count();
        let files_to_copy = operations.iter().filter(|op| matches!(op.action, OrganizeAction::Copy)).count();
        let files_to_skip = operations.iter().filter(|op| matches!(op.action, OrganizeAction::Skip)).count();
        let size_to_move_bytes = operations.iter()
            .filter(|op| matches!(op.action, OrganizeAction::Move))
            .map(|op| op.file_size_bytes)
            .sum();

        OrganizePreview {
            preview_id,
            root_path: self.base_path.clone(),
            operations,
            total_files,
            total_size_bytes,
            categories: categories.clone(),
            summary_stats: PreviewStats {
                files_to_move,
                files_to_copy,
                files_to_skip,
                total_categories: categories.len(),
                size_to_move_bytes,
            },
        }
    }

    /// 执行整理操作
    pub fn execute(
        &self,
        preview: &OrganizePreview,
        confirmed_operations: Option<Vec<usize>>, // 用户确认的操作索引（None表示全部执行）
        dry_run: bool,
    ) -> OrganizeResult {
        let mut moved = 0;
        let mut copied = 0;
        let mut skipped = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for (i, op) in preview.operations.iter().enumerate() {
            // 如果指定了确认列表，只执行确认的操作
            if let Some(ref confirmed) = confirmed_operations {
                if !confirmed.contains(&i) {
                    skipped += 1;
                    continue;
                }
            }

            if matches!(op.action, OrganizeAction::Skip) {
                skipped += 1;
                continue;
            }

            if dry_run {
                match op.action {
                    OrganizeAction::Move => moved += 1,
                    OrganizeAction::Copy => copied += 1,
                    _ => skipped += 1,
                }
                continue;
            }

            // 创建目标目录
            let dest_parent = Path::new(&op.dest_path).parent();
            if let Some(parent) = dest_parent {
                if !parent.exists() {
                    if let Err(e) = std::fs::create_dir_all(parent) {
                        failed += 1;
                        errors.push(format!("创建目录失败 {}: {}", parent.display(), e));
                        continue;
                    }
                }
            }

            // 处理文件名冲突
            let final_dest = Self::resolve_conflict(&op.dest_path);

            // 执行操作
            match op.action {
                OrganizeAction::Move => {
                    match std::fs::rename(&op.source_path, &final_dest) {
                        Ok(_) => moved += 1,
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("移动失败 {}: {}", op.file_name, e));
                        }
                    }
                }
                OrganizeAction::Copy => {
                    match std::fs::copy(&op.source_path, &final_dest) {
                        Ok(_) => copied += 1,
                        Err(e) => {
                            failed += 1;
                            errors.push(format!("复制失败 {}: {}", op.file_name, e));
                        }
                    }
                }
                OrganizeAction::Skip => skipped += 1,
            }
        }

        OrganizeResult {
            success: failed == 0,
            moved_count: moved,
            copied_count: copied,
            skipped_count: skipped,
            failed_count: failed,
            errors,
        }
    }

    /// 撤销整理操作（移动文件回原位）
    pub fn rollback(
        &self,
        preview: &OrganizePreview,
        _result: &OrganizeResult,
    ) -> OrganizeResult {
        let mut rolled_back = 0;
        let mut failed = 0;
        let mut errors = Vec::new();

        for op in &preview.operations {
            let dest_path = Path::new(&op.dest_path);
            if !dest_path.exists() {
                continue;
            }

            match std::fs::rename(dest_path, &op.source_path) {
                Ok(_) => rolled_back += 1,
                Err(e) => {
                    failed += 1;
                    errors.push(format!("回滚失败 {}: {}", op.file_name, e));
                }
            }
        }

        OrganizeResult {
            success: failed == 0,
            moved_count: rolled_back,
            copied_count: 0,
            skipped_count: 0,
            failed_count: failed,
            errors,
        }
    }

    fn resolve_conflict(path: &str) -> String {
        let p = Path::new(path);
        if !p.exists() {
            return path.to_string();
        }

        let stem = p.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let ext = p.extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .unwrap_or_default();
        let parent = p.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let mut counter = 1;
        loop {
            let new_path = format!("{}\\{} ({}){}", parent, stem, counter, ext);
            if !Path::new(&new_path).exists() {
                return new_path;
            }
            counter += 1;
        }
    }
}
