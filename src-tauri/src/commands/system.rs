use crate::models::{CpuInfo, DiskInfo, MemoryInfo, SystemInfo};
use crate::modules;
use crate::utils::path::SystemPaths;
use serde::{Deserialize, Serialize};
use tauri::command;

#[command]
pub async fn system_get_info() -> Result<SystemInfo, String> {
    modules::get_system_info()
}

#[command]
pub async fn system_get_disks() -> Result<Vec<DiskInfo>, String> {
    modules::get_disk_list()
}

#[command]
pub async fn system_get_cpu_info() -> Result<CpuInfo, String> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    modules::get_cpu_info(&sys)
}

#[command]
pub async fn system_get_memory_info() -> Result<MemoryInfo, String> {
    let mut sys = sysinfo::System::new_all();
    sys.refresh_all();
    Ok(modules::get_memory_info(&sys))
}

/// 系统路径响应
#[derive(Debug, Serialize)]
pub struct SystemPathsResponse {
    pub home: Option<String>,
    pub desktop: Option<String>,
    pub documents: Option<String>,
    pub downloads: Option<String>,
    pub temp: String,
}

/// 获取系统路径信息
#[command]
pub async fn system_get_paths() -> Result<SystemPathsResponse, String> {
    let home = SystemPaths::home_dir().map(|p| p.to_string_lossy().to_string());
    let desktop = SystemPaths::desktop_dir().map(|p| p.to_string_lossy().to_string());
    let documents = SystemPaths::documents_dir().map(|p| p.to_string_lossy().to_string());
    let downloads = dirs::download_dir().map(|p| p.to_string_lossy().to_string());
    let temp = SystemPaths::temp_dir().to_string_lossy().to_string();

    Ok(SystemPathsResponse {
        home,
        desktop,
        documents,
        downloads,
        temp,
    })
}

/// 路径解析请求
#[derive(Debug, Deserialize)]
pub struct ResolvePathRequest {
    pub path: String,
}

/// 路径解析响应
#[derive(Debug, Serialize)]
pub struct ResolvePathResponse {
    pub resolved: Option<String>,
    pub is_alias: bool,
    pub alias_name: Option<String>,
}

/// 智能解析路径（支持"桌面"、"desktop"、"文档"等别名）
#[command]
pub async fn system_resolve_path(request: ResolvePathRequest) -> Result<ResolvePathResponse, String> {
    let input = request.path.trim();
    let is_alias = !input.contains(':') && !input.contains('/') && !input.contains('\\');

    if let Some(resolved) = SystemPaths::resolve_path(input) {
        let alias_name = if is_alias { Some(input.to_string()) } else { None };
        Ok(ResolvePathResponse {
            resolved: Some(resolved.to_string_lossy().to_string()),
            is_alias,
            alias_name,
        })
    } else {
        Ok(ResolvePathResponse {
            resolved: None,
            is_alias,
            alias_name: if is_alias { Some(input.to_string()) } else { None },
        })
    }
}

/// 打开文件保存对话框并保存 HTML 内容
#[derive(Debug, Deserialize)]
pub struct SaveHtmlRequest {
    pub html_content: String,
    pub default_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SaveHtmlResponse {
    pub saved: bool,
    pub file_path: Option<String>,
}

#[command]
pub async fn system_save_html(request: SaveHtmlRequest, app: tauri::AppHandle) -> Result<SaveHtmlResponse, String> {
    use tauri_plugin_dialog::DialogExt;
    use std::fs;

    let default_name = request.default_name.unwrap_or_else(|| {
        format!("分析报告_{}.html", chrono::Local::now().format("%Y%m%d_%H%M%S"))
    });

    // 使用 Tauri 文件保存对话框
    let file_path = app
        .dialog()
        .file()
        .add_filter("HTML文件", &["html", "htm"])
        .set_file_name(&default_name)
        .blocking_save_file();

    if let Some(path) = file_path {
        let save_path = path.as_path()
            .ok_or_else(|| "无法获取文件路径".to_string())?
            .to_path_buf();
        fs::write(&save_path, &request.html_content)
            .map_err(|e| format!("写入文件失败: {}", e))?;
        Ok(SaveHtmlResponse {
            saved: true,
            file_path: Some(save_path.to_string_lossy().to_string()),
        })
    } else {
        Ok(SaveHtmlResponse {
            saved: false,
            file_path: None,
        })
    }
}

/// 将 HTML 保存到临时文件并在默认浏览器中打开
#[derive(Debug, Deserialize)]
pub struct OpenBrowserRequest {
    pub html_content: String,
}

#[derive(Debug, Serialize)]
pub struct OpenBrowserResponse {
    pub opened: bool,
    pub file_path: String,
}

#[command]
pub async fn system_open_in_browser(request: OpenBrowserRequest) -> Result<OpenBrowserResponse, String> {
    use std::fs;
    use std::path::PathBuf;

    // 写入临时目录
    let temp_dir = std::env::temp_dir();
    let file_name = format!("disktidy_report_{}.html", chrono::Local::now().format("%Y%m%d_%H%M%S"));
    let file_path: PathBuf = temp_dir.join(&file_name);

    fs::write(&file_path, &request.html_content)
        .map_err(|e| format!("写入临时文件失败: {}", e))?;

    // 使用系统默认浏览器打开
    open::that(file_path.to_string_lossy().to_string())
        .map_err(|e| format!("打开浏览器失败: {}", e))?;

    Ok(OpenBrowserResponse {
        opened: true,
        file_path: file_path.to_string_lossy().to_string(),
    })
}

/// 用默认程序打开文件（.html 会用浏览器打开）
#[command]
pub async fn system_open_file(path: String) -> Result<(), String> {
    open::that(&path).map_err(|e| format!("打开文件失败: {}", e))
}

/// 在资源管理器中打开文件所在目录
#[command]
pub async fn system_open_folder(path: String) -> Result<(), String> {
    let parent = std::path::Path::new(&path)
        .parent()
        .ok_or_else(|| "无法获取父目录".to_string())?;
    open::that(parent).map_err(|e| format!("打开目录失败: {}", e))
}
