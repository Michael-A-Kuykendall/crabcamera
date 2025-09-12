// Basic test infrastructure only - complex tests removed for v0.2.0 release
// Focus on core functionality that actually works

use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, Platform};
use crate::errors::CameraError;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use chrono::Utc;

/// Mock camera system for testing
#[derive(Clone)]
pub struct MockCameraSystem {
    devices: Arc<Mutex<Vec<CameraDeviceInfo>>>,
    capture_mode: Arc<Mutex<MockCaptureMode>>,
    error_mode: Arc<Mutex<Option<CameraError>>>,
}

#[derive(Debug, Clone)]
pub enum MockCaptureMode {
    Success,
    Failure,
    SlowCapture,
}

impl MockCameraSystem {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(Vec::new())),
            capture_mode: Arc::new(Mutex::new(MockCaptureMode::Success)),
            error_mode: Arc::new(Mutex::new(None)),
        }
    }
    
    pub async fn add_mock_devices(&self, platform: Platform) {
        let mut devices = self.devices.lock().unwrap();
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
            Platform::Unknown => vec![
                create_mock_device("unknown_0", "Generic Camera", platform),
            ],
        };
        
        devices.extend(test_devices);
    }
    
    pub async fn get_devices(&self) -> Vec<CameraDeviceInfo> {
        self.devices.lock().unwrap().clone()
    }
    
    pub fn set_capture_mode(&self, mode: MockCaptureMode) {
        *self.capture_mode.lock().unwrap() = mode;
    }
    
    pub fn set_error_mode(&self, error: Option<CameraError>) {
        *self.error_mode.lock().unwrap() = error;
    }
}

/// Helper function to create mock camera device
pub fn create_mock_device(id: &str, name: &str, platform: Platform) -> CameraDeviceInfo {
    CameraDeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!("Mock camera device for {} on {}", name, platform.as_str())),
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
pub async fn setup_test_environment() -> MockCameraSystem {
    let mock_system = MockCameraSystem::new();
    mock_system.add_mock_devices(Platform::current()).await;
    mock_system
}

/// Initialize test environment
pub fn init_test_env() {
    let _ = env_logger::builder().is_test(true).try_init();
}

// Mock camera mode storage for testing
use std::collections::HashMap;
lazy_static::lazy_static! {
    static ref MOCK_CAMERA_MODES: Arc<Mutex<HashMap<String, MockCaptureMode>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Set mock camera mode for testing
pub fn set_mock_camera_mode(device_id: &str, mode: MockCaptureMode) {
    let mut modes = MOCK_CAMERA_MODES.lock().unwrap();
    modes.insert(device_id.to_string(), mode);
}

/// Get mock camera mode for testing
pub fn get_mock_camera_mode(device_id: &str) -> MockCaptureMode {
    let modes = MOCK_CAMERA_MODES.lock().unwrap();
    modes.get(device_id).cloned().unwrap_or(MockCaptureMode::Success)
}