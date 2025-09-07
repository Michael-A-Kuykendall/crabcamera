use tauri::command;

#[command]
pub async fn request_camera_permission() -> Result<String, String> {
    Ok("Permission granted".to_string())
}

#[command] 
pub async fn check_camera_permission_status() -> Result<String, String> {
    Ok("Granted".to_string())
}