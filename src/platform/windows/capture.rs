use crate::errors::CameraError;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame};
use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{RequestedFormat, RequestedFormatType},
    Camera,
};

/// List available cameras on Windows  
pub fn list_cameras() -> Result<Vec<CameraDeviceInfo>, CameraError> {
    let mut all_cameras = Vec::new();

    // Try multiple backends to detect all camera types including OBS Virtual Camera
    let backends = vec![
        nokhwa::utils::ApiBackend::MediaFoundation,
        // DirectShow not available in current nokhwa version
        nokhwa::utils::ApiBackend::Auto,
    ];

    for backend in backends {
        match query(backend) {
            Ok(cameras) => {
                log::debug!(
                    "Found {} cameras using {:?} backend",
                    cameras.len(),
                    backend
                );

                // Filter duplicates based on camera name to avoid double-listing
                for camera_info in cameras {
                    let name = camera_info.human_name();

                    // Check if we already have this camera (avoid duplicates across backends)
                    if !all_cameras
                        .iter()
                        .any(|existing: &nokhwa::utils::CameraInfo| existing.human_name() == name)
                    {
                        all_cameras.push(camera_info);
                    }
                }
            }
            Err(e) => {
                log::debug!("Backend {:?} failed: {}", backend, e);
                // Continue trying other backends
            }
        }
    }

    if all_cameras.is_empty() {
        return Err(CameraError::InitializationError(
            "No cameras found on any backend".to_string(),
        ));
    }

    let mut device_list = Vec::new();
    for camera_info in all_cameras {
        let mut device =
            CameraDeviceInfo::new(camera_info.index().to_string(), camera_info.human_name());

        device = device.with_description(camera_info.description().to_string());

        // Add common Windows camera formats
        let formats = vec![
            CameraFormat::new(1920, 1080, 30.0),
            CameraFormat::new(1280, 720, 30.0),
            CameraFormat::new(640, 480, 30.0),
        ];
        device = device.with_formats(formats);

        device_list.push(device);
    }

    Ok(device_list)
}

/// Initialize camera on Windows with MediaFoundation backend
///
/// # Arguments
/// * `device_id` - The camera device index as a string
/// * `format` - Requested camera format (currently ignored - nokhwa uses highest resolution)
///
/// # Note
/// The `format` parameter is currently not applied because nokhwa's MediaFoundation
/// backend works best with AbsoluteHighestResolution mode. Format negotiation happens
/// at the frame capture level via MJPEG decoding.
pub fn initialize_camera(device_id: &str, format: CameraFormat) -> Result<Camera, CameraError> {
    log::debug!(
        "Requested format: {}x{} @ {}fps (note: nokhwa will use highest resolution)",
        format.width,
        format.height,
        format.fps
    );

    let device_index = device_id
        .parse::<u32>()
        .map_err(|_| CameraError::InitializationError("Invalid device ID".to_string()))?;

    let requested_format =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

    let camera = Camera::new(
        nokhwa::utils::CameraIndex::Index(device_index),
        requested_format,
    )
    .map_err(|e| CameraError::InitializationError(format!("Failed to initialize camera: {}", e)))?;

    Ok(camera)
}

/// Capture frame from Windows camera
/// Note: nokhwa returns MJPEG data even when RgbFormat is requested,
/// so we need to decode it manually to RGB
pub fn capture_frame(camera: &mut Camera, device_id: &str) -> Result<CameraFrame, CameraError> {
    let frame = camera
        .frame()
        .map_err(|e| CameraError::CaptureError(format!("Failed to capture frame: {}", e)))?;

    let raw_bytes = frame.buffer_bytes();
    let width = frame.resolution().width_x;
    let height = frame.resolution().height_y;

    log::debug!(
        "Raw frame: {}x{}, {} bytes, first 3 bytes: {:?}",
        width,
        height,
        raw_bytes.len(),
        raw_bytes.get(0..3).unwrap_or(&[])
    );

    // Check if the data is MJPEG (starts with FFD8FF) and needs decoding
    let rgb_data = if raw_bytes.len() >= 3
        && raw_bytes[0] == 0xFF
        && raw_bytes[1] == 0xD8
        && raw_bytes[2] == 0xFF
    {
        // Data is MJPEG - decode to RGB
        log::debug!("Decoding MJPEG frame ({} bytes) to RGB", raw_bytes.len());

        let img = image::load_from_memory(&raw_bytes)
            .map_err(|e| CameraError::CaptureError(format!("Failed to decode MJPEG: {}", e)))?;

        img.to_rgb8().into_raw()
    } else {
        // Data is already RGB (or at least not MJPEG)
        // Check if it's mostly zeros (invalid frame)
        let non_zero_count = raw_bytes.iter().filter(|&&b| b != 0).count();
        let total = raw_bytes.len();
        let pct_nonzero = (non_zero_count as f64 / total as f64) * 100.0;
        log::debug!("RGB frame: {:.1}% non-zero pixels", pct_nonzero);

        if pct_nonzero < 1.0 {
            log::warn!("Frame appears to be mostly zeros - camera may not be ready");
        }

        raw_bytes.to_vec()
    };

    let camera_frame = CameraFrame::new(rgb_data, width, height, device_id.to_string());

    Ok(camera_frame.with_format(frame.source_frame_format().to_string()))
}
