use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use super::analyze_data_tool::ReportData;

/// 缓存的 Excel 数据
#[derive(Debug, Clone)]
pub struct CachedExcelData {
    /// 目录路径
    pub directory: String,
    /// 文件名列表
    pub file_names: Vec<String>,
    /// 每文件行数
    pub row_counts: Vec<usize>,
    /// 列名
    pub columns: Vec<String>,
    /// 合并后的所有行（HashMap<列名, 值>）
    pub rows: Vec<HashMap<String, String>>,
    /// 分析报告（analyze_data 计算后缓存于此，避免通过 LLM 传递大 JSON）
    pub report: Option<ReportData>,
}

/// 全局 Excel 数据缓存（按目录路径索引）
static EXCEL_CACHE: Lazy<Mutex<HashMap<String, CachedExcelData>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// 将数据存入缓存
pub fn cache_put(key: &str, data: CachedExcelData) {
    if let Ok(mut cache) = EXCEL_CACHE.lock() {
        cache.insert(key.to_string(), data);
    }
}

/// 从缓存取出数据
pub fn cache_get(key: &str) -> Option<CachedExcelData> {
    EXCEL_CACHE.lock().ok()?.get(key).cloned()
}

/// 更新缓存中的 report 字段
pub fn cache_set_report(key: &str, report: ReportData) {
    if let Ok(mut cache) = EXCEL_CACHE.lock() {
        if let Some(data) = cache.get_mut(key) {
            data.report = Some(report);
        }
    }
}

/// 获取缓存中的 report
pub fn cache_get_report(key: &str) -> Option<ReportData> {
    EXCEL_CACHE
        .lock()
        .ok()?
        .get(key)?
        .report
        .clone()
}

/// 清除指定缓存（释放内存）
pub fn cache_clear(key: &str) {
    if let Ok(mut cache) = EXCEL_CACHE.lock() {
        cache.remove(key);
    }
}
