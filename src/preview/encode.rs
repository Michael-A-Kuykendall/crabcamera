use crate::types::CameraFrame;

/// Encode a `CameraFrame` to JPEG in-memory.
/// Returns `Vec<u8>` — caller wraps in `bytes::Bytes` for sharing.
pub fn encode_frame_jpeg(frame: &CameraFrame, quality: u8) -> Result<Vec<u8>, String> {
    let img = image::RgbImage::from_vec(frame.width, frame.height, frame.data.clone())
        .ok_or_else(|| "Failed to create image from frame data".to_string())?;

    let mut buf = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, quality);
    img.write_with_encoder(encoder)
        .map_err(|e| format!("JPEG encode failed: {e}"))?;

    Ok(buf)
}

/// Downscale a `CameraFrame` for preview using bilinear filtering.
/// Returns a new `CameraFrame` at reduced resolution.
pub fn downsample_frame(frame: &CameraFrame, scale: f32) -> CameraFrame {
    let new_w = (frame.width as f32 * scale) as u32;
    let new_h = (frame.height as f32 * scale) as u32;
    let img = image::RgbImage::from_vec(frame.width, frame.height, frame.data.clone())
        .expect("valid frame data");
    let resized = image::imageops::resize(&img, new_w, new_h, image::imageops::FilterType::Triangle);
    CameraFrame::new(resized.into_raw(), new_w, new_h, frame.device_id.clone())
}