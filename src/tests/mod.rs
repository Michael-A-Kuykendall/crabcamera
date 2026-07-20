// Basic test infrastructure only - complex tests removed for v0.2.0 release
// Focus on core functionality that actually works

use crate::errors::CameraError;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, Platform};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Mock camera system for testing
#[derive(Clone)]
pub struct MockCameraSystem {
    devices: Arc<Mutex<Vec<CameraDeviceInfo>>>,
    capture_mode: Arc<Mutex<MockCaptureMode>>,
    error_mode: Arc<Mutex<Option<CameraError>>>,
}

/// Detailed mock capture behavior.
#[derive(Debug, Clone)]
pub enum MockCaptureMode {
    /// Return a valid frame.
    Success,
    /// Return an error.
    Failure,
    /// Delay before returning a frame.
    SlowCapture,
}

impl MockCameraSystem {
    /// Initialize a new mock camera system registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add mock devices for testing
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn add_mock_devices(&self, platform: Platform) {
        let mut devices = self.devices.lock().expect("Devices mutex poisoned");
        devices.clear();

        let test_devices = match platform {
            Platform::Windows => vec![
                create_mock_device("win_cam_0", "Integrated Camera", platform),
                create_mock_device("win_cam_1", "USB Webcam", platform),
            ],
            Platform::MacOS => vec![
                create_mock_device("mac_cam_0", "FaceTime HD Camera", platform),
                create_mock_device("mac_cam_1", "External Camera", platform),
            ],
            Platform::Linux => vec![
                create_mock_device("v4l_0", "/dev/video0", platform),
                create_mock_device("v4l_1", "/dev/video1", platform),
            ],
            Platform::Unknown => vec![create_mock_device("unknown_0", "Generic Camera", platform)],
        };

        devices.extend(test_devices);
    }

    /// Get all devices
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn get_devices(&self) -> Vec<CameraDeviceInfo> {
        self.devices.lock().unwrap().clone()
    }

    /// Set capture mode
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn set_capture_mode(&self, mode: MockCaptureMode) {
        *self.capture_mode.lock().expect("Capture mode mutex poisoned") = mode;
    }

    /// Set error mode
    ///
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn set_error_mode(&self, error: Option<CameraError>) {
        *self.error_mode.lock().expect("Error mode mutex poisoned") = error;
    }
}

impl Default for MockCameraSystem {
    fn default() -> Self {
        Self {
            devices: Arc::new(Mutex::new(Vec::new())),
            capture_mode: Arc::new(Mutex::new(MockCaptureMode::Success)),
            error_mode: Arc::new(Mutex::new(None)),
        }
    }
}

/// Helper function to create mock camera device
pub fn create_mock_device(id: &str, name: &str, platform: Platform) -> CameraDeviceInfo {
    CameraDeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!(
            "Mock camera device for {} on {}",
            name,
            platform.as_str()
        )),
        platform,
        is_available: true,
        supports_formats: get_test_formats(),
    }
}

/// Get standard test formats
pub fn get_test_formats() -> Vec<CameraFormat> {
    vec![
        CameraFormat::low(),
        CameraFormat::standard(),
        CameraFormat::hd(),
    ]
}

/// Create mock camera frame
pub fn create_mock_frame(device_id: &str) -> CameraFrame {
    let width = 1280;
    let height = 720;
    let data = vec![128u8; (width * height * 3) as usize]; // RGB8 mock data

    CameraFrame {
        id: Uuid::new_v4().to_string(),
        device_id: device_id.to_string(),
        timestamp: Utc::now(),
        width,
        height,
        format: "RGB8".to_string(),
        data,
        size_bytes: (width * height * 3) as usize,
        metadata: crate::types::FrameMetadata::default(),
    }
}

/// Setup test environment
#[allow(clippy::unused_async)]
pub async fn setup_test_environment() -> MockCameraSystem {
    let mock_system = MockCameraSystem::new();
    mock_system.add_mock_devices(Platform::current());
    mock_system
}

/// Initialize test environment
pub fn init_test_env() {
    let _ = env_logger::builder().is_test(true).try_init();
}

// Mock camera mode storage for testing
use std::collections::HashMap;
use std::sync::LazyLock;
static MOCK_CAMERA_MODES: LazyLock<Arc<Mutex<HashMap<String, MockCaptureMode>>>> =
    LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Set mock camera mode for testing
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
pub fn set_mock_camera_mode(device_id: &str, mode: MockCaptureMode) {
    let mut modes = MOCK_CAMERA_MODES
        .lock()
        .expect("MOCK_CAMERA_MODES mutex poisoned");
    modes.insert(device_id.to_string(), mode);
}

/// Get mock camera mode for testing
///
/// # Panics
///
/// Panics if the internal mutex is poisoned.
#[allow(clippy::unwrap_used)] // Default fallback is intentional
pub fn get_mock_camera_mode(device_id: &str) -> MockCaptureMode {
    let modes = MOCK_CAMERA_MODES
        .lock()
        .expect("MOCK_CAMERA_MODES mutex poisoned");
    modes
        .get(device_id)
        .cloned()
        .unwrap_or(MockCaptureMode::Success)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_system_default_and_mode_setters() {
        let system = MockCameraSystem::new();

        system.set_capture_mode(MockCaptureMode::SlowCapture);
        system.set_error_mode(Some(CameraError::CaptureError("x".to_string())));

        // Default starts empty until devices are added.
        assert!(system.get_devices().is_empty());
    }

    #[test]
    fn test_add_mock_devices_per_platform() {
        let system = MockCameraSystem::new();

        system.add_mock_devices(Platform::Windows);
        assert_eq!(system.get_devices().len(), 2);

        system.add_mock_devices(Platform::MacOS);
        assert_eq!(system.get_devices().len(), 2);

        system.add_mock_devices(Platform::Linux);
        assert_eq!(system.get_devices().len(), 2);

        system.add_mock_devices(Platform::Unknown);
        assert_eq!(system.get_devices().len(), 1);
    }

    #[test]
    fn test_create_mock_device_and_formats() {
        let device = create_mock_device("cam-1", "Mock Cam", Platform::Windows);
        assert_eq!(device.id, "cam-1");
        assert_eq!(device.name, "Mock Cam");
        assert!(device.is_available);
        assert_eq!(device.platform, Platform::Windows);
        assert_eq!(device.supports_formats.len(), 3);

        let formats = get_test_formats();
        assert_eq!(formats.len(), 3);
    }

    #[test]
    fn test_create_mock_frame_shape() {
        let frame = create_mock_frame("device-a");
        assert_eq!(frame.device_id, "device-a");
        assert_eq!(frame.width, 1280);
        assert_eq!(frame.height, 720);
        assert_eq!(frame.format, "RGB8");
        assert_eq!(frame.size_bytes, frame.data.len());
        assert!(!frame.id.is_empty());
    }

    #[tokio::test]
    async fn test_setup_test_environment_creates_devices() {
        let env = setup_test_environment().await;
        assert!(!env.get_devices().is_empty());
    }

    #[test]
    fn test_mock_camera_mode_registry() {
        let id = "mode-cam";

        assert!(matches!(
            get_mock_camera_mode(id),
            MockCaptureMode::Success
        ));

        set_mock_camera_mode(id, MockCaptureMode::Failure);
        assert!(matches!(
            get_mock_camera_mode(id),
            MockCaptureMode::Failure
        ));
    }

    #[test]
    fn test_init_test_env_is_idempotent() {
        init_test_env();
        init_test_env();
    }
}
