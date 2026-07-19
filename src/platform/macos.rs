use crate::constants::*;
use crate::errors::CameraError;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams};
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};
use std::sync::{Arc, Mutex};

// Objective-C imports for AVFoundation integration
use objc::{msg_send, sel, sel_impl};
use objc::runtime::{Object, Class, YES, NO};

/// List available cameras on macOS
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let cameras = query(nokhwa::utils::ApiBackend::AVFoundation)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;

    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device =
            CameraDeviceInfo::new(camera_info.index().to_string(), camera_info.human_name());

        device = device.with_description(camera_info.description().to_string());

        // Add common macOS camera formats
        let formats = vec![
            CameraFormat::new(DEFAULT_RESOLUTION_WIDTH, DEFAULT_RESOLUTION_HEIGHT, DEFAULT_FPS),
            CameraFormat::new(FALLBACK_RESOLUTION_WIDTH, FALLBACK_RESOLUTION_HEIGHT, DEFAULT_FPS),
            CameraFormat::new(MIN_RESOLUTION_WIDTH, MIN_RESOLUTION_HEIGHT, DEFAULT_FPS),
        ];
        device = device.with_formats(formats);

        device_list.push(device);
    }

    Ok(device_list)
}

/// Initialize camera on macOS with AVFoundation backend
///
/// Uses nokhwa's CameraFormat API (0.10.x) with MJPEG frame format
/// for broad compatibility across macOS camera hardware.
pub fn initialize_camera(params: CameraInitParams) -> Result<MacOSCamera, CameraError> {
    let device_index = params
        .device_id
        .parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;

    // Create requested format using nokhwa 0.10.x CameraFormat API
    // Note: CameraFormat::new takes (Resolution, FrameFormat, fps)
    // Using MJPEG for broad hardware compatibility on macOS
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(
        nokhwa::utils::CameraFormat::new(
            nokhwa::utils::Resolution::new(params.format.width, params.format.height),
            nokhwa::utils::FrameFormat::MJPEG,
            params.format.fps as u32,
        ),
    ));
    let camera = Camera::new(
        nokhwa::utils::CameraIndex::Index(device_index),
        requested_format,
    )
    .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;

    Ok(MacOSCamera {
        camera: Arc::new(Mutex::new(camera)),
        device_id: params.device_id,
        format: params.format,
        callback: Arc::new(Mutex::new(None)),
    })
}

/// macOS-specific camera wrapper
pub struct MacOSCamera {
    camera: Arc<Mutex<Camera>>,
    device_id: String,
    format: CameraFormat,
    callback: Arc<Mutex<Option<Box<dyn Fn(CameraFrame) + Send + 'static>>>>,
}

// Constants for AVFoundation
const AV_CAPTURE_FOCUS_MODE_LOCKED: i64 = 0;
const AV_CAPTURE_FOCUS_MODE_AUTO: i64 = 1;
const AV_CAPTURE_FOCUS_MODE_CONTINUOUS_AUTO: i64 = 2;

const AV_CAPTURE_EXPOSURE_MODE_LOCKED: i64 = 0;
const AV_CAPTURE_EXPOSURE_MODE_AUTO: i64 = 1;
const AV_CAPTURE_EXPOSURE_MODE_CONTINUOUS_AUTO: i64 = 2;

// Custom AVFoundation helpers
trait AVCaptureDeviceExt {
    fn lock_for_configuration(&self) -> Result<(), CameraError>;
    fn unlock_for_configuration(&self);
    fn set_focus_mode(&self, mode: i64) -> Result<(), CameraError>;
    fn set_exposure_mode(&self, mode: i64) -> Result<(), CameraError>;
    fn set_lens_position(&self, position: f32) -> Result<(), CameraError>;
    // Exposure duration is complex due to CMTime struct passing via msg_send!
    // We omit it for this iteration to ensure stability.
}

// Wrapper struct for raw pointer to impl methods 
struct AVDeviceWrapper(*mut Object);

impl AVDeviceWrapper {
    fn new(device_id: &str) -> Option<Self> {
        unsafe {
            let cls = Class::get("AVCaptureDevice")?;
            // Convert device_id string to NSString
            let ns_string_cls = Class::get("NSString")?;
            let utf8_str = std::ffi::CString::new(device_id).ok()?;
            let ns_uuid: *mut Object = msg_send![ns_string_cls, stringWithUTF8String: utf8_str.as_ptr()];
            
            let device: *mut Object = msg_send![cls, deviceWithUniqueID: ns_uuid];
            
            if device.is_null() {
                None
            } else {
                Some(AVDeviceWrapper(device))
            }
        }
    }
}

impl AVCaptureDeviceExt for AVDeviceWrapper {
    fn lock_for_configuration(&self) -> Result<(), CameraError> {
        let device = self.0;
        unsafe {
            let mut err: *mut Object = std::ptr::null_mut();
            let success: bool = msg_send![device, lockForConfiguration: &mut err];
            if success {
                Ok(())
            } else {
                Err(CameraError::InitializationError("Failed to lock device configuration".to_string()))
            }
        }
    }

    fn unlock_for_configuration(&self) {
        let device = self.0;
        unsafe {
            let _: () = msg_send![device, unlockForConfiguration];
        }
    }

    fn set_focus_mode(&self, mode: i64) -> Result<(), CameraError> {
        let device = self.0;
        unsafe {
            let supported: bool = msg_send![device, isFocusModeSupported: mode];
            if supported {
                let _: () = msg_send![device, setFocusMode: mode];
                Ok(())
            } else {
                log::warn!("Focus mode {} not supported by device", mode);
                Ok(())
            }
        }
    }

    fn set_exposure_mode(&self, mode: i64) -> Result<(), CameraError> {
        let device = self.0;
        unsafe {
             let supported: bool = msg_send![device, isExposureModeSupported: mode];
             if supported {
                 let _: () = msg_send![device, setExposureMode: mode];
                 Ok(())
             } else {
                 log::warn!("Exposure mode {} not supported by device", mode);
                 Ok(())
             }
        }
    }

    fn set_lens_position(&self, position: f32) -> Result<(), CameraError> {
        let device = self.0;
        unsafe {
            // setFocusModeLockedWithLensPosition:completionHandler:
            // We pass null for the handler
            let null_handler: *mut Object = std::ptr::null_mut();
            let _: () = msg_send![device, setFocusModeLockedWithLensPosition: position completionHandler: null_handler];
            Ok(())
        }
    }
}

impl MacOSCamera {
    /// Capture frame from macOS camera using AVFoundation
    pub fn capture_frame(&self) -> Result<CameraFrame, CameraError> {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::CaptureError("Failed to lock camera".to_string()))?;

        let frame = camera
            .frame()
            .map_err(|e| CameraError::CaptureError(format!("Failed to capture frame: {}", e)))?;

        let camera_frame = CameraFrame::new(
            frame.buffer_bytes().to_vec(),
            frame.resolution().width_x,
            frame.resolution().height_y,
            self.device_id.clone(),
        );

        let camera_frame = camera_frame.with_format(format!("{:?}", self.format));

        // Call callback if set
        if let Some(ref cb) = *self.callback.lock().unwrap() {
            cb(camera_frame.clone());
        }

        Ok(camera_frame)
    }

    /// Get current format
    pub fn get_format(&self) -> &CameraFormat {
        &self.format
    }

    /// Get device ID
    pub fn get_device_id(&self) -> &str {
        &self.device_id
    }

    /// Check if camera is available
    pub fn is_available(&self) -> bool {
        self.camera
            .lock()
            .map(|c| c.is_stream_open())
            .unwrap_or(false)
    }

    /// Start camera stream
    pub fn start_stream(&self) -> Result<(), CameraError> {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;

        camera.open_stream().map_err(|e| {
            CameraError::InitializationError(format!("Failed to start stream: {}", e))
        })?;

        Ok(())
    }

    /// Stop camera stream
    pub fn stop_stream(&self) -> Result<(), CameraError> {
        let mut camera = self
            .camera
            .lock()
            .map_err(|_| CameraError::InitializationError("Failed to lock camera".to_string()))?;

        camera.stop_stream().map_err(|e| {
            CameraError::InitializationError(format!("Failed to stop stream: {}", e))
        })?;

        Ok(())
    }

    /// Get camera controls
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        unsafe {
            let wrapper = match AVDeviceWrapper::new(&self.device_id) {
                Some(w) => w,
                None => return Ok(crate::types::CameraControls::default()),
            };
            
            let device = wrapper.0;

            let focus_mode: i64 = msg_send![device, focusMode];
            let exposure_mode: i64 = msg_send![device, exposureMode];
            let lens_position: f32 = msg_send![device, lensPosition];
            let iso: f32 = msg_send![device, ISO];
            
            Ok(crate::types::CameraControls {
                auto_focus: Some(focus_mode == 1 || focus_mode == 2),
                focus_distance: Some(lens_position),
                auto_exposure: Some(exposure_mode == 1 || exposure_mode == 2),
                exposure_time: None,
                iso_sensitivity: Some(iso as u32),
                white_balance: Some(crate::types::WhiteBalance::Auto),
                aperture: None,
                zoom: Some(1.0),
                brightness: Some(0.0),
                contrast: Some(0.0),
                saturation: Some(0.0),
                sharpness: Some(0.0),
                noise_reduction: None,
                image_stabilization: None,
            })
        }
    }

    /// Apply camera controls
    pub fn apply_controls(
        &mut self,
        controls: &crate::types::CameraControls,
    ) -> Result<crate::types::ControlApplicationResult, CameraError> {
        let wrapper = match AVDeviceWrapper::new(&self.device_id) {
            Some(w) => w,
            None => return Err(CameraError::InitializationError("Device not found".to_string())),
        };

        wrapper.lock_for_configuration()?;

        let mut applied = Vec::new();
        let mut rejected = Vec::new();

        // Focus
        if let Some(af) = controls.auto_focus {
            let mode = if af { AV_CAPTURE_FOCUS_MODE_CONTINUOUS_AUTO } else { AV_CAPTURE_FOCUS_MODE_LOCKED };
            match wrapper.set_focus_mode(mode) {
                Ok(_) => applied.push("auto_focus".to_string()),
                Err(e) => {
                    log::warn!("AVFoundation set_focus_mode failed: {}", e);
                    rejected.push("auto_focus".to_string());
                }
            }
        }

        if let Some(dist) = controls.focus_distance {
            match wrapper.set_lens_position(dist) {
                Ok(_) => applied.push("focus_distance".to_string()),
                Err(e) => {
                    log::warn!("AVFoundation set_lens_position failed: {}", e);
                    rejected.push("focus_distance".to_string());
                }
            }
        }

        // Exposure
        if let Some(ae) = controls.auto_exposure {
            let mode = if ae { AV_CAPTURE_EXPOSURE_MODE_CONTINUOUS_AUTO } else { AV_CAPTURE_EXPOSURE_MODE_LOCKED };
            match wrapper.set_exposure_mode(mode) {
                Ok(_) => applied.push("auto_exposure".to_string()),
                Err(e) => {
                    log::warn!("AVFoundation set_exposure_mode failed: {}", e);
                    rejected.push("auto_exposure".to_string());
                }
            }
        }

        wrapper.unlock_for_configuration();

        Ok(crate::types::ControlApplicationResult { applied, rejected })
    }

    /// Test camera capabilities (macOS AVFoundation)
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        let wrapper = match AVDeviceWrapper::new(&self.device_id) {
            Some(w) => w,
            None => return Err(CameraError::InitializationError("Device not found".to_string())),
        };
        
        // Default capabilities structure
        let mut caps = crate::types::CameraCapabilities::default();
        
        unsafe {
            let device = wrapper.0;
            
            // Focus Checks
            caps.supports_manual_focus = msg_send![device, isFocusModeSupported: AV_CAPTURE_FOCUS_MODE_LOCKED];
            caps.supports_auto_focus = msg_send![device, isFocusModeSupported: AV_CAPTURE_FOCUS_MODE_CONTINUOUS_AUTO] 
                                    || msg_send![device, isFocusModeSupported: AV_CAPTURE_FOCUS_MODE_AUTO];
            
            // Exposure Checks
            caps.supports_manual_exposure = msg_send![device, isExposureModeSupported: AV_CAPTURE_EXPOSURE_MODE_LOCKED];
            caps.supports_auto_exposure = msg_send![device, isExposureModeSupported: AV_CAPTURE_EXPOSURE_MODE_CONTINUOUS_AUTO]
                                       || msg_send![device, isExposureModeSupported: AV_CAPTURE_EXPOSURE_MODE_AUTO];
            
            // Format support is currently limited to default resolutions
        }
        
        Ok(caps)
    }

    /// Get performance metrics (Not implemented)
    pub fn get_performance_metrics(
        &self,
    ) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        Err(CameraError::UnsupportedOperation("Performance metrics not yet implemented on macOS".to_string()))
    }

    /// Set frame callback for real-time processing
    pub fn set_callback<F>(&self, callback: F) -> Result<(), CameraError>
    where
        F: Fn(CameraFrame) + Send + 'static,
    {
        *self.callback.lock().unwrap() = Some(Box::new(callback));
        Ok(())
    }
}

// Ensure the camera is properly cleaned up
impl Drop for MacOSCamera {
    fn drop(&mut self) {
        if let Ok(mut camera) = self.camera.lock() {
            let _ = camera.stop_stream();
        }
    }
}

// Thread-safe implementation
unsafe impl Send for MacOSCamera {}
unsafe impl Sync for MacOSCamera {}
