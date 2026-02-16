use crate::models::{CpuInfo, DiskInfo, MemoryInfo, SystemInfo};
use crate::modules;
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
