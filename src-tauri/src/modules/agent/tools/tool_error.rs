use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolError {
    #[error("工具执行失败: {0}")]
    ExecutionFailed(String),

    #[error("参数错误: {0}")]
    InvalidArgs(String),

    #[error("扫描失败: {0}")]
    ScanFailed(String),

    #[error("清理失败: {0}")]
    CleanFailed(String),

    #[error("需要用户确认")]
    RequiresConfirmation,
}
