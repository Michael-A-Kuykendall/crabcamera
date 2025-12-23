//! Tests for headless camera functionality

#[cfg(feature = "headless")]
mod headless_tests {
    use crabcamera::headless::{list_devices, list_formats, CaptureConfig, AudioMode};
    use crabcamera::types::CameraFormat;

    #[test]
    fn test_list_devices_no_panic() {
        // Should not panic even without cameras
        let result = list_devices();
        // We don't assert success since there may be no cameras in test environment
        let _ = result;
    }

    #[test]
    fn test_capture_config_creation() {
        let config = CaptureConfig::new("test".to_string(), CameraFormat::hd());
        assert_eq!(config.device_id, "test");
        assert!(matches!(config.audio_mode, AudioMode::Disabled));
    }

    #[test]
    fn test_list_formats_for_invalid_device() {
        let result = list_formats("nonexistent");
        assert!(result.is_err());
    }
}