use crate::config::CrabCameraConfig;
use std::sync::{Arc, RwLock};
use tauri::command;

lazy_static::lazy_static! {
    static ref GLOBAL_CONFIG: Arc<RwLock<CrabCameraConfig>> = Arc::new(RwLock::new(CrabCameraConfig::load_or_default()));
}

/// Get the current configuration
#[command]
pub async fn get_config() -> Result<CrabCameraConfig, String> {
    let config = GLOBAL_CONFIG
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(config.clone())
}

/// Update configuration
#[command]
pub async fn update_config(new_config: CrabCameraConfig) -> Result<String, String> {
    // Validate first
    new_config
        .validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    {
        let mut config = GLOBAL_CONFIG
            .write()
            .map_err(|e| format!("Failed to write config: {}", e))?;
        *config = new_config.clone();
    }

    // Save to file
    new_config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Configuration updated successfully".to_string())
}

/// Reset configuration to defaults
#[command]
pub async fn reset_config() -> Result<CrabCameraConfig, String> {
    let default_config = CrabCameraConfig::default();

    {
        let mut config = GLOBAL_CONFIG
            .write()
            .map_err(|e| format!("Failed to write config: {}", e))?;
        *config = default_config.clone();
    }

    // Save defaults to file
    default_config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(default_config)
}

/// Get camera configuration
#[command]
pub async fn get_camera_config() -> Result<crate::config::CameraConfig, String> {
    let config = GLOBAL_CONFIG
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(config.camera.clone())
}

/// Get quality configuration (full config object)
#[command]
pub async fn get_full_quality_config() -> Result<crate::config::QualityConfig, String> {
    let config = GLOBAL_CONFIG
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(config.quality.clone())
}

/// Get storage configuration
#[command]
pub async fn get_storage_config() -> Result<crate::config::StorageConfig, String> {
    let config = GLOBAL_CONFIG
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(config.storage.clone())
}

/// Get advanced configuration
#[command]
pub async fn get_advanced_config() -> Result<crate::config::AdvancedConfig, String> {
    let config = GLOBAL_CONFIG
        .read()
        .map_err(|e| format!("Failed to read config: {}", e))?;
    Ok(config.advanced.clone())
}

/// Update camera configuration
#[command]
pub async fn update_camera_config(
    camera_config: crate::config::CameraConfig,
) -> Result<String, String> {
    let mut config = GLOBAL_CONFIG
        .write()
        .map_err(|e| format!("Failed to write config: {}", e))?;
    config.camera = camera_config;

    config
        .validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Camera configuration updated".to_string())
}

/// Update quality configuration (full config object)
#[command]
pub async fn update_full_quality_config(
    quality_config: crate::config::QualityConfig,
) -> Result<String, String> {
    let mut config = GLOBAL_CONFIG
        .write()
        .map_err(|e| format!("Failed to write config: {}", e))?;
    config.quality = quality_config;

    config
        .validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Quality configuration updated".to_string())
}

/// Update storage configuration
#[command]
pub async fn update_storage_config(
    storage_config: crate::config::StorageConfig,
) -> Result<String, String> {
    let mut config = GLOBAL_CONFIG
        .write()
        .map_err(|e| format!("Failed to write config: {}", e))?;
    config.storage = storage_config;

    config
        .validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Storage configuration updated".to_string())
}

/// Update advanced configuration
#[command]
pub async fn update_advanced_config(
    advanced_config: crate::config::AdvancedConfig,
) -> Result<String, String> {
    let mut config = GLOBAL_CONFIG
        .write()
        .map_err(|e| format!("Failed to write config: {}", e))?;
    config.advanced = advanced_config;

    config
        .validate()
        .map_err(|e| format!("Configuration validation failed: {}", e))?;

    config
        .save_to_file(CrabCameraConfig::default_path())
        .map_err(|e| format!("Failed to save config: {}", e))?;

    Ok("Advanced configuration updated".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_config() {
        let result = get_config().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.camera.default_fps, 30);
    }

    #[tokio::test]
    async fn test_reset_config() {
        let result = reset_config().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.camera.default_resolution, [1920, 1080]);
    }

    #[tokio::test]
    async fn test_get_camera_config() {
        let result = get_camera_config().await;
        assert!(result.is_ok());

        let camera_config = result.unwrap();
        assert!(camera_config.auto_reconnect);
    }
}
