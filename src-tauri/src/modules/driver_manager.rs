use crate::models::{DriverInfo, DriverStatus};
use std::process::Command;
use serde_json::Value;

pub fn get_driver_list() -> Result<Vec<DriverInfo>, String> {
    let output = Command::new("powershell")
        .args([
            "-Command",
            r#"
            $drivers = Get-WmiObject Win32_PnPSignedDriver | Where-Object { $_.DeviceName } | Select-Object DeviceName, DriverVersion, DriverDate, Manufacturer, Status, InfName, IsSigned, DeviceID;
            $devices = Get-PnpDevice -PresentOnly | Select-Object InstanceId, Status;
            $driverMap = @{};
            foreach ($dev in $devices) {
                $driverMap[$dev.InstanceId] = $dev.Status;
            }
            $result = foreach ($d in $drivers) {
                $devStatus = if ($d.DeviceID -and $driverMap.ContainsKey($d.DeviceID)) { $driverMap[$d.DeviceID] } else { $d.Status };
                [PSCustomObject]@{
                    DeviceName = $d.DeviceName
                    DriverVersion = $d.DriverVersion
                    DriverDate = $d.DriverDate
                    Manufacturer = $d.Manufacturer
                    Status = $devStatus
                    InfName = $d.InfName
                    IsSigned = $d.IsSigned
                    DeviceID = $d.DeviceID
                }
            }
            $result | ConvertTo-Json -Depth 3
            "#
        ])
        .output()
        .map_err(|e| format!("执行PowerShell命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("获取驱动列表失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let drivers = parse_driver_info(&stdout)?;
    
    Ok(drivers)
}

fn parse_driver_info(json_str: &str) -> Result<Vec<DriverInfo>, String> {
    let trimmed = json_str.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let json: Value = serde_json::from_str(trimmed)
        .map_err(|e| format!("解析JSON失败: {}", e))?;

    let drivers = match json {
        Value::Array(arr) => arr,
        Value::Object(_) => vec![json],
        _ => return Ok(Vec::new()),
    };

    let mut result = Vec::new();
    for driver in drivers {
        if let Some(device_name) = driver.get("DeviceName").and_then(|v| v.as_str()) {
            if device_name.is_empty() {
                continue;
            }

            let id = driver.get("DeviceID")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let version = driver.get("DriverVersion")
                .and_then(|v| v.as_str())
                .unwrap_or("未知")
                .to_string();

            let date = driver.get("DriverDate")
                .and_then(|v| v.as_str())
                .map(|d| {
                    if d.len() >= 10 {
                        d[..10].replace("T", " ")
                    } else {
                        d.to_string()
                    }
                })
                .unwrap_or_else(|| "未知".to_string());

            let provider = driver.get("Manufacturer")
                .and_then(|v| v.as_str())
                .unwrap_or("未知")
                .to_string();

            let status_str = driver.get("Status")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            let status = match status_str.to_lowercase().as_str() {
                "ok" | "starting" | "running" => DriverStatus::Running,
                "error" | "degraded" | "pred fail" | "nonrecover" | "notpresent" => DriverStatus::Error,
                "stopped" | "stopping" | "disabled" | "service" | "stressed" | "unknown" => DriverStatus::Stopped,
                _ => {
                    if status_str.is_empty() {
                        DriverStatus::Unknown
                    } else {
                        DriverStatus::Running
                    }
                }
            };

            let inf_name = driver.get("InfName")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let signed = driver.get("IsSigned")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let driver_type = determine_driver_type(device_name);

            result.push(DriverInfo {
                id: id.clone(),
                name: device_name.to_string(),
                version,
                date,
                provider,
                status,
                driver_type,
                device_name: device_name.to_string(),
                inf_name,
                signed,
            });
        }
    }

    result.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(result)
}

fn determine_driver_type(device_name: &str) -> String {
    let name_lower = device_name.to_lowercase();
    
    if name_lower.contains("nvidia") || name_lower.contains("amd") || name_lower.contains("intel") && name_lower.contains("graphics") {
        "显卡驱动".to_string()
    } else if name_lower.contains("audio") || name_lower.contains("sound") || name_lower.contains("realtek") && name_lower.contains("audio") {
        "音频驱动".to_string()
    } else if name_lower.contains("network") || name_lower.contains("ethernet") || name_lower.contains("wi-fi") || name_lower.contains("wireless") {
        "网络驱动".to_string()
    } else if name_lower.contains("bluetooth") {
        "蓝牙驱动".to_string()
    } else if name_lower.contains("usb") {
        "USB驱动".to_string()
    } else if name_lower.contains("storage") || name_lower.contains("disk") || name_lower.contains("sata") || name_lower.contains("nvme") {
        "存储驱动".to_string()
    } else if name_lower.contains("keyboard") || name_lower.contains("mouse") || name_lower.contains("hid") {
        "输入设备驱动".to_string()
    } else if name_lower.contains("printer") {
        "打印机驱动".to_string()
    } else if name_lower.contains("camera") || name_lower.contains("webcam") {
        "摄像头驱动".to_string()
    } else {
        "其他驱动".to_string()
    }
}

pub fn delete_driver(inf_name: &str) -> Result<(), String> {
    if inf_name.is_empty() {
        return Err("INF文件名不能为空".to_string());
    }

    let output = Command::new("pnputil")
        .args([
            "/delete-driver",
            inf_name,
            "/uninstall",
            "/force"
        ])
        .output()
        .map_err(|e| format!("执行pnputil命令失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("删除驱动失败: {} {}", stdout, stderr));
    }

    Ok(())
}

pub fn get_driver_by_id(drivers: &[DriverInfo], id: &str) -> Option<DriverInfo> {
    drivers.iter().find(|d| d.id == id).cloned()
}
