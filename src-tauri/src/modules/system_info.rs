use crate::models::{CpuInfo, DiskInfo, MemoryInfo, SystemInfo};
use std::env;
use sysinfo::System;
use windows::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW,
};
use windows::Win32::Foundation::MAX_PATH;

pub fn get_system_info() -> Result<SystemInfo, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let os_arch = env::consts::ARCH.to_string();
    let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

    let cpu_info = get_cpu_info(&sys)?;
    let memory_info = get_memory_info(&sys);

    Ok(SystemInfo {
        os_name,
        os_version,
        os_arch,
        hostname,
        cpu_info,
        memory_info,
    })
}

pub fn get_cpu_info(sys: &System) -> Result<CpuInfo, String> {
    let cpus = sys.cpus();
    
    if cpus.is_empty() {
        return Ok(CpuInfo {
            name: "Unknown".to_string(),
            cores: 0,
            usage: 0.0,
        });
    }

    let name = cpus[0].brand().to_string();
    let cores = cpus.len();
    
    let total_usage: f32 = cpus.iter().map(|cpu| cpu.cpu_usage()).sum();
    let usage = if cores > 0 { total_usage / cores as f32 } else { 0.0 };

    Ok(CpuInfo {
        name,
        cores,
        usage,
    })
}

pub fn get_memory_info(sys: &System) -> MemoryInfo {
    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = sys.available_memory();
    let usage_percent = if total > 0 {
        (used as f32 / total as f32) * 100.0
    } else {
        0.0
    };

    MemoryInfo {
        total,
        used,
        free,
        usage_percent,
    }
}

pub fn get_disk_list() -> Result<Vec<DiskInfo>, String> {
    let mut disks = Vec::new();
    
    let mut drive_strings = vec![0u16; 256];
    let len = unsafe { GetLogicalDriveStringsW(Some(drive_strings.as_mut_slice())) };
    
    if len == 0 {
        return Err("Failed to get logical drive strings".to_string());
    }

    let drive_strings: Vec<String> = drive_strings[..len as usize]
        .chunks(4)
        .filter(|chunk| chunk[0] != 0)
        .map(|chunk| {
            String::from_utf16_lossy(&chunk[..3])
        })
        .collect();

    for drive in drive_strings {
        if let Ok(disk_info) = get_disk_info(&drive) {
            disks.push(disk_info);
        }
    }

    Ok(disks)
}

fn get_disk_info(drive: &str) -> Result<DiskInfo, String> {
    let wide_drive: Vec<u16> = drive.encode_utf16().chain(std::iter::once(0)).collect();
    
    let mut free_bytes: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut available_bytes: u64 = 0;

    let result = unsafe {
        GetDiskFreeSpaceExW(
            windows::core::PCWSTR(wide_drive.as_ptr()),
            Some(&mut free_bytes),
            Some(&mut total_bytes),
            Some(&mut available_bytes),
        )
    };

    if result.is_err() {
        return Err(format!("Failed to get disk space for {}", drive));
    }

    let used_bytes = total_bytes - free_bytes;
    let usage_percent = if total_bytes > 0 {
        (used_bytes as f32 / total_bytes as f32) * 100.0
    } else {
        0.0
    };

    let mut volume_name = vec![0u16; MAX_PATH as usize + 1];
    
    unsafe {
        let _ = GetVolumeInformationW(
            windows::core::PCWSTR(wide_drive.as_ptr()),
            Some(volume_name.as_mut_slice()),
            None,
            None,
            None,
            None,
        );
    };

    let volume_name_str = String::from_utf16_lossy(&volume_name)
        .trim_end_matches('\0')
        .to_string();
    let file_system_str = String::new();

    let _drive_type = unsafe { GetDriveTypeW(windows::core::PCWSTR(wide_drive.as_ptr())) };
    
    // 确保 mount_point 以反斜杠结尾，表示根目录（如 E:\）
    let mount_point = if drive.ends_with('\\') {
        drive.to_string()
    } else {
        format!("{}\\", drive)
    };

    let name = if volume_name_str.is_empty() {
        format!("本地磁盘 ({})", drive.trim_end_matches('\\'))
    } else {
        format!("{} ({})", volume_name_str, drive.trim_end_matches('\\'))
    };

    Ok(DiskInfo {
        name,
        mount_point,
        file_system: file_system_str,
        total_size: total_bytes,
        used_size: used_bytes,
        free_size: free_bytes,
        usage_percent,
        volume_name: volume_name_str,
    })
}
