//! H.264 encoder wrapper using openh264

use crate::errors::CameraError;
use openh264::encoder::{Encoder, FrameType};
use openh264::formats::YUVBuffer;

/// H.264 encoder using openh264
pub struct H264Encoder {
    encoder: Encoder,
    width: u32,
    height: u32,
    frame_count: u64,
    last_frame_was_keyframe: bool,
}

impl H264Encoder {
    /// Create a new H.264 encoder with the specified parameters
    ///
    /// Note: openh264 0.6.x API determines dimensions from the `YUVSource` at encode time.
    /// The fps and bitrate are hints for the encoder's rate control.
    ///
    /// # Errors
    /// Returns `CameraError` if the openh264 encoder fails to initialize.
    pub fn new(width: u32, height: u32, _fps: f64, _bitrate: u32) -> Result<Self, CameraError> {
        // openh264 0.6.x: Encoder::new() creates with default config
        // Dimensions are inferred from the YUVSource at encode time
        let encoder = Encoder::new()
            .map_err(|e| CameraError::EncodingError(format!("Failed to create encoder: {e}")))?;

        Ok(Self {
            encoder,
            width,
            height,
            frame_count: 0,
            last_frame_was_keyframe: false,
        })
    }

    /// Encode an RGB frame to H.264
    /// Returns the encoded NAL units as a single buffer (Annex B format)
    ///
    /// # Errors
    /// Returns `CameraError` if the frame size is invalid or encoding fails.
    pub fn encode_rgb(&mut self, rgb_data: &[u8]) -> Result<EncodedFrame, CameraError> {
        // Validate input size
        let expected_size = (self.width * self.height * 3) as usize;
        if rgb_data.len() != expected_size {
            return Err(CameraError::EncodingError(format!(
                "Invalid frame size: expected {expected_size} bytes, got {}",
                rgb_data.len()
            )));
        }

        // Convert RGB to YUV420
        let yuv = rgb_to_yuv420(rgb_data, self.width, self.height);

        // Encode the frame
        self.encode_yuv(&yuv)
    }

    /// Encode a YUV420 frame to H.264
    ///
    /// # Errors
    /// Returns `CameraError` if the underlying encoder fails.
    pub fn encode_yuv(&mut self, yuv_data: &[u8]) -> Result<EncodedFrame, CameraError> {
        // openh264 0.6.x: YUVBuffer::from_vec(data, width, height)
        let yuv_buffer =
            YUVBuffer::from_vec(yuv_data.to_vec(), self.width as usize, self.height as usize);

        let bitstream = self
            .encoder
            .encode(&yuv_buffer)
            .map_err(|e| CameraError::EncodingError(format!("Encoding failed: {e}")))?;

        self.frame_count += 1;

        // Check if this frame is a keyframe (IDR or I)
        let is_keyframe = matches!(bitstream.frame_type(), FrameType::IDR | FrameType::I);
        self.last_frame_was_keyframe = is_keyframe;

        // Convert bitstream to Vec using to_vec() method
        let data = bitstream.to_vec();

        Ok(EncodedFrame { data, is_keyframe })
    }

    /// Get the number of frames encoded
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Check if the last encoded frame was a keyframe (IDR)
    pub fn last_was_keyframe(&self) -> bool {
        self.last_frame_was_keyframe
    }

    /// Force the next frame to be a keyframe
    pub fn force_keyframe(&mut self) {
        // openh264 0.6.x: force_intra_frame() takes no arguments
        self.encoder.force_intra_frame();
    }
}

/// Result of encoding a single frame
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    /// Encoded H.264 data in Annex B format (with start codes)
    pub data: Vec<u8>,
    /// Whether this frame is a keyframe (IDR/I frame)
    pub is_keyframe: bool,
}

/// Convert RGB24 to YUV420 planar format
fn rgb_to_yuv420(rgb: &[u8], width: u32, height: u32) -> Vec<u8> {
    let width_usize = width as usize;
    let height_usize = height as usize;

    // YUV420: Y plane (w*h) + U plane (w/2 * h/2) + V plane (w/2 * h/2)
    let y_size = width_usize * height_usize;
    let uv_size = (width_usize / 2) * (height_usize / 2);
    let mut yuv = vec![0u8; y_size + uv_size * 2];

    let (y_plane, uv_planes) = yuv.split_at_mut(y_size);
    let (u_plane, v_plane) = uv_planes.split_at_mut(uv_size);

    // Convert each pixel
    for y in 0..height_usize {
        for x in 0..width_usize {
            let rgb_idx = (y * width_usize + x) * 3;
            let red = i32::from(rgb[rgb_idx]);
            let green = i32::from(rgb[rgb_idx + 1]);
            let blue = i32::from(rgb[rgb_idx + 2]);

            // BT.601 conversion
            let y_val = ((66 * red + 129 * green + 25 * blue + 128) >> 8) + 16;
            // i32→u8: clamped to 0-255 before cast, always in range
            y_plane[y * width_usize + x] = u8::try_from(y_val.clamp(0, 255)).unwrap_or(u8::MAX);

            // Subsample U and V (2x2 blocks)
            if y % 2 == 0 && x % 2 == 0 {
                let uv_idx = (y / 2) * (width_usize / 2) + (x / 2);
                let u_val = ((-38 * red - 74 * green + 112 * blue + 128) >> 8) + 128;
                let v_val = ((112 * red - 94 * green - 18 * blue + 128) >> 8) + 128;
                // i32→u8: clamped to 0-255 before cast, always in range
                u_plane[uv_idx] = u8::try_from(u_val.clamp(0, 255)).unwrap_or(u8::MAX);
                v_plane[uv_idx] = u8::try_from(v_val.clamp(0, 255)).unwrap_or(u8::MAX);
            }
        }
    }

    yuv
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_to_yuv420_size() {
        let width = 640u32;
        let height = 480u32;
        let rgb = vec![128u8; (width * height * 3) as usize];

        let yuv = rgb_to_yuv420(&rgb, width, height);

        // YUV420: Y + U + V = w*h + w*h/4 + w*h/4 = w*h * 1.5
        let expected = (width * height * 3 / 2) as usize;
        assert_eq!(yuv.len(), expected);
    }

    #[test]
    fn test_encoder_creation() {
        let result = H264Encoder::new(640, 480, 30.0, 1_000_000);
        assert!(result.is_ok(), "Encoder should be created successfully");
    }

    #[test]
    fn test_encode_frame() {
        let mut encoder =
            H264Encoder::new(640, 480, 30.0, 1_000_000).expect("Encoder creation failed");

        // Create a test frame (gray)
        let rgb = vec![128u8; 640 * 480 * 3];

        let result = encoder.encode_rgb(&rgb);
        assert!(result.is_ok(), "Encoding should succeed");

        let encoded_frame = result.expect("encoding should succeed");
        assert!(
            !encoded_frame.data.is_empty(),
            "Encoded data should not be empty"
        );

        // First bytes should be start code (0x00 0x00 0x00 0x01 or 0x00 0x00 0x01)
        assert!(
            encoded_frame.data.starts_with(&[0x00, 0x00, 0x00, 0x01])
                || encoded_frame.data.starts_with(&[0x00, 0x00, 0x01]),
            "Should start with Annex B start code"
        );

        // First frame should be a keyframe
        assert!(
            encoded_frame.is_keyframe,
            "First frame should be a keyframe"
        );
    }
}
