/// Camera controls and capabilities
pub mod controls;
/// Headless-specific errors
pub mod errors;
/// Core session management
pub mod session;
/// Headless operation types
pub mod types;

pub use controls::{ControlId, ControlInfo, ControlKind, ControlValue};
pub use errors::HeadlessError;
pub use session::HeadlessSession;
pub use types::{
    AudioMode, AudioPacket, BufferPolicy, CaptureConfig, DeviceInfo, FormatInfo, Frame,
};

/// List all available camera devices.
pub fn list_devices() -> Result<Vec<DeviceInfo>, HeadlessError> {
    crate::platform::CameraSystem::list_cameras().map_err(HeadlessError::backend)
}

/// List formats for the given device.
///
/// Note: currently sourced from the platform-provided device info list.
pub fn list_formats(device_id: &str) -> Result<Vec<FormatInfo>, HeadlessError> {
    let devices = list_devices()?;
    let device = devices
        .into_iter()
        .find(|d| d.id == device_id)
        .ok_or_else(|| HeadlessError::not_found("device", device_id))?;

    Ok(device.supports_formats)
}

/// List deterministic control descriptors (schema-level, not hardware-probed).
pub fn list_controls(_device_id: &str) -> Result<Vec<ControlInfo>, HeadlessError> {
    // Hardware support varies; deterministic listing is the schema we support.
    Ok(controls::all_controls())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    

    #[test]
    fn test_list_controls_returns_schema() {
        let controls = list_controls("any").expect("schema controls should always succeed");
        assert!(controls.len() >= 10);
        assert!(controls.iter().any(|c| c.id == ControlId::AutoFocus));
    }

    #[test]
    fn test_list_formats_not_found_produces_structured_error() {
        let result = list_formats("definitely-missing-device-id");
        assert!(result.is_err());
        let err = result.expect_err("unknown device should not resolve formats");
        assert_eq!(err.kind, errors::HeadlessErrorKind::NotFound);
        assert!(err.message.contains("device not found"));
    }

    #[test]
    fn test_open_with_audio_enabled_without_audio_feature_is_rejected() {
        #[cfg(not(feature = "audio"))]
        {
            let mut cfg = CaptureConfig::new("0".to_string(), CameraFormat::standard());
            cfg.buffer_policy = BufferPolicy::DropOldest { capacity: 2 };
            cfg.audio_mode = AudioMode::Enabled;

            let result = HeadlessSession::open(cfg);
            match result {
                Ok(_) => panic!("audio should be rejected when feature is disabled"),
                Err(err) => assert_eq!(err.kind, errors::HeadlessErrorKind::Unsupported),
            }
        }
    }
}
