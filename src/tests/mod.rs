pub mod platform_tests;
pub mod camera_tests;
pub mod commands_tests;
pub mod integration_tests;

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
    
    pub async fn mock_capture(&self, device_id: &str) -> Result<CameraFrame, CameraError> {
        // Check for error mode first
        if let Some(ref error) = *self.error_mode.lock().unwrap() {
            return Err(match error {
                CameraError::PermissionDenied(msg) => CameraError::PermissionDenied(msg.clone()),
                CameraError::CaptureError(msg) => CameraError::CaptureError(msg.clone()),
                CameraError::InitializationError(msg) => CameraError::InitializationError(msg.clone()),
            });
        }
        
        let mode = self.capture_mode.lock().unwrap().clone();
        
        match mode {
            MockCaptureMode::Success => {
                Ok(create_mock_frame(device_id))
            },
            MockCaptureMode::Failure => {
                Err(CameraError::CaptureError("Mock capture failure".to_string()))
            },
            MockCaptureMode::SlowCapture => {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                Ok(create_mock_frame(device_id))
            },
        }
    }
}

/// Helper function to create test camera device
pub fn create_test_camera_device(id: &str, name: &str) -> CameraDeviceInfo {
    CameraDeviceInfo {
        id: id.to_string(),
        name: name.to_string(),
        description: Some(format!("Test camera device: {}", name)),
        platform: Platform::current(),
        is_available: true,
        supports_formats: get_test_formats(),
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

/// Helper function to create test camera format
pub fn create_test_format(width: u32, height: u32, fps: f32) -> CameraFormat {
    CameraFormat {
        width,
        height,
        fps,
        format_type: "RGB8".to_string(),
    }
}

/// Get standard test formats
pub fn get_test_formats() -> Vec<CameraFormat> {
    vec![
        CameraFormat::low(),
        CameraFormat::standard(),
        CameraFormat::hd(),
        create_test_format(3840, 2160, 30.0), // 4K
    ]
}

/// Get plant photography optimized formats
pub fn get_plant_photography_formats() -> Vec<CameraFormat> {
    vec![
        CameraFormat { width: 2592, height: 1944, fps: 15.0, format_type: "RAW".to_string() }, // 5MP
        CameraFormat { width: 3264, height: 2448, fps: 10.0, format_type: "RAW".to_string() }, // 8MP
        CameraFormat { width: 4032, height: 3024, fps: 5.0, format_type: "RAW".to_string() },  // 12MP
        CameraFormat::hd().with_format("RAW".to_string()),
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
    }
}

/// Create mock camera frame with specific dimensions
pub fn create_mock_frame_with_size(device_id: &str, width: u32, height: u32) -> CameraFrame {
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
    }
}

/// Create plant photography optimized mock frame
pub fn create_plant_mock_frame(device_id: &str) -> CameraFrame {
    create_mock_frame_with_size(device_id, 2592, 1944) // 5MP plant photography resolution
}

/// Validate frame quality for plant photography
pub fn validate_plant_frame_quality(frame: &CameraFrame) -> bool {
    // Plant photography quality requirements
    frame.width >= 1280 &&
    frame.height >= 720 &&
    frame.is_valid() &&
    frame.aspect_ratio() >= 1.0 &&
    frame.aspect_ratio() <= 2.0
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

/// Check if running in CI environment
pub fn is_ci_environment() -> bool {
    std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok()
}

/// Helper trait for test frame building
pub trait CameraFrameTestExt {
    fn with_format(self, format: String) -> Self;
}

impl CameraFrameTestExt for CameraFrame {
    fn with_format(mut self, format: String) -> Self {
        self.format = format;
        self
    }
}

/// Helper trait for test format building
pub trait CameraFormatTestExt {
    fn with_format(self, format: String) -> Self;
}

impl CameraFormatTestExt for CameraFormat {
    fn with_format(mut self, format: String) -> Self {
        self.format_type = format;
        self
    }
}