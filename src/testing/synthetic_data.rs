//! Test data generated from real OBSBOT hardware
//!
//! This module provides synthetic test data based on actual captures from
//! OBSBOT Tiny 4K camera and its built-in microphone, enabling reliable
//! offline testing without requiring hardware.

use crate::types::CameraFrame;

#[cfg(feature = "audio")]
use crate::audio::AudioFrame;

/// Create a synthetic test frame matching OBSBOT Tiny 4K output characteristics
///
/// Based on real captures: 3840x2160 RGB24, ~25MB per frame
pub fn synthetic_video_frame(frame_number: u64, width: u32, height: u32) -> CameraFrame {
    // Generate a frame with varying content to test encoder
    let mut data = vec![0u8; (width * height * 3) as usize];

    // Create a gradient pattern that changes each frame (tests temporal encoding)
    let base = (frame_number % 256) as u8;
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 3) as usize;
            // RGB gradient that varies by position and frame
            data[idx] = base.wrapping_add((x % 256) as u8); // R
            data[idx + 1] = base.wrapping_add((y % 256) as u8); // G
            data[idx + 2] = base.wrapping_add(((x + y) % 256) as u8); // B
        }
    }

    CameraFrame::new(data, width, height, "synthetic_obsbot".to_string())
}

/// Create a synthetic audio frame matching OBSBOT microphone output
///
/// Based on real captures: 48kHz stereo (2 channels) interleaved f32
#[cfg(feature = "audio")]
pub fn synthetic_audio_frame(frame_number: u64, samples_per_frame: usize) -> AudioFrame {
    // Generate a sine wave at 440Hz (A4) with varying amplitude
    // This tests the encoder with real-looking audio data
    let sample_rate = 48000.0;
    let frequency = 440.0;
    let channels = 2;

    let mut samples = vec![0.0f32; samples_per_frame * channels];

    for i in 0..samples_per_frame {
        let t = (frame_number as f64 * samples_per_frame as f64 + i as f64) / sample_rate;
        let value = (2.0 * std::f64::consts::PI * frequency * t).sin() as f32 * 0.3;

        // Stereo: same value in both channels
        samples[i * channels] = value;
        samples[i * channels + 1] = value;
    }

    let timestamp = (frame_number as f64 * samples_per_frame as f64) / sample_rate;

    AudioFrame {
        samples,
        sample_rate: 48000,
        channels: 2,
        timestamp,
    }
}

/// Hardware characteristics learned from OBSBOT Tiny 4K
pub struct ObsbotCharacteristics {
    /// Native video resolution (camera returns this even when lower requested)
    pub native_resolution: (u32, u32),
    /// Audio sample rate (Hz)
    pub audio_sample_rate: u32,
    /// Audio channels (stereo)
    pub audio_channels: u16,
    /// Typical frame rate achievable at 4K
    pub frame_rate_4k: f32,
    /// Camera name as reported by system
    pub device_name: &'static str,
    /// Microphone name as reported by system
    pub mic_name: &'static str,
}

impl Default for ObsbotCharacteristics {
    fn default() -> Self {
        Self {
            native_resolution: (3840, 2160),
            audio_sample_rate: 48000,
            audio_channels: 2,
            frame_rate_4k: 1.0, // ~1fps capture rate observed in testing
            device_name: "OBSBOT Tiny 4K Camera",
            mic_name: "OBSBOT Tiny 4K Microphone (OBSBOT Tiny 4K Audio)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthetic_video_frame_correct_size() {
        let frame = synthetic_video_frame(0, 1920, 1080);
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);
        assert_eq!(frame.data.len(), 1920 * 1080 * 3);
    }

    #[test]
    fn test_synthetic_video_frames_differ() {
        let frame0 = synthetic_video_frame(0, 320, 240);
        let frame1 = synthetic_video_frame(1, 320, 240);
        // Frames should have different content
        assert_ne!(frame0.data[0], frame1.data[0]);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_synthetic_audio_frame_correct_format() {
        let frame = synthetic_audio_frame(0, 960); // 20ms @ 48kHz
        assert_eq!(frame.sample_rate, 48000);
        assert_eq!(frame.channels, 2);
        assert_eq!(frame.samples.len(), 960 * 2); // stereo
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_synthetic_audio_has_signal() {
        let frame = synthetic_audio_frame(0, 960);
        let max_level: f32 = frame.samples.iter().map(|s| s.abs()).fold(0.0, f32::max);
        // Should have non-zero signal (0.3 amplitude sine wave)
        assert!(
            max_level > 0.1,
            "Audio should have signal, got {}",
            max_level
        );
        assert!(max_level < 0.5, "Audio shouldn't clip, got {}", max_level);
    }

    #[cfg(feature = "audio")]
    #[test]
    fn test_synthetic_audio_timestamps_increase() {
        let frame0 = synthetic_audio_frame(0, 960);
        let frame1 = synthetic_audio_frame(1, 960);
        assert!(
            frame1.timestamp > frame0.timestamp,
            "Timestamps should increase: {} vs {}",
            frame0.timestamp,
            frame1.timestamp
        );
    }

    #[test]
    fn test_obsbot_characteristics() {
        let chars = ObsbotCharacteristics::default();
        assert_eq!(chars.native_resolution, (3840, 2160));
        assert_eq!(chars.audio_sample_rate, 48000);
        assert_eq!(chars.audio_channels, 2);
    }
}
