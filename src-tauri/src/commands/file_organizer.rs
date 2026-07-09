use serde::Deserialize;

use crate::modules::file_organizer::{
    FileOrganizerScanner, ContentClassifier, CategoryRule, FileOrganizer,
    ScanResult, ClassificationResult, OrganizePreview, OrganizeResult,
};

/// 扫描目录请求
#[derive(Debug, Deserialize)]
pub struct OrganizeScanRequest {
    pub path: String,
    pub include_hidden: Option<bool>,
    pub max_depth: Option<usize>,
}

/// 预览请求
#[derive(Debug, Deserialize)]
pub struct OrganizePreviewRequest {
    pub path: String,
    pub include_hidden: Option<bool>,
    pub max_depth: Option<usize>,
    pub custom_rules: Option<Vec<CategoryRule>>,
}

/// 执行整理请求
#[derive(Debug, Deserialize)]
pub struct OrganizeExecuteRequest {
    pub path: String,
    pub include_hidden: Option<bool>,
    pub max_depth: Option<usize>,
    pub custom_rules: Option<Vec<CategoryRule>>,
    pub confirmed_operations: Option<Vec<usize>>,
    pub dry_run: Option<bool>,
}

/// 扫描目录，获取文件列表和统计
#[tauri::command]
pub async fn organizer_scan(
    request: OrganizeScanRequest,
) -> Result<ScanResult, String> {
    let result = FileOrganizerScanner::scan(
        &request.path,
        request.include_hidden.unwrap_or(false),
        request.max_depth,
        None,
    );
    Ok(result)
}

/// 预览整理方案
#[tauri::command]
pub async fn organizer_preview(
    request: OrganizePreviewRequest,
) -> Result<OrganizePreview, String> {
    let scan_result = FileOrganizerScanner::scan(
        &request.path,
        request.include_hidden.unwrap_or(false),
        request.max_depth,
        None,
    );

    let rules = request.custom_rules.unwrap_or_else(ContentClassifier::default_rules);
    let classifier = ContentClassifier::new(rules);
    let classification = classifier.classify(&scan_result.files);

    let organizer = FileOrganizer::new(&request.path, vec![]);
    let preview = organizer.preview(&scan_result, &classification);

    Ok(preview)
}

/// 执行文件整理
#[tauri::command]
pub async fn organizer_execute(
    request: OrganizeExecuteRequest,
) -> Result<OrganizeResult, String> {
    let scan_result = FileOrganizerScanner::scan(
        &request.path,
        request.include_hidden.unwrap_or(false),
        request.max_depth,
        None,
    );

    let rules = request.custom_rules.unwrap_or_else(ContentClassifier::default_rules);
    let classifier = ContentClassifier::new(rules);
    let classification = classifier.classify(&scan_result.files);

    let organizer = FileOrganizer::new(&request.path, vec![]);
    let preview = organizer.preview(&scan_result, &classification);

    let result = organizer.execute(
        &preview,
        request.confirmed_operations,
        request.dry_run.unwrap_or(false),
    );

    Ok(result)
}

/// 获取默认分类规则
#[tauri::command]
pub async fn organizer_default_rules() -> Result<Vec<CategoryRule>, String> {
    Ok(ContentClassifier::default_rules())
}

/// 根据自定义规则分类文件
#[derive(Debug, Deserialize)]
pub struct OrganizerClassifyRequest {
    pub path: String,
    pub include_hidden: Option<bool>,
    pub max_depth: Option<usize>,
    pub rules: Vec<CategoryRule>,
}

#[tauri::command]
pub async fn organizer_classify(
    request: OrganizerClassifyRequest,
) -> Result<ClassificationResult, String> {
    let scan_result = FileOrganizerScanner::scan(
        &request.path,
        request.include_hidden.unwrap_or(false),
        request.max_depth,
        None,
    );

    let classifier = ContentClassifier::new(request.rules);
    let classification = classifier.classify(&scan_result.files);

    Ok(classification)
}
