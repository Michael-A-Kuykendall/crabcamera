//! Opus audio encoder
//!
//! Encodes PCM audio into Opus packets suitable for MP4 muxing
//! with proper frame buffering and flush semantics.
//!
//! ## Properties
//!
//! - Accepts interleaved f32 PCM samples
//! - Outputs valid Opus packets (RFC 6716)
//! - Flush operation emits remaining packets
//! - Operates at 48kHz (Opus standard)
//! - No hidden resampling
//! - Maintains channel count

use super::capture::AudioFrame;
use crate::errors::CameraError;
use crate::constants::{OPUS_SAMPLE_RATE, OPUS_APPLICATION_AUDIO, OPUS_FRAME_SAMPLES};

/// Encoded Opus audio packet
#[derive(Debug, Clone)]
pub struct EncodedAudio {
    /// Raw Opus packet data
    pub data: Vec<u8>,
    /// Presentation timestamp in seconds
    pub timestamp: f64,
    /// Duration of this packet in seconds
    pub duration: f64,
}

/// Opus encoder for PCM to Opus conversion
///
/// # Thread Safety
/// This type implements `Send` to allow moving the encoder to a dedicated audio thread.
/// The underlying `libopus` encoder is NOT thread-safe for concurrent access, but IS safe
/// to use from a single thread after being moved there.
///
/// **Invariant:** Once created, an `OpusEncoder` must only be accessed from one thread
/// at a time. The current architecture enforces this by:
/// 1. Creating the encoder in `start_audio_capture()`
/// 2. Moving it into a dedicated audio thread via `std::thread::spawn(move || ...)`
/// 3. The encoder never escapes that thread until dropped
///
/// Do NOT implement `Clone` or `Sync` for this type.
pub struct OpusEncoder {
    encoder: *mut libopus_sys::OpusEncoder,
    channels: u16,
    sample_rate: u32,
    /// Buffer for accumulating samples until we have a full frame
    sample_buffer: Vec<f32>,
    /// Timestamp of the first sample in the buffer (set once, never updated)
    buffer_start_pts: Option<f64>,
    /// Total samples encoded (for PTS calculation)
    samples_encoded: u64,
}

// SAFETY: OpusEncoder can be sent to another thread because:
// 1. The raw pointer `encoder` points to memory allocated by libopus
// 2. libopus encoders are safe to use from any single thread
// 3. We do NOT implement Sync, preventing concurrent access
// 4. The ownership model ensures only one thread accesses the encoder at a time
unsafe impl Send for OpusEncoder {}

impl OpusEncoder {
    /// Create a new Opus encoder
    ///
    /// # Arguments
    /// * `sample_rate` - Must be 48000 (Opus requirement)
    /// * `channels` - 1 for mono, 2 for stereo
    /// * `bitrate` - Target bitrate in bits per second (e.g., 128000)
    pub fn new(sample_rate: u32, channels: u16, bitrate: u32) -> Result<Self, CameraError> {
        if sample_rate != OPUS_SAMPLE_RATE {
            return Err(CameraError::AudioError(
                format!("Opus requires {OPUS_SAMPLE_RATE} Hz sample rate"),
            ));
        }

        if channels != 1 && channels != 2 {
            return Err(CameraError::AudioError(
                "Opus supports only mono (1) or stereo (2) channels".to_string(),
            ));
        }

        let mut error: i32 = 0;
        let encoder = unsafe {
            libopus_sys::opus_encoder_create(
                sample_rate as i32,
                i32::from(channels),
                OPUS_APPLICATION_AUDIO,
                &raw mut error,
            )
        };

        if encoder.is_null() || error != 0 {
            return Err(CameraError::AudioError(format!(
                "Failed to create Opus encoder: error code {error}"
            )));
        }

        // Set bitrate
        let result = unsafe {
            #[allow(clippy::cast_possible_wrap)] // Safe: constant is valid i32
            libopus_sys::opus_encoder_ctl(
                encoder,
                libopus_sys::OPUS_SET_BITRATE_REQUEST as i32,
                bitrate as i32,
            )
        };

        if result != 0 {
            unsafe { libopus_sys::opus_encoder_destroy(encoder) };
            return Err(CameraError::AudioError(format!(
                "Failed to set bitrate: error code {result}"
            )));
        }

        Ok(Self {
            encoder,
            channels,
            sample_rate,
            sample_buffer: Vec::with_capacity(OPUS_FRAME_SAMPLES * channels as usize * 2),
            buffer_start_pts: None,
            samples_encoded: 0,
        })
    }

    /// Encode an audio frame.
    ///
    /// May return empty vec if not enough samples accumulated for a full Opus frame.
    /// May return multiple packets if input contains multiple frames worth of samples.
    ///
    /// # Errors
    ///
    /// * `CameraError::AudioError`: If sample rate/channels don't match or encoding fails.
    pub fn encode(&mut self, frame: &AudioFrame) -> Result<Vec<EncodedAudio>, CameraError> {
        // Validate input
        if frame.sample_rate != self.sample_rate {
            return Err(CameraError::AudioError(format!(
                "Sample rate mismatch: expected {}, got {}",
                self.sample_rate, frame.sample_rate
            )));
        }

        if frame.channels != self.channels {
            return Err(CameraError::AudioError(format!(
                "Channel count mismatch: expected {}, got {}",
                self.channels, frame.channels
            )));
        }

        // Track PTS of first sample in buffer
        if self.buffer_start_pts.is_none() && !frame.samples.is_empty() {
            self.buffer_start_pts = Some(frame.timestamp);
        }

        // Add samples to buffer
        self.sample_buffer.extend_from_slice(&frame.samples);

        // Encode complete frames
        let mut encoded_packets = Vec::new();
        let samples_per_frame = OPUS_FRAME_SAMPLES * self.channels as usize;

        // Use f64::from for safe lossless casting where possible
        let sample_rate_f64 = f64::from(self.sample_rate);
        #[allow(clippy::cast_precision_loss)] // u64 -> f64 is lossy but acceptable for timestamps here
        let opus_samples_f64 = OPUS_FRAME_SAMPLES as f64;
        let frame_duration = opus_samples_f64 / sample_rate_f64;

        while self.sample_buffer.len() >= samples_per_frame {
            let frame_samples: Vec<f32> = self.sample_buffer.drain(..samples_per_frame).collect();

            // Calculate PTS for this frame
            #[allow(clippy::cast_precision_loss)] // u64 -> f64 is lossy but acceptable for timestamps here
            let pts = self.samples_encoded as f64 / sample_rate_f64;

            // Encode to Opus
            let mut output = vec![0u8; 4000]; // Max Opus packet size
            let len = unsafe {
                #[allow(clippy::cast_possible_wrap)] // Safe for small Opus frame sizes
                libopus_sys::opus_encode_float(
                    self.encoder,
                    frame_samples.as_ptr(),
                    OPUS_FRAME_SAMPLES as i32,
                    output.as_mut_ptr(),
                    output.len() as i32,
                )
            };

            if len < 0 {
                return Err(CameraError::AudioError(format!(
                    "Opus encoding failed: error code {len}"
                )));
            }

            output.truncate(len as usize);

            encoded_packets.push(EncodedAudio {
                data: output,
                timestamp: self.buffer_start_pts.unwrap_or(0.0) + pts,
                duration: frame_duration,
            });

            self.samples_encoded += OPUS_FRAME_SAMPLES as u64;
        }

        // NOTE: Do NOT update buffer_start_pts here. The samples_encoded counter
        // already tracks absolute position from recording start. Updating
        // buffer_start_pts would cause double-counting of timestamps.

        Ok(encoded_packets)
    }

    /// Flush remaining samples.
    ///
    /// Call this when recording ends to encode any remaining buffered samples.
    ///
    /// # Errors
    ///
    /// * `CameraError::AudioError`: If encoding fails.
    pub fn flush(&mut self) -> Result<Vec<EncodedAudio>, CameraError> {
        if self.sample_buffer.is_empty() {
            return Ok(Vec::new());
        }

        // Pad to full frame size
        let samples_per_frame = OPUS_FRAME_SAMPLES * self.channels as usize;
        let padding_needed = samples_per_frame - (self.sample_buffer.len() % samples_per_frame);
        if padding_needed < samples_per_frame {
            self.sample_buffer.extend(vec![0.0f32; padding_needed]);
        }

        // Encode remaining
        let mut encoded_packets = Vec::new();
        // Use f64::from for safe lossless casting where possible
        let sample_rate_f64 = f64::from(self.sample_rate);
        #[allow(clippy::cast_precision_loss)] // Safe for small Opus frame sizes
        let opus_samples_f64 = OPUS_FRAME_SAMPLES as f64;
        let frame_duration = opus_samples_f64 / sample_rate_f64;

        while self.sample_buffer.len() >= samples_per_frame {
            let frame_samples: Vec<f32> = self.sample_buffer.drain(..samples_per_frame).collect();
            #[allow(clippy::cast_precision_loss)] // u64 -> f64 is lossy but acceptable for timestamps here
            let pts = self.samples_encoded as f64 / sample_rate_f64;

            let mut output = vec![0u8; 4000];
            let len = unsafe {
                #[allow(clippy::cast_possible_wrap)] // Safe for small Opus frame sizes
                libopus_sys::opus_encode_float(
                    self.encoder,
                    frame_samples.as_ptr(),
                    OPUS_FRAME_SAMPLES as i32,
                    output.as_mut_ptr(),
                    output.len() as i32,
                )
            };

            if len < 0 {
                return Err(CameraError::AudioError(format!(
                    "Opus flush failed: error code {len}"
                )));
            }

            output.truncate(len as usize);

            encoded_packets.push(EncodedAudio {
                data: output,
                timestamp: self.buffer_start_pts.unwrap_or(0.0) + pts,
                duration: frame_duration,
            });

            self.samples_encoded += OPUS_FRAME_SAMPLES as u64;
        }

        Ok(encoded_packets)
    }

    /// Get the configured sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the configured channel count
    pub fn channels(&self) -> u16 {
        self.channels
    }
}

impl Drop for OpusEncoder {
    fn drop(&mut self) {
        if !self.encoder.is_null() {
            unsafe {
                libopus_sys::opus_encoder_destroy(self.encoder);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_creation() {
        let encoder = OpusEncoder::new(48000, 2, 128_000);
        assert!(encoder.is_ok());
    }

    #[test]
    fn test_encoder_rejects_wrong_sample_rate() {
        let encoder = OpusEncoder::new(44100, 2, 128_000);
        assert!(encoder.is_err());
    }

    #[test]
    fn test_encoder_rejects_wrong_channels() {
        let encoder = OpusEncoder::new(48000, 5, 128_000);
        assert!(encoder.is_err());
    }

    #[test]
    fn test_encode_full_frame() {
        let mut encoder = OpusEncoder::new(48000, 2, 128_000).unwrap();

        // Create a full frame worth of stereo samples (960 samples * 2 channels)
        let frame = AudioFrame {
            samples: vec![0.0f32; OPUS_FRAME_SAMPLES * 2],
            sample_rate: 48000,
            channels: 2,
            timestamp: 0.0,
        };

        let encoded_packets = encoder.encode(&frame).unwrap();
        assert_eq!(encoded_packets.len(), 1);
        assert!(!encoded_packets[0].data.is_empty());
    }

    #[test]
    fn test_encode_partial_frame() {
        let mut encoder = OpusEncoder::new(48000, 2, 128_000).unwrap();

        // Less than a full frame
        let frame = AudioFrame {
            samples: vec![0.0f32; 100],
            sample_rate: 48000,
            channels: 2,
            timestamp: 0.0,
        };

        let encoded_packets = encoder.encode(&frame).unwrap();
        assert!(
            encoded_packets.is_empty(),
            "Partial frame should not produce output"
        );
    }

    #[test]
    fn test_flush_remaining() {
        let mut encoder = OpusEncoder::new(48000, 2, 128_000).unwrap();

        // Add partial frame
        let frame = AudioFrame {
            samples: vec![0.0f32; 100],
            sample_rate: 48000,
            channels: 2,
            timestamp: 0.0,
        };
        encoder.encode(&frame).unwrap();

        // Flush should produce output
        let flushed = encoder.flush().unwrap();
        assert_eq!(flushed.len(), 1);
    }
}
