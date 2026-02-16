use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiskTidyError {
    #[error("系统信息获取失败: {0}")]
    SystemInfoFailed(String),

    #[error("磁盘信息获取失败: {0}")]
    DiskInfoFailed(String),

    #[error("扫描任务不存在: {0}")]
    ScanNotFound(String),

    #[error("扫描任务已在运行中")]
    ScanAlreadyRunning,

    #[error("扫描路径无效: {0}")]
    ScanPathInvalid(String),

    #[error("扫描权限被拒绝: {0}")]
    ScanPermissionDenied(String),

    #[error("清理文件不存在: {0}")]
    CleanFileNotFound(String),

    #[error("清理权限被拒绝: {0}")]
    CleanPermissionDenied(String),

    #[error("文件正在使用中: {0}")]
    CleanFileInUse(String),

    #[error("受保护的文件: {0}")]
    CleanProtectedFile(String),

    #[error("设置加载失败: {0}")]
    SettingsLoadFailed(String),

    #[error("设置保存失败: {0}")]
    SettingsSaveFailed(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("权限不足: {path}")]
    PermissionDenied { path: String },

    #[error("文件不存在: {path}")]
    FileNotFound { path: String },

    #[error("文件已被占用: {path}")]
    FileInUse { path: String },

    #[error("路径受保护: {path}")]
    ProtectedPath { path: String },

    #[error("哈希计算失败: {path}")]
    HashCalculationFailed { path: String },

    #[error("哈希计算超时: {path}")]
    HashCalculationTimeout { path: String },

    #[error("无效参数: {message}")]
    InvalidParameter { message: String },

    #[error("系统调用失败: {api} - {message}")]
    SystemCallFailed { api: String, message: String },

    #[error("配置文件错误: {message}")]
    ConfigError { message: String },

    #[error("未知错误: {0}")]
    Unknown(String),
}

impl DiskTidyError {
    pub fn error_code(&self) -> &str {
        match self {
            Self::SystemInfoFailed(_) => "E001",
            Self::DiskInfoFailed(_) => "E002",
            Self::ScanNotFound(_) => "E003",
            Self::ScanAlreadyRunning => "E004",
            Self::ScanPathInvalid(_) => "E005",
            Self::ScanPermissionDenied(_) => "E006",
            Self::CleanFileNotFound(_) => "E007",
            Self::CleanPermissionDenied(_) => "E008",
            Self::CleanFileInUse(_) => "E009",
            Self::CleanProtectedFile(_) => "E010",
            Self::SettingsLoadFailed(_) => "E011",
            Self::SettingsSaveFailed(_) => "E012",
            Self::IoError(_) => "E013",
            Self::PermissionDenied { .. } => "E014",
            Self::FileNotFound { .. } => "E015",
            Self::FileInUse { .. } => "E016",
            Self::ProtectedPath { .. } => "E017",
            Self::HashCalculationFailed { .. } => "E018",
            Self::HashCalculationTimeout { .. } => "E022",
            Self::InvalidParameter { .. } => "E019",
            Self::SystemCallFailed { .. } => "E020",
            Self::ConfigError { .. } => "E021",
            Self::Unknown(_) => "E999",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: i32,
    pub message: String,
    pub details: Option<String>,
}

impl From<DiskTidyError> for ErrorResponse {
    fn from(error: DiskTidyError) -> Self {
        let (code, message) = match &error {
            DiskTidyError::SystemInfoFailed(_) => (1001, error.to_string()),
            DiskTidyError::DiskInfoFailed(_) => (1002, error.to_string()),
            DiskTidyError::ScanNotFound(_) => (2001, error.to_string()),
            DiskTidyError::ScanAlreadyRunning => (2002, error.to_string()),
            DiskTidyError::ScanPathInvalid(_) => (2003, error.to_string()),
            DiskTidyError::ScanPermissionDenied(_) => (2004, error.to_string()),
            DiskTidyError::CleanFileNotFound(_) => (3001, error.to_string()),
            DiskTidyError::CleanPermissionDenied(_) => (3002, error.to_string()),
            DiskTidyError::CleanFileInUse(_) => (3003, error.to_string()),
            DiskTidyError::CleanProtectedFile(_) => (3004, error.to_string()),
            DiskTidyError::SettingsLoadFailed(_) => (4001, error.to_string()),
            DiskTidyError::SettingsSaveFailed(_) => (4002, error.to_string()),
            DiskTidyError::IoError(_) => (5001, error.to_string()),
            DiskTidyError::PermissionDenied { .. } => (5002, error.to_string()),
            DiskTidyError::FileNotFound { .. } => (5003, error.to_string()),
            DiskTidyError::FileInUse { .. } => (5004, error.to_string()),
            DiskTidyError::ProtectedPath { .. } => (5005, error.to_string()),
            DiskTidyError::HashCalculationFailed { .. } => (5006, error.to_string()),
            DiskTidyError::HashCalculationTimeout { .. } => (5010, error.to_string()),
            DiskTidyError::InvalidParameter { .. } => (5007, error.to_string()),
            DiskTidyError::SystemCallFailed { .. } => (5008, error.to_string()),
            DiskTidyError::ConfigError { .. } => (5009, error.to_string()),
            DiskTidyError::Unknown(_) => (9999, error.to_string()),
        };

        ErrorResponse {
            code,
            message,
            details: None,
        }
    }
}
