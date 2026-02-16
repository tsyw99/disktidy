use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub os_arch: String,
    pub hostname: String,
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub cores: usize,
    pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub free: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub total_size: u64,
    pub used_size: u64,
    pub free_size: u64,
    pub usage_percent: f32,
    pub volume_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub date: String,
    pub provider: String,
    pub status: DriverStatus,
    pub driver_type: String,
    pub device_name: String,
    pub inf_name: String,
    pub signed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DriverStatus {
    Running,
    Stopped,
    Error,
    Unknown,
}

impl std::fmt::Display for DriverStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverStatus::Running => write!(f, "运行中"),
            DriverStatus::Stopped => write!(f, "已停止"),
            DriverStatus::Error => write!(f, "异常"),
            DriverStatus::Unknown => write!(f, "未知"),
        }
    }
}
