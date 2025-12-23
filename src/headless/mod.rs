pub mod controls;
pub mod errors;
pub mod session;
pub mod types;

pub use controls::{ControlId, ControlInfo, ControlKind, ControlValue};
pub use errors::HeadlessError;
pub use session::HeadlessSession;
pub use types::{AudioMode, AudioPacket, BufferPolicy, CaptureConfig, DeviceInfo, FormatInfo, Frame};

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
