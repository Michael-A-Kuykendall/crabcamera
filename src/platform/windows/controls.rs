use crate::types::{CameraControls, CameraCapabilities, WhiteBalance};
use crate::errors::CameraError;
use windows::Win32::Media::DirectShow::{
    IAMCameraControl, IAMVideoProcAmp,
};
use windows::Win32::Media::MediaFoundation::{
    IMFMediaSource,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows::core::Interface;

/// Control range information for normalization
#[derive(Debug, Clone)]
pub struct ControlRange {
    pub min: i32,
    pub max: i32,
    pub step: i32,
    pub default: i32,
}

/// MediaFoundation camera controls interface
pub struct MediaFoundationControls {
    device_index: u32,
    camera_control: Option<IAMCameraControl>,
    video_proc_amp: Option<IAMVideoProcAmp>,
    // Cache control ranges for efficiency
    focus_range: Option<ControlRange>,
    exposure_range: Option<ControlRange>,
    brightness_range: Option<ControlRange>,
    contrast_range: Option<ControlRange>,
    saturation_range: Option<ControlRange>,
    white_balance_range: Option<ControlRange>,
}

impl MediaFoundationControls {
    /// Create new MediaFoundation controls interface for device
    pub fn new(device_index: u32) -> Result<Self, CameraError> {
        log::debug!("Initializing MediaFoundation controls for device {}", device_index);
        
        // Initialize COM
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() {
                return Err(CameraError::InitializationError("COM initialization failed".to_string()));
            }
        }
        
        // Try to find MediaFoundation device (simplified for now)
        let (camera_control, video_proc_amp) = match Self::find_media_source(device_index) {
            Ok(media_source) => {
                // Query for control interfaces from MediaFoundation
                let camera_control = media_source.cast::<IAMCameraControl>().ok();
                let video_proc_amp = media_source.cast::<IAMVideoProcAmp>().ok();
                (camera_control, video_proc_amp)
            }
            Err(_) => {
                // MediaFoundation device discovery failed - continue without interfaces for now
                log::warn!("MediaFoundation device discovery failed - controls will be stubs");
                (None, None)
            }
        };
        
        let mut controls = MediaFoundationControls {
            device_index,
            camera_control,
            video_proc_amp,
            focus_range: None,
            exposure_range: None,
            brightness_range: None,
            contrast_range: None,
            saturation_range: None,
            white_balance_range: None,
        };
        
        // Cache control ranges for efficiency
        controls.cache_control_ranges()?;
        
        log::info!("MediaFoundation controls initialized for device {} - camera_control: {}, video_proc_amp: {}", 
            device_index, 
            controls.camera_control.is_some(), 
            controls.video_proc_amp.is_some()
        );
        
        Ok(controls)
    }

    /// Apply camera controls using MediaFoundation APIs
    pub fn apply_controls(&mut self, controls: &CameraControls) -> Result<Vec<String>, CameraError> {
        let mut unsupported = Vec::new();
        
        // Focus controls
        if let Some(auto_focus) = controls.auto_focus {
            match self.set_auto_focus(auto_focus) {
                Ok(_) => log::debug!("Set auto focus: {}", auto_focus),
                Err(e) => {
                    log::warn!("Auto focus not supported: {}", e);
                    unsupported.push("auto_focus".to_string());
                }
            }
        }
        
        if let Some(focus_distance) = controls.focus_distance {
            match self.set_focus_distance(focus_distance) {
                Ok(_) => log::debug!("Set focus distance: {}", focus_distance),
                Err(e) => {
                    log::warn!("Focus distance not supported: {}", e);
                    unsupported.push("focus_distance".to_string());
                }
            }
        }

        // Exposure controls
        if let Some(auto_exposure) = controls.auto_exposure {
            match self.set_auto_exposure(auto_exposure) {
                Ok(_) => log::debug!("Set auto exposure: {}", auto_exposure),
                Err(e) => {
                    log::warn!("Auto exposure not supported: {}", e);
                    unsupported.push("auto_exposure".to_string());
                }
            }
        }

        if let Some(exposure_time) = controls.exposure_time {
            match self.set_exposure_time(exposure_time) {
                Ok(_) => log::debug!("Set exposure time: {}s", exposure_time),
                Err(e) => {
                    log::warn!("Exposure time not supported: {}", e);
                    unsupported.push("exposure_time".to_string());
                }
            }
        }

        // Video processing controls
        if let Some(ref white_balance) = controls.white_balance {
            match self.set_white_balance(white_balance) {
                Ok(_) => log::debug!("Set white balance: {:?}", white_balance),
                Err(e) => {
                    log::warn!("White balance not supported: {}", e);
                    unsupported.push("white_balance".to_string());
                }
            }
        }

        if let Some(brightness) = controls.brightness {
            match self.set_brightness(brightness) {
                Ok(_) => log::debug!("Set brightness: {}", brightness),
                Err(e) => {
                    log::warn!("Brightness not supported: {}", e);
                    unsupported.push("brightness".to_string());
                }
            }
        }

        if let Some(contrast) = controls.contrast {
            match self.set_contrast(contrast) {
                Ok(_) => log::debug!("Set contrast: {}", contrast),
                Err(e) => {
                    log::warn!("Contrast not supported: {}", e);
                    unsupported.push("contrast".to_string());
                }
            }
        }

        if let Some(saturation) = controls.saturation {
            match self.set_saturation(saturation) {
                Ok(_) => log::debug!("Set saturation: {}", saturation),
                Err(e) => {
                    log::warn!("Saturation not supported: {}", e);
                    unsupported.push("saturation".to_string());
                }
            }
        }

        // Return list of unsupported controls for user feedback
        Ok(unsupported)
    }

    /// Get current camera control values
    pub fn get_controls(&self) -> Result<CameraControls, CameraError> {
        let mut controls = CameraControls::default();
        
        // Read camera controls
        if let Some(ref camera_control) = self.camera_control {
            // Get focus settings
            if let Ok((value, flags)) = self.get_camera_control_value(0) { // Focus property
                if flags == 0x0001 { // Auto flag
                    controls.auto_focus = Some(true);
                } else if let Some(ref range) = self.focus_range {
                    controls.auto_focus = Some(false);
                    controls.focus_distance = Some(device_to_normalized_range(value, range));
                }
            }
            
            // Get exposure settings
            if let Ok((value, flags)) = self.get_camera_control_value(1) { // Exposure property
                if flags == 0x0001 { // Auto flag
                    controls.auto_exposure = Some(true);
                } else if let Some(ref range) = self.exposure_range {
                    controls.auto_exposure = Some(false);
                    // Convert from log base 2 back to seconds
                    let log_exposure = device_to_normalized_range(value, range);
                    controls.exposure_time = Some(2.0_f32.powf(log_exposure));
                }
            }
        }
        
        // Read video processing controls
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            // Get brightness
            if let Some(ref range) = self.brightness_range {
                if let Ok((value, _)) = self.get_video_proc_value(0) { // Brightness property
                    controls.brightness = Some(device_to_normalized_range(value, range));
                }
            }
            
            // Get contrast
            if let Some(ref range) = self.contrast_range {
                if let Ok((value, _)) = self.get_video_proc_value(1) { // Contrast property
                    controls.contrast = Some(device_to_normalized_range(value, range));
                }
            }
            
            // Get saturation
            if let Some(ref range) = self.saturation_range {
                if let Ok((value, _)) = self.get_video_proc_value(3) { // Saturation property
                    controls.saturation = Some(device_to_normalized_range(value, range));
                }
            }
            
            // Get white balance
            if let Ok((value, flags)) = self.get_video_proc_value(4) { // White balance property
                if flags == 0x0001 { // Auto flag
                    controls.white_balance = Some(WhiteBalance::Auto);
                } else {
                    controls.white_balance = Some(WhiteBalance::Custom(value as u32));
                }
            }
        }
        
        Ok(controls)
    }

    /// Test camera capabilities and return supported features
    pub fn get_capabilities(&self) -> Result<CameraCapabilities, CameraError> {
        let mut capabilities = CameraCapabilities {
            supports_auto_focus: false,
            supports_manual_focus: false,
            supports_auto_exposure: false,
            supports_manual_exposure: false,
            supports_white_balance: false,
            supports_zoom: false,
            supports_flash: false,
            supports_burst_mode: true, // Supported by capture mechanism
            supports_hdr: false,
            max_resolution: (1920, 1080), // Default, could be queried
            max_fps: 30.0, // Default, could be queried
            exposure_range: None,
            iso_range: None,
            focus_range: None,
        };
        
        // Test camera control capabilities
        if let Some(ref camera_control) = self.camera_control {
            // Test focus support
            if self.test_camera_control_support(0) { // Focus property
                capabilities.supports_auto_focus = true;
                capabilities.supports_manual_focus = true;
                if let Some(ref range) = self.focus_range {
                    capabilities.focus_range = Some((0.0, 1.0)); // Normalized range
                }
            }
            
            // Test exposure support
            if self.test_camera_control_support(1) { // Exposure property
                capabilities.supports_auto_exposure = true;
                capabilities.supports_manual_exposure = true;
                if let Some(ref range) = self.exposure_range {
                    // Convert device range to approximate seconds range
                    let min_seconds = 2.0_f32.powf(device_to_normalized_range(range.min, range));
                    let max_seconds = 2.0_f32.powf(device_to_normalized_range(range.max, range));
                    capabilities.exposure_range = Some((min_seconds, max_seconds));
                }
            }
            
            // Test zoom support
            if self.test_camera_control_support(2) { // Zoom property
                capabilities.supports_zoom = true;
            }
        }
        
        // Test video processing capabilities
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if self.test_video_proc_support(4) { // White balance property
                capabilities.supports_white_balance = true;
            }
        }
        
        log::debug!("Camera capabilities: focus({}/{}), exposure({}/{}), white_balance({}), zoom({})",
            capabilities.supports_auto_focus, capabilities.supports_manual_focus,
            capabilities.supports_auto_exposure, capabilities.supports_manual_exposure,
            capabilities.supports_white_balance, capabilities.supports_zoom
        );
        
        Ok(capabilities)
    }

    // Individual control implementation methods (stubs for now)
    
    fn set_auto_focus(&mut self, enabled: bool) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            // Note: Using integer constants for now
            let flags = if enabled { 0x0001 } else { 0x0002 }; // Auto vs Manual flags
            
            unsafe {
                camera_control.Set(
                    0, // Focus property
                    0, // Value doesn't matter for auto mode
                    flags,
                ).map_err(|e| CameraError::ControlError(format!("Failed to set auto focus: {}", e)))?;
            }
            
            log::debug!("Set auto focus: {}", enabled);
            Ok(())
        } else {
            Err(CameraError::ControlError("Camera control interface not available".to_string()))
        }
    }

    fn set_focus_distance(&mut self, distance: f32) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            if let Some(ref range) = self.focus_range {
                let device_value = normalize_to_device_range(distance, range);
                
                unsafe {
                    camera_control.Set(
                        0, // Focus property
                        device_value,
                        0x0002, // Manual flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set focus distance: {}", e)))?;
                }
                
                log::debug!("Set focus distance: {} (device value: {})", distance, device_value);
                Ok(())
            } else {
                Err(CameraError::ControlError("Focus range not available".to_string()))
            }
        } else {
            Err(CameraError::ControlError("Camera control interface not available".to_string()))
        }
    }

    fn set_auto_exposure(&mut self, enabled: bool) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            let flags = if enabled { 0x0001 } else { 0x0002 }; // Auto vs Manual flags
            
            unsafe {
                camera_control.Set(
                    1, // Exposure property
                    0, // Value doesn't matter for auto mode
                    flags,
                ).map_err(|e| CameraError::ControlError(format!("Failed to set auto exposure: {}", e)))?;
            }
            
            log::debug!("Set auto exposure: {}", enabled);
            Ok(())
        } else {
            Err(CameraError::ControlError("Camera control interface not available".to_string()))
        }
    }

    fn set_exposure_time(&mut self, time_seconds: f32) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            if let Some(ref range) = self.exposure_range {
                // Convert seconds to device-specific exposure units
                // Note: MediaFoundation exposure is often in log base 2 seconds
                let log_exposure = time_seconds.log2();
                let device_value = normalize_to_device_range(log_exposure, range);
                
                unsafe {
                    camera_control.Set(
                        1, // Exposure property
                        device_value,
                        0x0002, // Manual flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set exposure time: {}", e)))?;
                }
                
                log::debug!("Set exposure time: {}s (log2: {}, device value: {})", time_seconds, log_exposure, device_value);
                Ok(())
            } else {
                Err(CameraError::ControlError("Exposure range not available".to_string()))
            }
        } else {
            Err(CameraError::ControlError("Camera control interface not available".to_string()))
        }
    }

    fn set_white_balance(&mut self, wb: &WhiteBalance) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            let kelvin_temp = white_balance_to_kelvin(wb);
            
            if kelvin_temp == -1 {
                // Auto white balance
                unsafe {
                    video_proc_amp.Set(
                        4, // White balance property
                        0,
                        0x0001, // Auto flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set auto white balance: {}", e)))?;
                }
                log::debug!("Set white balance: Auto");
            } else {
                // Manual white balance with Kelvin temperature
                unsafe {
                    video_proc_amp.Set(
                        4, // White balance property
                        kelvin_temp,
                        0x0002, // Manual flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set white balance: {}", e)))?;
                }
                log::debug!("Set white balance: {}K", kelvin_temp);
            }
            
            Ok(())
        } else {
            Err(CameraError::ControlError("Video processing interface not available".to_string()))
        }
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if let Some(ref range) = self.brightness_range {
                let device_value = normalize_to_device_range(brightness, range);
                
                unsafe {
                    video_proc_amp.Set(
                        0, // Brightness property
                        device_value,
                        0x0002, // Manual flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set brightness: {}", e)))?;
                }
                
                log::debug!("Set brightness: {} (device value: {})", brightness, device_value);
                Ok(())
            } else {
                Err(CameraError::ControlError("Brightness range not available".to_string()))
            }
        } else {
            Err(CameraError::ControlError("Video processing interface not available".to_string()))
        }
    }

    fn set_contrast(&mut self, contrast: f32) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if let Some(ref range) = self.contrast_range {
                let device_value = normalize_to_device_range(contrast, range);
                
                unsafe {
                    video_proc_amp.Set(
                        1, // Contrast property
                        device_value,
                        0x0002, // Manual flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set contrast: {}", e)))?;
                }
                
                log::debug!("Set contrast: {} (device value: {})", contrast, device_value);
                Ok(())
            } else {
                Err(CameraError::ControlError("Contrast range not available".to_string()))
            }
        } else {
            Err(CameraError::ControlError("Video processing interface not available".to_string()))
        }
    }

    fn set_saturation(&mut self, saturation: f32) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if let Some(ref range) = self.saturation_range {
                let device_value = normalize_to_device_range(saturation, range);
                
                unsafe {
                    video_proc_amp.Set(
                        3, // Saturation property
                        device_value,
                        0x0002, // Manual flag
                    ).map_err(|e| CameraError::ControlError(format!("Failed to set saturation: {}", e)))?;
                }
                
                log::debug!("Set saturation: {} (device value: {})", saturation, device_value);
                Ok(())
            } else {
                Err(CameraError::ControlError("Saturation range not available".to_string()))
            }
        } else {
            Err(CameraError::ControlError("Video processing interface not available".to_string()))
        }
    }
    
    // Helper methods for MediaFoundation device discovery and interface management
    
    /// Find MediaFoundation media source for the specified device index
    /// SIMPLIFIED: Returns a stub for now - device discovery will be implemented later
    fn find_media_source(_device_index: u32) -> Result<IMFMediaSource, CameraError> {
        // TODO: Implement full MediaFoundation device discovery
        // For now, return an error to allow compilation
        Err(CameraError::InitializationError(
            "MediaFoundation device discovery not yet implemented - using nokhwa for capture".to_string()
        ))
    }
    
    /// Cache control ranges for efficient value conversion
    fn cache_control_ranges(&mut self) -> Result<(), CameraError> {
        // Cache camera control ranges
        if let Some(ref camera_control) = self.camera_control {
            self.focus_range = self.query_camera_control_range(0); // Focus property
            self.exposure_range = self.query_camera_control_range(1); // Exposure property
        }
        
        // Cache video processing ranges
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            self.brightness_range = self.query_video_proc_range(0); // Brightness property
            self.contrast_range = self.query_video_proc_range(1); // Contrast property
            self.saturation_range = self.query_video_proc_range(3); // Saturation property
            self.white_balance_range = self.query_video_proc_range(4); // White balance property
        }
        
        log::debug!("Cached control ranges - focus: {}, exposure: {}, brightness: {}, contrast: {}, saturation: {}, white_balance: {}",
            self.focus_range.is_some(),
            self.exposure_range.is_some(),
            self.brightness_range.is_some(),
            self.contrast_range.is_some(),
            self.saturation_range.is_some(),
            self.white_balance_range.is_some()
        );
        
        Ok(())
    }
    
    /// Query camera control range
    fn query_camera_control_range(&self, property: i32) -> Option<ControlRange> {
        if let Some(ref camera_control) = self.camera_control {
            unsafe {
                let mut min = 0i32;
                let mut max = 0i32;
                let mut step = 0i32;
                let mut default = 0i32;
                let mut flags = 0i32;
                
                if camera_control.GetRange(property, &mut min, &mut max, &mut step, &mut default, &mut flags).is_ok() {
                    return Some(ControlRange { min, max, step, default });
                }
            }
        }
        None
    }
    
    /// Query video processing range
    fn query_video_proc_range(&self, property: i32) -> Option<ControlRange> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            unsafe {
                let mut min = 0i32;
                let mut max = 0i32;
                let mut step = 0i32;
                let mut default = 0i32;
                let mut flags = 0i32;
                
                if video_proc_amp.GetRange(property, &mut min, &mut max, &mut step, &mut default, &mut flags).is_ok() {
                    return Some(ControlRange { min, max, step, default });
                }
            }
        }
        None
    }
    
    /// Get current camera control value and flags
    fn get_camera_control_value(&self, property: i32) -> Result<(i32, i32), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            unsafe {
                let mut value = 0i32;
                let mut flags = 0i32;
                
                camera_control.Get(property, &mut value, &mut flags)
                    .map_err(|e| CameraError::ControlError(format!("Failed to get camera control value: {}", e)))?;
                
                Ok((value, flags))
            }
        } else {
            Err(CameraError::ControlError("Camera control interface not available".to_string()))
        }
    }
    
    /// Get current video processing value and flags
    fn get_video_proc_value(&self, property: i32) -> Result<(i32, i32), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            unsafe {
                let mut value = 0i32;
                let mut flags = 0i32;
                
                video_proc_amp.Get(property, &mut value, &mut flags)
                    .map_err(|e| CameraError::ControlError(format!("Failed to get video proc value: {}", e)))?;
                
                Ok((value, flags))
            }
        } else {
            Err(CameraError::ControlError("Video processing interface not available".to_string()))
        }
    }
    
    /// Test if a camera control is supported
    fn test_camera_control_support(&self, property: i32) -> bool {
        self.query_camera_control_range(property).is_some()
    }
    
    /// Test if a video processing control is supported
    fn test_video_proc_support(&self, property: i32) -> bool {
        self.query_video_proc_range(property).is_some()
    }
}

// Proper resource cleanup for COM interfaces
impl Drop for MediaFoundationControls {
    fn drop(&mut self) {
        // COM interfaces are automatically released when dropped in the windows crate
        // but we should uninitialize COM if we initialized it
        unsafe {
            CoUninitialize();
        }
        log::debug!("MediaFoundation controls resources cleaned up for device {}", self.device_index);
    }
}

// SAFETY: MediaFoundationControls manages COM interfaces that are apartment-threaded.
// We ensure thread safety by:
// 1. COM interfaces are properly initialized with COINIT_APARTMENTTHREADED
// 2. All access is synchronized through the containing WindowsCamera
// 3. Windows crate provides proper COM interface management
unsafe impl Send for MediaFoundationControls {}
unsafe impl Sync for MediaFoundationControls {}

// Helper functions for value conversion

/// Convert normalized value (-1.0 to 1.0) to device-specific range
fn normalize_to_device_range(normalized: f32, range: &ControlRange) -> i32 {
    let device_range = range.max - range.min;
    let normalized_clamped = normalized.clamp(-1.0, 1.0);
    let zero_to_one = (normalized_clamped + 1.0) / 2.0;
    range.min + (zero_to_one * device_range as f32) as i32
}

/// Convert device-specific value to normalized range (-1.0 to 1.0)
fn device_to_normalized_range(device_value: i32, range: &ControlRange) -> f32 {
    let device_range = range.max - range.min;
    let zero_to_one = (device_value - range.min) as f32 / device_range as f32;
    (zero_to_one * 2.0) - 1.0
}

/// Convert WhiteBalance enum to Kelvin temperature for MediaFoundation
fn white_balance_to_kelvin(wb: &WhiteBalance) -> i32 {
    match wb {
        WhiteBalance::Auto => -1,        // Use auto mode
        WhiteBalance::Incandescent => 2700,
        WhiteBalance::Fluorescent => 4200,
        WhiteBalance::Daylight => 5500,
        WhiteBalance::Flash => 5500,
        WhiteBalance::Cloudy => 6500,
        WhiteBalance::Shade => 7500,
        WhiteBalance::Custom(temp) => *temp as i32,
    }
}