pub mod tool_error;
pub mod disk_scan_tool;
pub mod file_classifier_tool;
pub mod large_file_tool;
pub mod cleaner_tool;
pub mod app_cache_tool;
pub mod software_residue_tool;
pub mod garbage_analyzer_tool;
pub mod file_search_tool;
pub mod file_delete_tool;
pub mod file_content_tool;
pub mod file_organizer_tool;
pub mod scan_desktop_tool;
pub mod organize_files_tool;
pub mod resolve_path_tool;
pub mod file_write_tool;
pub mod excel_cache;
pub mod read_excel_tool;
pub mod analyze_data_tool;
pub mod generate_html_tool;
pub mod scan_garbage_tool;
pub mod clean_garbage_tool;

pub use tool_error::ToolError;
pub use disk_scan_tool::DiskScanTool;
pub use file_classifier_tool::FileClassifierTool;
pub use large_file_tool::LargeFileTool;
pub use cleaner_tool::CleanerTool;
pub use app_cache_tool::AppCacheTool;
pub use software_residue_tool::SoftwareResidueTool;
pub use garbage_analyzer_tool::GarbageAnalyzerTool;
pub use file_search_tool::FileSearchTool;
pub use file_delete_tool::FileDeleteTool;
pub use file_content_tool::FileContentTool;
pub use file_organizer_tool::FileOrganizerTool;
pub use scan_desktop_tool::ScanDesktopTool;
pub use organize_files_tool::OrganizeFilesTool;
pub use resolve_path_tool::ResolvePathTool;
pub use read_excel_tool::ReadExcelTool;
pub use analyze_data_tool::AnalyzeDataTool;
pub use generate_html_tool::GenerateHtmlTool;
pub use scan_garbage_tool::ScanGarbageTool;
pub use clean_garbage_tool::CleanGarbageTool;

/// 工具专属提示词 trait，仅在工具被调用时加载到上下文
pub trait ToolPrompt {
    fn detailed_prompt(&self) -> &'static str {
        ""
    }
}
