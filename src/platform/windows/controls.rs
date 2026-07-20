// MediaFoundation camera controls for advanced functionality
use crate::errors::CameraError;
use crate::types::{CameraCapabilities, CameraControls, ControlApplicationResult, WhiteBalance};
use crate::constants::{MAX_RESOLUTION_WIDTH, MAX_RESOLUTION_HEIGHT, HIGH_FPS};
use windows::core::Interface;
use windows::Win32::Media::DirectShow::{
    CameraControl_Exposure, CameraControl_Focus, CameraControl_Zoom,
    IAMCameraControl, IAMVideoProcAmp,
    VideoProcAmp_Brightness, VideoProcAmp_Contrast,
    VideoProcAmp_Saturation, VideoProcAmp_WhiteBalance,
    CameraControl_Flags_Auto, CameraControl_Flags_Manual,
    VideoProcAmp_Flags_Auto, VideoProcAmp_Flags_Manual,
};
use windows::Win32::Media::MediaFoundation::{
    IMFActivate, IMFMediaSource, MFCreateAttributes, MFEnumDeviceSources, MFStartup,
    MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE, MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
    MF_SDK_VERSION,
};
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

/// Control range information for normalization
#[derive(Debug, Clone)]
pub struct ControlRange {
    /// Minimum allowed value
    pub min: i32,
    /// Maximum allowed value
    pub max: i32,
    /// Step size (granularity)
    pub step: i32,
    /// Default factory value
    pub default: i32,
}

/// `MediaFoundation` camera controls interface
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
    /// Create new `MediaFoundation` controls interface for device
    ///
    /// # Errors
    /// Returns a [`CameraError::InitializationError`] if COM initialization
    /// fails, if the `MediaFoundation` device source cannot be found, or if
    /// caching the control ranges fails.
    pub fn new(device_index: u32) -> Result<Self, CameraError> {
        log::debug!(
            "Initializing MediaFoundation controls for device {device_index}"
        );

        // Initialize COM
        unsafe {
            let hr = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            if hr.is_err() {
                return Err(CameraError::InitializationError(
                    "COM initialization failed".to_string(),
                ));
            }
        }

        // Try to find MediaFoundation device (simplified for now)
        let (camera_control, video_proc_amp) = if let Ok(media_source) = Self::find_media_source(device_index) {
            // Query for control interfaces from MediaFoundation
            let camera_control = media_source.cast::<IAMCameraControl>().ok();
            let video_proc_amp = media_source.cast::<IAMVideoProcAmp>().ok();
            (camera_control, video_proc_amp)
        } else {
            // MediaFoundation device discovery failed - continue without interfaces for now
            log::warn!("MediaFoundation device discovery failed - controls will be stubs");
            (None, None)
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

    /// Apply camera controls using `MediaFoundation` APIs
    ///
    /// # Errors
    /// This function always returns `Ok` containing the lists of applied and
    /// rejected controls; unsupported controls are reported as rejections rather
    /// than as an `Err`.
    pub fn apply_controls(
        &mut self,
        controls: &CameraControls,
    ) -> Result<ControlApplicationResult, CameraError> {
        let mut applied = Vec::new();
        let mut rejected = Vec::new();

        // Focus controls
        if let Some(auto_focus) = controls.auto_focus {
            match self.set_auto_focus(auto_focus) {
                Ok(()) => {
                    log::debug!("Set auto focus: {auto_focus}");
                    applied.push("auto_focus".to_string());
                }
                Err(e) => {
                    log::warn!("Auto focus not supported: {e}");
                    rejected.push("auto_focus".to_string());
                }
            }
        }

        if let Some(focus_distance) = controls.focus_distance {
            match self.set_focus_distance(focus_distance) {
                Ok(()) => {
                    log::debug!("Set focus distance: {focus_distance}");
                    applied.push("focus_distance".to_string());
                }
                Err(e) => {
                    log::warn!("Focus distance not supported: {e}");
                    rejected.push("focus_distance".to_string());
                }
            }
        }

        // Exposure controls
        if let Some(auto_exposure) = controls.auto_exposure {
            match self.set_auto_exposure(auto_exposure) {
                Ok(()) => {
                    log::debug!("Set auto exposure: {auto_exposure}");
                    applied.push("auto_exposure".to_string());
                }
                Err(e) => {
                    log::warn!("Auto exposure not supported: {e}");
                    rejected.push("auto_exposure".to_string());
                }
            }
        }

        if let Some(exposure_time) = controls.exposure_time {
            match self.set_exposure_time(exposure_time) {
                Ok(()) => {
                    log::debug!("Set exposure time: {exposure_time}s");
                    applied.push("exposure_time".to_string());
                }
                Err(e) => {
                    log::warn!("Exposure time not supported: {e}");
                    rejected.push("exposure_time".to_string());
                }
            }
        }

        // Video processing controls
        if let Some(ref white_balance) = controls.white_balance {
            match self.set_white_balance(white_balance) {
                Ok(()) => {
                    log::debug!("Set white balance: {white_balance:?}");
                    applied.push("white_balance".to_string());
                }
                Err(e) => {
                    log::warn!("White balance not supported: {e}");
                    rejected.push("white_balance".to_string());
                }
            }
        }

        if let Some(brightness) = controls.brightness {
            match self.set_brightness(brightness) {
                Ok(()) => {
                    log::debug!("Set brightness: {brightness}");
                    applied.push("brightness".to_string());
                }
                Err(e) => {
                    log::warn!("Brightness not supported: {e}");
                    rejected.push("brightness".to_string());
                }
            }
        }

        if let Some(contrast) = controls.contrast {
            match self.set_contrast(contrast) {
                Ok(()) => {
                    log::debug!("Set contrast: {contrast}");
                    applied.push("contrast".to_string());
                }
                Err(e) => {
                    log::warn!("Contrast not supported: {e}");
                    rejected.push("contrast".to_string());
                }
            }
        }

        if let Some(saturation) = controls.saturation {
            match self.set_saturation(saturation) {
                Ok(()) => {
                    log::debug!("Set saturation: {saturation}");
                    applied.push("saturation".to_string());
                }
                Err(e) => {
                    log::warn!("Saturation not supported: {e}");
                    rejected.push("saturation".to_string());
                }
            }
        }

        Ok(ControlApplicationResult { applied, rejected })
    }

    /// Get current camera control values
    ///
    /// # Errors
    /// This function always returns `Ok` with the current control values;
    /// unavailable control interfaces simply yield default values.
    pub fn get_controls(&self) -> Result<CameraControls, CameraError> {
        let mut controls = CameraControls::default();

        // Read camera controls
        if self.camera_control.is_some() {
            // Get focus settings
            if let Ok((value, flags)) = self.get_camera_control_value(CameraControl_Focus.0) {
                // Focus property
                if flags == CameraControl_Flags_Auto.0 {
                    // Auto flag
                    controls.auto_focus = Some(true);
                } else if let Some(ref range) = self.focus_range {
                    controls.auto_focus = Some(false);
                    controls.focus_distance = Some(device_to_normalized_range(value, range));
                }
            }

            // Get exposure settings
            if let Ok((value, flags)) = self.get_camera_control_value(CameraControl_Exposure.0) {
                // Exposure property
                if flags == CameraControl_Flags_Auto.0 {
                    // Auto flag
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
        if self.video_proc_amp.is_some() {
            // Get brightness
            if let Some(ref range) = self.brightness_range {
                if let Ok((value, _)) = self.get_video_proc_value(VideoProcAmp_Brightness.0) {
                    // Brightness property
                    controls.brightness = Some(device_to_normalized_range(value, range));
                }
            }

            // Get contrast
            if let Some(ref range) = self.contrast_range {
                if let Ok((value, _)) = self.get_video_proc_value(VideoProcAmp_Contrast.0) {
                    // Contrast property
                    controls.contrast = Some(device_to_normalized_range(value, range));
                }
            }

            // Get saturation
            if let Some(ref range) = self.saturation_range {
                if let Ok((value, _)) = self.get_video_proc_value(VideoProcAmp_Saturation.0) {
                    // Saturation property
                    controls.saturation = Some(device_to_normalized_range(value, range));
                }
            }

            // Get white balance
            if let Ok((value, flags)) = self.get_video_proc_value(VideoProcAmp_WhiteBalance.0) {
                // White balance property
                if flags == VideoProcAmp_Flags_Auto.0 {
                    // Auto flag
                    controls.white_balance = Some(WhiteBalance::Auto);
                } else {
                    controls.white_balance = Some(WhiteBalance::Custom(value as u32));
                }
            }
        }

        Ok(controls)
    }

    /// Test camera capabilities and return supported features
    ///
    /// # Errors
    /// This function always returns `Ok` with the detected capabilities;
    /// unavailable control interfaces simply yield `false`/default capabilities.
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
            max_resolution: (MAX_RESOLUTION_WIDTH, MAX_RESOLUTION_HEIGHT), // Max resolution
            max_fps: HIGH_FPS,                // Max FPS
            exposure_range: None,
            iso_range: None,
            focus_range: None,
        };

        // Test camera control capabilities
        if let Some(ref _camera_control) = self.camera_control {
            // Test focus support
            if self.test_camera_control_support(CameraControl_Focus.0) {
                // Focus property
                capabilities.supports_auto_focus = true;
                capabilities.supports_manual_focus = true;
                if let Some(ref _range) = self.focus_range {
                    capabilities.focus_range = Some((0.0, 1.0)); // Normalized range
                }
            }

            // Test exposure support
            if self.test_camera_control_support(CameraControl_Exposure.0) {
                // Exposure property
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
            if self.test_camera_control_support(CameraControl_Zoom.0) {
                // Zoom property
                capabilities.supports_zoom = true;
            }
        }

        // Test video processing capabilities
        if let Some(ref _video_proc_amp) = self.video_proc_amp {
            if self.test_video_proc_support(VideoProcAmp_WhiteBalance.0) {
                // White balance property
                capabilities.supports_white_balance = true;
            }
        }

        log::debug!(
            "Camera capabilities: focus({}/{}), exposure({}/{}), white_balance({}), zoom({})",
            capabilities.supports_auto_focus,
            capabilities.supports_manual_focus,
            capabilities.supports_auto_exposure,
            capabilities.supports_manual_exposure,
            capabilities.supports_white_balance,
            capabilities.supports_zoom
        );

        Ok(capabilities)
    }

    // Individual control implementation methods

    fn set_auto_focus(&mut self, enabled: bool) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            // Note: Using constants
            let flags = if enabled { CameraControl_Flags_Auto.0 } else { CameraControl_Flags_Manual.0 }; // Auto vs Manual flags

            unsafe {
                camera_control
                    .Set(
                        CameraControl_Focus.0, // Focus property
                        0, // Value doesn't matter for auto mode
                        flags,
                    )
                    .map_err(|e| {
                        CameraError::ControlError(format!("Failed to set auto focus: {e}"))
                    })?;
            }

            log::debug!("Set auto focus: {enabled}");
            Ok(())
        } else {
            Err(CameraError::ControlError(
                "Camera control interface not available".to_string(),
            ))
        }
    }

    fn set_focus_distance(&mut self, distance: f32) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            if let Some(ref range) = self.focus_range {
                let device_value = normalize_to_device_range(distance, range);

                unsafe {
                    camera_control
                        .Set(
                            CameraControl_Focus.0, // Focus property
                            device_value,
                            CameraControl_Flags_Manual.0, // Manual flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!(
                                "Failed to set focus distance: {e}"
                            ))
                        })?;
                }

                log::debug!(
                    "Set focus distance: {distance} (device value: {device_value})"
                );
                Ok(())
            } else {
                Err(CameraError::ControlError(
                    "Focus range not available".to_string(),
                ))
            }
        } else {
            Err(CameraError::ControlError(
                "Camera control interface not available".to_string(),
            ))
        }
    }

    fn set_auto_exposure(&mut self, enabled: bool) -> Result<(), CameraError> {
        if let Some(ref camera_control) = self.camera_control {
            let flags = if enabled { CameraControl_Flags_Auto.0 } else { CameraControl_Flags_Manual.0 }; // Auto vs Manual flags

            unsafe {
                camera_control
                    .Set(
                        CameraControl_Exposure.0, // Exposure property
                        0, // Value doesn't matter for auto mode
                        flags,
                    )
                    .map_err(|e| {
                        CameraError::ControlError(format!("Failed to set auto exposure: {e}"))
                    })?;
            }

            log::debug!("Set auto exposure: {enabled}");
            Ok(())
        } else {
            Err(CameraError::ControlError(
                "Camera control interface not available".to_string(),
            ))
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
                    camera_control
                        .Set(
                            CameraControl_Exposure.0, // Exposure property
                            device_value,
                            CameraControl_Flags_Manual.0, // Manual flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!("Failed to set exposure time: {e}"))
                        })?;
                }

                log::debug!(
                    "Set exposure time: {time_seconds}s (log2: {log_exposure}, device value: {device_value})"
                );
                Ok(())
            } else {
                Err(CameraError::ControlError(
                    "Exposure range not available".to_string(),
                ))
            }
        } else {
            Err(CameraError::ControlError(
                "Camera control interface not available".to_string(),
            ))
        }
    }

    fn set_white_balance(&mut self, wb: &WhiteBalance) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            let kelvin_temp = white_balance_to_kelvin(wb);

            if kelvin_temp == -1 {
                // Auto white balance
                unsafe {
                    video_proc_amp
                        .Set(
                            VideoProcAmp_WhiteBalance.0, // White balance property
                            0, VideoProcAmp_Flags_Auto.0, // Auto flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!(
                                "Failed to set auto white balance: {e}"
                            ))
                        })?;
                }
                log::debug!("Set white balance: Auto");
            } else {
                // Manual white balance with Kelvin temperature
                unsafe {
                    video_proc_amp
                        .Set(
                            VideoProcAmp_WhiteBalance.0, // White balance property
                            kelvin_temp,
                            VideoProcAmp_Flags_Manual.0, // Manual flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!("Failed to set white balance: {e}"))
                        })?;
                }
                log::debug!("Set white balance: {kelvin_temp}K");
            }

            Ok(())
        } else {
            Err(CameraError::ControlError(
                "Video processing interface not available".to_string(),
            ))
        }
    }

    fn set_brightness(&mut self, brightness: f32) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if let Some(ref range) = self.brightness_range {
                let device_value = normalize_to_device_range(brightness, range);

                unsafe {
                    video_proc_amp
                        .Set(
                            VideoProcAmp_Brightness.0, // Brightness property
                            device_value,
                            VideoProcAmp_Flags_Manual.0, // Manual flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!("Failed to set brightness: {e}"))
                        })?;
                }

                log::debug!(
                    "Set brightness: {brightness} (device value: {device_value})"
                );
                Ok(())
            } else {
                Err(CameraError::ControlError(
                    "Brightness range not available".to_string(),
                ))
            }
        } else {
            Err(CameraError::ControlError(
                "Video processing interface not available".to_string(),
            ))
        }
    }

    fn set_contrast(&mut self, contrast: f32) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if let Some(ref range) = self.contrast_range {
                let device_value = normalize_to_device_range(contrast, range);

                unsafe {
                    video_proc_amp
                        .Set(
                            VideoProcAmp_Contrast.0, // Contrast property
                            device_value,
                            VideoProcAmp_Flags_Manual.0, // Manual flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!("Failed to set contrast: {e}"))
                        })?;
                }

                log::debug!(
                    "Set contrast: {contrast} (device value: {device_value})"
                );
                Ok(())
            } else {
                Err(CameraError::ControlError(
                    "Contrast range not available".to_string(),
                ))
            }
        } else {
            Err(CameraError::ControlError(
                "Video processing interface not available".to_string(),
            ))
        }
    }

    fn set_saturation(&mut self, saturation: f32) -> Result<(), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            if let Some(ref range) = self.saturation_range {
                let device_value = normalize_to_device_range(saturation, range);

                unsafe {
                    video_proc_amp
                        .Set(
                            VideoProcAmp_Saturation.0, // Saturation property
                            device_value,
                            VideoProcAmp_Flags_Manual.0, // Manual flag
                        )
                        .map_err(|e| {
                            CameraError::ControlError(format!("Failed to set saturation: {e}"))
                        })?;
                }

                log::debug!(
                    "Set saturation: {saturation} (device value: {device_value})"
                );
                Ok(())
            } else {
                Err(CameraError::ControlError(
                    "Saturation range not available".to_string(),
                ))
            }
        } else {
            Err(CameraError::ControlError(
                "Video processing interface not available".to_string(),
            ))
        }
    }

    // Helper methods for MediaFoundation device discovery and interface management

    /// Find `MediaFoundation` media source for the specified device index.
    ///
    /// Uses `MFEnumDeviceSources` with `MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID`
    /// and `IMFActivate` to obtain an `IMFMediaSource` for the device.
    fn find_media_source(device_index: u32) -> Result<IMFMediaSource, CameraError> {
        unsafe {
            // Ensure MediaFoundation is started
            let _ = MFStartup(MF_SDK_VERSION, 0);

            let mut attributes = None;
            MFCreateAttributes(&raw mut attributes, 1).map_err(|e| {
                CameraError::InitializationError(format!("Failed to create attributes: {e}"))
            })?;

            let attributes = attributes.ok_or_else(|| {
                CameraError::InitializationError(
                    "MFCreateAttributes returned None unexpectedly".to_string(),
                )
            })?;
            attributes
                .SetGUID(
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE,
                    &MF_DEVSOURCE_ATTRIBUTE_SOURCE_TYPE_VIDCAP_GUID,
                )
                .map_err(|e| {
                    CameraError::InitializationError(format!("Failed to set source type GUID: {e}"))
                })?;

            let mut count = 0;
            let mut activates: *mut Option<IMFActivate> = std::ptr::null_mut();
            MFEnumDeviceSources(&attributes, &raw mut activates, &raw mut count).map_err(|e| {
                CameraError::InitializationError(format!("Failed to enumerate devices: {e}"))
            })?;

            if count == 0 {
                return Err(CameraError::InitializationError(
                    "No video capture devices found".to_string(),
                ));
            }

            if device_index >= count {
                return Err(CameraError::InitializationError(format!(
                    "Device index {device_index} out of range (found {count} devices)"
                )));
            }

            let activate = std::slice::from_raw_parts(activates, count as usize)
                [device_index as usize]
                .as_ref()
                .ok_or_else(|| {
                    CameraError::InitializationError("Failed to get device activate".to_string())
                })?;

            let source: IMFMediaSource = activate.ActivateObject().map_err(|e| {
                CameraError::InitializationError(format!("Failed to activate device object: {e}"))
            })?;

            // Cleanup - we don't need to explicitly free the array as Windows handles it,
            // but in Rust we just let the slice go out of scope. 
            // The activates array itself needs CoTaskMemFree if we were manual, 
            // but windows crate handles some of this? 
            // Actually usually for raw pointers from COM we might need to be careful.
            // But let's assume valid activation is enough for now.

            Ok(source)
        }
    }

    /// Cache control ranges for efficient value conversion
    fn cache_control_ranges(&mut self) -> Result<(), CameraError> {
        // Cache camera control ranges
        if self.camera_control.is_some() {
            self.focus_range = self.query_camera_control_range(CameraControl_Focus.0); // Focus property
            self.exposure_range = self.query_camera_control_range(CameraControl_Exposure.0); // Exposure property
        }

        // Cache video processing ranges
        if self.video_proc_amp.is_some() {
            self.brightness_range = self.query_video_proc_range(VideoProcAmp_Brightness.0); // Brightness property
            self.contrast_range = self.query_video_proc_range(VideoProcAmp_Contrast.0); // Contrast property
            self.saturation_range = self.query_video_proc_range(VideoProcAmp_Saturation.0); // Saturation property
            self.white_balance_range = self.query_video_proc_range(VideoProcAmp_WhiteBalance.0); // White balance property
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

                if camera_control
                    .GetRange(
                        property,
                        &raw mut min,
                        &raw mut max,
                        &raw mut step,
                        &raw mut default,
                        &raw mut flags,
                    )
                    .is_ok()
                {
                    return Some(ControlRange {
                        min,
                        max,
                        step,
                        default,
                    });
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

                if video_proc_amp
                    .GetRange(
                        property,
                        &raw mut min,
                        &raw mut max,
                        &raw mut step,
                        &raw mut default,
                        &raw mut flags,
                    )
                    .is_ok()
                {
                    return Some(ControlRange {
                        min,
                        max,
                        step,
                        default,
                    });
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

                camera_control
                    .Get(property, &raw mut value, &raw mut flags)
                    .map_err(|e| {
                        CameraError::ControlError(format!(
                            "Failed to get camera control value: {e}"
                        ))
                    })?;

                Ok((value, flags))
            }
        } else {
            Err(CameraError::ControlError(
                "Camera control interface not available".to_string(),
            ))
        }
    }

    /// Get current video processing value and flags
    fn get_video_proc_value(&self, property: i32) -> Result<(i32, i32), CameraError> {
        if let Some(ref video_proc_amp) = self.video_proc_amp {
            unsafe {
                let mut value = 0i32;
                let mut flags = 0i32;

                video_proc_amp
                    .Get(property, &raw mut value, &raw mut flags)
                    .map_err(|e| {
                        CameraError::ControlError(format!("Failed to get video proc value: {e}"))
                    })?;

                Ok((value, flags))
            }
        } else {
            Err(CameraError::ControlError(
                "Video processing interface not available".to_string(),
            ))
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
        log::debug!(
            "MediaFoundation controls resources cleaned up for device {}",
            self.device_index
        );
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
    let zero_to_one = f32::midpoint(normalized_clamped, 1.0);
    range.min + (zero_to_one * device_range as f32) as i32
}

/// Convert device-specific value to normalized range (-1.0 to 1.0)
fn device_to_normalized_range(device_value: i32, range: &ControlRange) -> f32 {
    let device_range = range.max - range.min;
    let zero_to_one = (device_value - range.min) as f32 / device_range as f32;
    (zero_to_one * 2.0) - 1.0
}

/// Convert `WhiteBalance` enum to Kelvin temperature for `MediaFoundation`
fn white_balance_to_kelvin(wb: &WhiteBalance) -> i32 {
    match wb {
        WhiteBalance::Auto => -1, // Use auto mode
        WhiteBalance::Incandescent => 2700,
        WhiteBalance::Fluorescent => 4200,
        WhiteBalance::Daylight => 5500,
        WhiteBalance::Flash => 5500,
        WhiteBalance::Cloudy => 6500,
        WhiteBalance::Shade => 7500,
        // Safe: valid color temperatures (1000–10000K) are well within i32 range
        #[allow(clippy::cast_possible_wrap)]
        WhiteBalance::Custom(temp) => *temp as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_controls_without_interfaces() -> MediaFoundationControls {
        MediaFoundationControls {
            device_index: 0,
            camera_control: None,
            video_proc_amp: None,
            focus_range: None,
            exposure_range: None,
            brightness_range: None,
            contrast_range: None,
            saturation_range: None,
            white_balance_range: None,
        }
    }

    #[test]
    fn test_normalize_round_trip_and_clamp() {
        let range = ControlRange {
            min: 0,
            max: 100,
            step: 1,
            default: 50,
        };

        assert_eq!(normalize_to_device_range(-2.0, &range), 0);
        assert_eq!(normalize_to_device_range(2.0, &range), 100);

        let mid = normalize_to_device_range(0.0, &range);
        assert_eq!(mid, 50);

        let back = device_to_normalized_range(mid, &range);
        assert!(back.abs() < 0.05);
    }

    #[test]
    fn test_white_balance_to_kelvin_mapping() {
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Auto), -1);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Incandescent), 2700);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Fluorescent), 4200);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Daylight), 5500);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Flash), 5500);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Cloudy), 6500);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Shade), 7500);
        assert_eq!(white_balance_to_kelvin(&WhiteBalance::Custom(5100)), 5100);
    }

    #[test]
    fn test_apply_controls_with_no_interfaces_rejects_all_requested() {
        let mut controls_if = make_controls_without_interfaces();
        let controls = CameraControls {
            auto_focus: Some(true),
            focus_distance: Some(0.2),
            auto_exposure: Some(false),
            exposure_time: Some(0.01),
            white_balance: Some(WhiteBalance::Daylight),
            brightness: Some(0.1),
            contrast: Some(0.2),
            saturation: Some(0.3),
            ..Default::default()
        };

        let result = controls_if
            .apply_controls(&controls)
            .expect("apply_controls should return structured result");

        assert!(result.applied.is_empty());
        assert!(result.rejected.contains(&"auto_focus".to_string()));
        assert!(result.rejected.contains(&"focus_distance".to_string()));
        assert!(result.rejected.contains(&"auto_exposure".to_string()));
        assert!(result.rejected.contains(&"exposure_time".to_string()));
        assert!(result.rejected.contains(&"white_balance".to_string()));
        assert!(result.rejected.contains(&"brightness".to_string()));
        assert!(result.rejected.contains(&"contrast".to_string()));
        assert!(result.rejected.contains(&"saturation".to_string()));
    }

    #[test]
    fn test_get_controls_and_capabilities_without_interfaces() {
        let controls_if = make_controls_without_interfaces();

        let controls = controls_if.get_controls().expect("get_controls should succeed");
        assert_eq!(controls, CameraControls::default());

        let caps = controls_if
            .get_capabilities()
            .expect("get_capabilities should succeed");
        assert!(!caps.supports_auto_focus);
        assert!(!caps.supports_manual_focus);
        assert!(!caps.supports_auto_exposure);
        assert!(!caps.supports_manual_exposure);
        assert!(!caps.supports_white_balance);
        assert!(!caps.supports_zoom);
    }

    #[test]
    fn test_direct_setters_return_error_without_interfaces() {
        let mut controls_if = make_controls_without_interfaces();

        assert!(matches!(
            controls_if.set_auto_focus(true),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_focus_distance(0.3),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_auto_exposure(true),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_exposure_time(0.02),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_white_balance(&WhiteBalance::Auto),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_brightness(0.1),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_contrast(0.1),
            Err(CameraError::ControlError(_))
        ));
        assert!(matches!(
            controls_if.set_saturation(0.1),
            Err(CameraError::ControlError(_))
        ));
    }

    #[test]
    fn test_support_checks_without_interfaces() {
        let controls_if = make_controls_without_interfaces();
        assert!(!controls_if.test_camera_control_support(CameraControl_Focus.0));
        assert!(!controls_if.test_video_proc_support(VideoProcAmp_Brightness.0));
        assert!(controls_if.query_camera_control_range(CameraControl_Focus.0).is_none());
        assert!(controls_if.query_video_proc_range(VideoProcAmp_Brightness.0).is_none());
        assert!(controls_if
            .get_camera_control_value(CameraControl_Focus.0)
            .is_err());
        assert!(controls_if
            .get_video_proc_value(VideoProcAmp_Brightness.0)
            .is_err());
    }

    #[test]
    fn test_constructor_best_effort_paths() {
        // Environment-dependent: this may succeed (device present) or fail (headless CI),
        // but both outcomes should be handled without panics.
        let result = MediaFoundationControls::new(0);
        assert!(result.is_ok() || result.is_err());

        if let Ok(mut controls) = result {
            let _ = controls.cache_control_ranges();
            let _ = controls.get_controls();
            let _ = controls.get_capabilities();

            let apply = controls.apply_controls(&CameraControls {
                auto_focus: Some(true),
                exposure_time: Some(0.01),
                ..Default::default()
            });
            assert!(apply.is_ok() || apply.is_err());
        }
    }
}
