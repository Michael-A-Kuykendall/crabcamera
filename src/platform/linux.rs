use crate::errors::CameraError;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams};
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};
use std::sync::{Arc, Mutex};

// Add proper imports for V4L2 format enumeration
use v4l::video::Capture;
use v4l::Device;

/// List available cameras on Linux using both nokhwa for device discovery and v4l for detailed format enumeration
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    // Queries via nokhwa first to get base list
    let cameras = query(nokhwa::utils::ApiBackend::Video4Linux)
        .map_err(|e| CameraError::InitializationError(format!("Failed to query cameras: {}", e)))?;

    let mut device_list = Vec::new();
    for camera_info in cameras {
        let mut device =
            CameraDeviceInfo::new(camera_info.index().to_string(), camera_info.human_name());

        device = device.with_description(camera_info.description().to_string());

        // Use v4l crate to get real supported formats
        let mut formats = Vec::new();
        let device_index = camera_info.index().as_index().unwrap_or(0);
        let path = format!("/dev/video{}", device_index);
        
        if let Ok(dev) = Device::with_path(&path) {
            if let Ok(format_iter) = dev.enum_formats() {
                for fmt_desc in format_iter {
                    // Get resolutions for this format
                    if let Ok(frames) = dev.enum_framesize(fmt_desc.fourcc) {
                        for frame in frames {
                            for stepwise in frame.size.to_stepwise() {
                                // For each resolution, get frame intervals (fps)
                                if let Ok(intervals) = dev.enum_frameintervals(fmt_desc.fourcc, stepwise.max_width, stepwise.max_height) {
                                    for interval in intervals {
                                        // v4l returns interval as fraction (numerator/denominator)
                                        // fps = denominator / numerator
                                        let fps = if interval.interval.numerator != 0 {
                                             interval.interval.denominator as f32 / interval.interval.numerator as f32
                                        } else {
                                            30.0 // Default fallback
                                        };

                                        // Try to map FourCC to format string
                                        let format_str = match &fmt_desc.fourcc.repr {
                                            b"YUYV" => "YUYV",
                                            b"MJPG" => "MJPEG", 
                                            b"RGB3" => "RGB",
                                            other => std::str::from_utf8(other).unwrap_or("UNKNOWN"),
                                        }.to_string();

                                        let cf = CameraFormat::new(
                                            stepwise.max_width, 
                                            stepwise.max_height, 
                                            fps
                                        ).with_format_type(format_str);
                                        
                                        formats.push(cf);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fallback to defaults if real enumeration failed (e.g. permission error) but warn
        if formats.is_empty() {
             log::warn!("Could not enumerate formats for {}, using defaults", path);
             formats = vec![
                CameraFormat::new(1920, 1080, 30.0).with_format_type("YUYV".to_string()),
                CameraFormat::new(1280, 720, 30.0).with_format_type("YUYV".to_string()),
                CameraFormat::new(640, 480, 30.0).with_format_type("YUYV".to_string()),
            ];
        }

        device = device.with_formats(formats);
        device_list.push(device);
    }

    Ok(device_list)
}

/// Initialize camera on Linux with V4L2 backend
pub fn initialize_camera(params: CameraInitParams) -> Result<LinuxCamera, CameraError> {
    let device_index = params
        .device_id
        .parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;

    // Simple format request for V4L2
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);

    let camera = Camera::new(
        nokhwa::utils::CameraIndex::Index(device_index),
        requested_format,
    )
    .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;

    Ok(LinuxCamera {
        camera: Arc::new(Mutex::new(camera)),
        device_id: params.device_id,
        format: params.format,
        callback: Arc::new(Mutex::new(None)),
    })
}

/// Linux-specific camera wrapper
pub struct LinuxCamera {
    camera: Arc<Mutex<Camera>>,
    device_id: String,
    format: CameraFormat,
    callback: Arc<Mutex<Option<Box<dyn Fn(CameraFrame) + Send + 'static>>>>,
}

impl LinuxCamera {
    /// Capture frame from Linux camera using V4L2
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

    /// Get supported V4L2 formats for this device
    pub fn get_supported_formats(&self) -> Result<Vec<CameraFormat>, CameraError> {
        // This would typically query V4L2 for actual supported formats
        // For now, return common formats
        Ok(vec![
            CameraFormat::new(1920, 1080, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(640, 480, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1920, 1080, 15.0).with_format_type("MJPEG".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("MJPEG".to_string()),
        ])
    }

    /// Set camera controls (Linux V4L2 specific)
    pub fn set_control(&self, control: &str, value: i32) -> Result<(), CameraError> {
        let device_index = self.device_id.parse::<usize>().unwrap_or(0);
        let path = format!("/dev/video{}", device_index);
        let dev = Device::with_path(&path).map_err(|e| CameraError::InitializationError(format!("Failed to open device: {}", e)))?;

        // Standard V4L2 CIDs (from videodev2.h)
        const V4L2_CID_BRIGHTNESS: u32 = 0x00980900;
        const V4L2_CID_CONTRAST: u32 = 0x00980901;
        const V4L2_CID_SATURATION: u32 = 0x00980902;
        const V4L2_CID_HUE: u32 = 0x00980903;
        const V4L2_CID_GAMMA: u32 = 0x00980910;
        const V4L2_CID_SHARPNESS: u32 = 0x0098091b;

        let id = match control {
            "brightness" => V4L2_CID_BRIGHTNESS,
            "contrast" => V4L2_CID_CONTRAST,
            "saturation" => V4L2_CID_SATURATION,
            "hue" => V4L2_CID_HUE,
            "gamma" => V4L2_CID_GAMMA,
            "sharpness" => V4L2_CID_SHARPNESS,
            _ => return Err(CameraError::InitializationError(format!("Unsupported control: {}", control))),
        };

        // Create control struct
        let ctrl = v4l::control::Control {
            id,
            value: v4l::control::Value::Integer(value as i64),
        };
        
        dev.set_control(ctrl).map_err(|e| CameraError::InitializationError(format!("Failed to set control {}: {}", control, e)))?;
        
        Ok(())
    }

    /// Get camera controls
    pub fn get_controls(&self) -> Result<crate::types::CameraControls, CameraError> {
        let device_index = self.device_id.parse::<usize>().unwrap_or(0);
        let path = format!("/dev/video{}", device_index);
        
        // Return default if we can't open device (e.g. if it's busy and driver doesn't support multiple handles)
        // But we should try.
        let dev = match Device::with_path(&path) {
            Ok(d) => d,
            Err(_) => return Ok(crate::types::CameraControls::default()),
        };

        const V4L2_CID_BRIGHTNESS: u32 = 0x00980900;
        const V4L2_CID_CONTRAST: u32 = 0x00980901;
        const V4L2_CID_SATURATION: u32 = 0x00980902;
        const V4L2_CID_ZOOM_ABSOLUTE: u32 = 0x009a090d;
        const V4L2_CID_FOCUS_AUTO: u32 = 0x009a090c;
        const V4L2_CID_FOCUS_ABSOLUTE: u32 = 0x009a090a;
        const V4L2_CID_EXPOSURE_AUTO: u32 = 0x009a0901;
        const V4L2_CID_EXPOSURE_ABSOLUTE: u32 = 0x009a0902;
        const V4L2_CID_SHARPNESS: u32 = 0x0098091b;

        // Helper to normalize value: (val - min) / (max - min)
        let get_norm = |id: u32| -> Option<f32> {
             // Query description for range
             if let Ok(controls) = dev.query_controls() {
                 if let Some(desc) = controls.iter().find(|d| d.id == id) {
                     if let Ok(val) = dev.control(id) {
                         match val.value {
                             v4l::control::Value::Integer(v) => {
                                 // Access min/max from description
                                 let min = desc.minimum;
                                 let max = desc.maximum;
                                 if max > min {
                                     Some((v - min) as f32 / (max - min) as f32)
                                 } else {
                                     Some(0.0)
                                 }
                             },
                             _ => None
                         }
                     } else { None }
                 } else { None }
             } else { None }
        };

        // Optimized helper that uses query_control directly if v4l exposed it, but we use query_controls list
        // Actually v4l might not have query_control(id), only query_controls() -> Vec
        
        // Helper to get raw value
        let get_val = |id: u32| -> Option<v4l::control::Value> {
            dev.control(id).map(|c| c.value).ok()
        };

        let auto_focus = get_val(V4L2_CID_FOCUS_AUTO)
             .map(|v| match v { v4l::control::Value::Boolean(b) => Some(b), _ => None }).unwrap_or(None);
             
        let auto_exposure = get_val(V4L2_CID_EXPOSURE_AUTO)
             .map(|v| match v { v4l::control::Value::Integer(i) => Some(i != 1), _ => None }).unwrap_or(None); // 1 is manual usually

        Ok(crate::types::CameraControls {
            auto_focus,
            focus_distance: get_norm(V4L2_CID_FOCUS_ABSOLUTE),
            auto_exposure, // Boolean
            exposure_time: get_norm(V4L2_CID_EXPOSURE_ABSOLUTE), 
            iso_sensitivity: None, // V4L2 ISO handling is complex/device specific
            white_balance: Some(crate::types::WhiteBalance::Auto), // Simplified
            aperture: None,
            zoom: get_norm(V4L2_CID_ZOOM_ABSOLUTE),
            brightness: get_norm(V4L2_CID_BRIGHTNESS),
            contrast: get_norm(V4L2_CID_CONTRAST),
            saturation: get_norm(V4L2_CID_SATURATION),
            sharpness: get_norm(V4L2_CID_SHARPNESS),
            noise_reduction: None,
            image_stabilization: None,
        })
    }

    /// Apply camera controls
    pub fn apply_controls(
        &mut self,
        controls: &crate::types::CameraControls,
    ) -> Result<(), CameraError> {
        let device_index = self.device_id.parse::<usize>().unwrap_or(0);
        let path = format!("/dev/video{}", device_index);
        let dev = Device::with_path(&path).map_err(|e| CameraError::InitializationError(format!("Failed to open device for controls: {}", e)))?;
        
        const V4L2_CID_BRIGHTNESS: u32 = 0x00980900;
        const V4L2_CID_CONTRAST: u32 = 0x00980901;
        const V4L2_CID_SATURATION: u32 = 0x00980902;
        const V4L2_CID_ZOOM_ABSOLUTE: u32 = 0x009a090d;
        const V4L2_CID_FOCUS_AUTO: u32 = 0x009a090c;
        const V4L2_CID_FOCUS_ABSOLUTE: u32 = 0x009a090a;
        const V4L2_CID_EXPOSURE_AUTO: u32 = 0x009a0901;
        const V4L2_CID_EXPOSURE_ABSOLUTE: u32 = 0x009a0902;
        const V4L2_CID_SHARPNESS: u32 = 0x0098091b;

        // Helper to denormalize: min + val * (max - min)
        let set_norm = |id: u32, val: f32| {
            if let Ok(controls) = dev.query_controls() {
                if let Some(desc) = controls.iter().find(|d| d.id == id) {
                    let min = desc.minimum;
                    let max = desc.maximum;
                    let actual = min + (val.clamp(0.0, 1.0) * (max - min) as f32) as i64;
                     
                     let ctrl = v4l::control::Control {
                        id,
                        value: v4l::control::Value::Integer(actual),
                    };
                    let _ = dev.set_control(ctrl);
                }
            }
        };

        if let Some(b) = controls.brightness { set_norm(V4L2_CID_BRIGHTNESS, b); }
        if let Some(c) = controls.contrast { set_norm(V4L2_CID_CONTRAST, c); }
        if let Some(s) = controls.saturation { set_norm(V4L2_CID_SATURATION, s); }
        if let Some(sh) = controls.sharpness { set_norm(V4L2_CID_SHARPNESS, sh); }
        if let Some(z) = controls.zoom { set_norm(V4L2_CID_ZOOM_ABSOLUTE, z); }

        if let Some(af) = controls.auto_focus {
             let ctrl = v4l::control::Control {
                id: V4L2_CID_FOCUS_AUTO,
                value: v4l::control::Value::Boolean(af),
            };
             let _ = dev.set_control(ctrl);
        }
        
        // Manual Focus
        if let Some(fd) = controls.focus_distance {
             if controls.auto_focus != Some(true) {
                 set_norm(V4L2_CID_FOCUS_ABSOLUTE, fd);
             }
        }
        
        // Auto Exposure
        if let Some(ae) = controls.auto_exposure {
             // V4L2_CID_EXPOSURE_AUTO: 0=Auto, 1=Manual (usually, check docs)
             // Actually enum: 0=Auto, 1=Manual, 2=Shutter Prio, 3=Aperture Prio
             // For simplify: 0=Auto, 1=Manual
             let val = if ae { 0 } else { 1 }; 
             let ctrl = v4l::control::Control {
                id: V4L2_CID_EXPOSURE_AUTO,
                value: v4l::control::Value::Integer(val),
            };
             let _ = dev.set_control(ctrl);
        }

        // Manual Exposure
        if let Some(et) = controls.exposure_time {
             if controls.auto_exposure != Some(true) {
                 set_norm(V4L2_CID_EXPOSURE_ABSOLUTE, et);
             } 
        }

        Ok(())
    }

    /// Get camera capabilities (Linux V4L2)
    pub fn test_capabilities(&self) -> Result<crate::types::CameraCapabilities, CameraError> {
        let device_index = self.device_id.parse::<usize>().unwrap_or(0);
        let path = format!("/dev/video{}", device_index);
        let dev = Device::with_path(&path).map_err(|e| CameraError::InitializationError(format!("Failed to open device: {}", e)))?;
        
        let mut caps = crate::types::CameraCapabilities::default();
        
        // Check controls for capabilities
        if let Ok(controls) = dev.query_controls() {
            // Manual Focus
            const V4L2_CID_FOCUS_ABSOLUTE: u32 = 0x009a090a;
            caps.supports_manual_focus = controls.iter().any(|c| c.id == V4L2_CID_FOCUS_ABSOLUTE);
            
            // Manual Exposure
            const V4L2_CID_EXPOSURE_ABSOLUTE: u32 = 0x009a0902;
            caps.supports_manual_exposure = controls.iter().any(|c| c.id == V4L2_CID_EXPOSURE_ABSOLUTE);
            
            // Zoom
            const V4L2_CID_ZOOM_ABSOLUTE: u32 = 0x009a090d;
            caps.supports_zoom = controls.iter().any(|c| c.id == V4L2_CID_ZOOM_ABSOLUTE);

            // Auto Focus/Exposure usually supported if the CID exists for the mode
            const V4L2_CID_FOCUS_AUTO: u32 = 0x009a090c;
            caps.supports_auto_focus = controls.iter().any(|c| c.id == V4L2_CID_FOCUS_AUTO);

             const V4L2_CID_EXPOSURE_AUTO: u32 = 0x009a0901;
             caps.supports_auto_exposure = controls.iter().any(|c| c.id == V4L2_CID_EXPOSURE_AUTO);
        }

        // Get actual ranges/resolutions if possible (requires more complex enumeration)
        if let Ok(formats) = self.get_supported_formats() {
            if let Some(max) = formats.iter().max_by_key(|f| (f.width * f.height) as u64) {
                 caps.max_resolution = (max.width, max.height);
                 caps.max_fps = max.fps;
            }
        }

        Ok(caps)
    }

    /// Get performance metrics (Not implemented)
    pub fn get_performance_metrics(
        &self,
    ) -> Result<crate::types::CameraPerformanceMetrics, CameraError> {
        Err(CameraError::UnsupportedOperation("Performance metrics not yet implemented on Linux".to_string()))
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
impl Drop for LinuxCamera {
    fn drop(&mut self) {
        if let Ok(mut camera) = self.camera.lock() {
            let _ = camera.stop_stream();
        }
    }
}

// Thread-safe implementation
unsafe impl Send for LinuxCamera {}
unsafe impl Sync for LinuxCamera {}

/// Linux-specific utilities
pub mod utils {
    use super::*;

    /// Check if V4L2 is available on the system
    pub fn is_v4l2_available() -> bool {
        std::path::Path::new("/dev/video0").exists()
    }

    /// List all V4L2 devices in /dev/video*
    pub fn list_v4l2_devices() -> Result<Vec<String>, CameraError> {
        let mut devices = Vec::new();

        for i in 0..10 {
            // Check video0 through video9
            let device_path = format!("/dev/video{}", i);
            if std::path::Path::new(&device_path).exists() {
                devices.push(device_path);
            }
        }

        Ok(devices)
    }

    /// Get V4L2 device capabilities
    pub fn get_device_caps(_device_path: &str) -> Result<Vec<String>, CameraError> {
        // This would typically query V4L2 capabilities
        // For now, return common capabilities
        Ok(vec![
            "Video Capture".to_string(),
            "Streaming".to_string(),
            "Extended Controls".to_string(),
        ])
    }
}
