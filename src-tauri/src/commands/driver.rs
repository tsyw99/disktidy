use crate::models::DriverInfo;
use crate::modules;
use tauri::command;

#[command]
pub async fn driver_get_list() -> Result<Vec<DriverInfo>, String> {
    modules::get_driver_list()
}

#[command]
pub async fn driver_delete(inf_name: String) -> Result<String, String> {
    modules::delete_driver(&inf_name)?;
    Ok(format!("驱动 {} 已成功删除", inf_name))
}

#[command]
pub async fn driver_get_by_id(id: String) -> Result<Option<DriverInfo>, String> {
    let drivers = modules::get_driver_list()?;
    Ok(modules::get_driver_by_id(&drivers, &id))
}
