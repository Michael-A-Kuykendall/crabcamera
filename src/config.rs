//! Configuration management for CrabCamera
//!
//! Provides configuration loading, saving, and management for camera settings,
//! quality thresholds, storage preferences, and other runtime options.

use crate::errors::CameraError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrabCameraConfig {
    pub camera: CameraConfig,
    pub quality: QualityConfig,
    pub storage: StorageConfig,
    pub advanced: AdvancedConfig,
}

/// Camera-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Default camera resolution [width, height]
    pub default_resolution: [u32; 2],
    /// Default frames per second
    pub default_fps: u32,
    /// Auto-reconnect on device disconnect
    pub auto_reconnect: bool,
    /// Reconnect retry attempts
    pub reconnect_attempts: u32,
    /// Reconnect delay in milliseconds
    pub reconnect_delay_ms: u64,
}

/// Quality validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Enable automatic quality-based retry
    pub auto_retry_enabled: bool,
    /// Maximum retry attempts for quality capture
    pub max_retry_attempts: u32,
    /// Minimum acceptable blur threshold (0.0-1.0)
    pub min_blur_threshold: f32,
    /// Minimum acceptable exposure score (0.0-1.0)
    pub min_exposure_score: f32,
    /// Minimum overall quality score (0.0-1.0)
    pub min_overall_score: f32,
    /// Retry delay between attempts in milliseconds
    pub retry_delay_ms: u64,
}

/// Storage and file management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Default output directory for captures
    pub output_directory: String,
    /// Auto-organize files by date
    pub auto_organize_by_date: bool,
    /// Date format for organization (e.g., "YYYY-MM-DD")
    pub date_format: String,
    /// Default image format (jpeg, png, bmp)
    pub default_format: String,
    /// JPEG quality (0-100)
    pub jpeg_quality: u8,
    /// Auto-delete low quality captures
    pub auto_delete_low_quality: bool,
}

/// Advanced features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    /// Enable focus stacking
    pub focus_stacking_enabled: bool,
    /// Number of focus steps for stacking
    pub focus_stack_steps: u32,
    /// Enable HDR capture
    pub hdr_enabled: bool,
    /// Number of exposure brackets for HDR
    pub hdr_brackets: u32,
    /// Enable WebRTC streaming
    pub webrtc_enabled: bool,
    /// WebRTC maximum bitrate (kbps)
    pub webrtc_bitrate: u32,
}

impl Default for CrabCameraConfig {
    fn default() -> Self {
        Self {
            camera: CameraConfig {
                default_resolution: [1920, 1080],
                default_fps: 30,
                auto_reconnect: true,
                reconnect_attempts: 3,
                reconnect_delay_ms: 1000,
            },
            quality: QualityConfig {
                auto_retry_enabled: true,
                max_retry_attempts: 10,
                min_blur_threshold: 0.7,
                min_exposure_score: 0.6,
                min_overall_score: 0.7,
                retry_delay_ms: 100,
            },
            storage: StorageConfig {
                output_directory: "./captures".to_string(),
                auto_organize_by_date: true,
                date_format: "YYYY-MM-DD".to_string(),
                default_format: "jpeg".to_string(),
                jpeg_quality: 95,
                auto_delete_low_quality: false,
            },
            advanced: AdvancedConfig {
                focus_stacking_enabled: false,
                focus_stack_steps: 10,
                hdr_enabled: false,
                hdr_brackets: 3,
                webrtc_enabled: false,
                webrtc_bitrate: 2000,
            },
        }
    }
}

impl CrabCameraConfig {
    /// Load configuration from TOML file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, CameraError> {
        let path = path.as_ref();

        if !path.exists() {
            log::info!("Config file not found at {:?}, using defaults", path);
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path).map_err(|e| {
            CameraError::InitializationError(format!("Failed to read config file: {}", e))
        })?;

        let config: CrabCameraConfig = toml::from_str(&contents).map_err(|e| {
            CameraError::InitializationError(format!("Failed to parse config file: {}", e))
        })?;

        log::info!("Loaded configuration from {:?}", path);
        Ok(config)
    }

    /// Save configuration to TOML file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), CameraError> {
        let path = path.as_ref();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CameraError::InitializationError(format!(
                    "Failed to create config directory: {}",
                    e
                ))
            })?;
        }

        let toml_string = toml::to_string_pretty(self).map_err(|e| {
            CameraError::InitializationError(format!("Failed to serialize config: {}", e))
        })?;

        fs::write(path, toml_string).map_err(|e| {
            CameraError::InitializationError(format!("Failed to write config file: {}", e))
        })?;

        log::info!("Saved configuration to {:?}", path);
        Ok(())
    }

    /// Get default config file path
    pub fn default_path() -> PathBuf {
        PathBuf::from("crabcamera.toml")
    }

    /// Load from default location or create with defaults
    pub fn load_or_default() -> Self {
        Self::load_from_file(Self::default_path()).unwrap_or_else(|e| {
            log::warn!("Failed to load config, using defaults: {}", e);
            Self::default()
        })
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), String> {
        // Validate camera config
        if self.camera.default_resolution[0] == 0 || self.camera.default_resolution[1] == 0 {
            return Err("Invalid default resolution".to_string());
        }
        if self.camera.default_fps == 0 || self.camera.default_fps > 240 {
            return Err("Invalid default FPS (must be 1-240)".to_string());
        }

        // Validate quality config
        if !(0.0..=1.0).contains(&self.quality.min_blur_threshold) {
            return Err("Blur threshold must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.quality.min_exposure_score) {
            return Err("Exposure score must be between 0.0 and 1.0".to_string());
        }
        if !(0.0..=1.0).contains(&self.quality.min_overall_score) {
            return Err("Overall score must be between 0.0 and 1.0".to_string());
        }

        // Validate storage config
        if self.storage.jpeg_quality == 0 || self.storage.jpeg_quality > 100 {
            return Err("JPEG quality must be between 1 and 100".to_string());
        }

        // Validate advanced config
        if self.advanced.focus_stack_steps == 0 || self.advanced.focus_stack_steps > 100 {
            return Err("Focus stack steps must be between 1 and 100".to_string());
        }
        if self.advanced.hdr_brackets == 0 || self.advanced.hdr_brackets > 10 {
            return Err("HDR brackets must be between 1 and 10".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CrabCameraConfig::default();
        assert_eq!(config.camera.default_resolution, [1920, 1080]);
        assert_eq!(config.camera.default_fps, 30);
        assert!(config.quality.auto_retry_enabled);
    }

    #[test]
    fn test_config_validation() {
        let config = CrabCameraConfig::default();
        assert!(config.validate().is_ok());

        let mut bad_config = config.clone();
        bad_config.camera.default_resolution = [0, 0];
        assert!(bad_config.validate().is_err());

        let mut bad_quality = CrabCameraConfig::default();
        bad_quality.quality.min_blur_threshold = 1.5;
        assert!(bad_quality.validate().is_err());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("test_crabcamera.toml");

        // Clean up any existing test file
        let _ = fs::remove_file(&config_path);

        let config = CrabCameraConfig::default();
        assert!(config.save_to_file(&config_path).is_ok());

        let loaded = CrabCameraConfig::load_from_file(&config_path).unwrap();
        assert_eq!(loaded.camera.default_fps, config.camera.default_fps);
        assert_eq!(
            loaded.quality.max_retry_attempts,
            config.quality.max_retry_attempts
        );

        // Clean up
        let _ = fs::remove_file(&config_path);
    }

    #[test]
    fn test_config_toml_format() {
        let config = CrabCameraConfig::default();
        let toml_string = toml::to_string_pretty(&config).unwrap();

        // Verify TOML contains expected sections
        assert!(toml_string.contains("[camera]"));
        assert!(toml_string.contains("[quality]"));
        assert!(toml_string.contains("[storage]"));
        assert!(toml_string.contains("[advanced]"));
        assert!(toml_string.contains("default_resolution"));
        assert!(toml_string.contains("auto_retry_enabled"));
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = CrabCameraConfig::load_from_file("nonexistent_file.toml");
        assert!(result.is_ok()); // Should return default
        assert_eq!(result.unwrap().camera.default_fps, 30);
    }
}
