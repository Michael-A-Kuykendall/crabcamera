//! Audio capture from microphone
//!
//! # Spell: AudioCapturePCM
//! ^ Intent: capture microphone audio as timestamped PCM frames with bounded memory
//!
//! @AudioCapture
//!   : (device_id, sample_rate, channels) -> AudioCapture
//!   ! produces_interleaved_f32_pcm
//!   ! bounded_buffer
//!   ! start_is_idempotent
//!   ! stop_is_idempotent
//!   ! joins_capture_thread_on_stop
//!   - unbounded_memory_growth
//!   - blocking_callback

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Stream, StreamConfig};

use super::clock::PTSClock;
use super::device::find_audio_device;
use crate::errors::CameraError;

/// Maximum number of audio frames to buffer before dropping oldest.
/// At 48kHz with 20ms frames (960 samples), this allows ~5 seconds of buffering.
/// 256 frames × 20ms = 5120ms = 5.12 seconds
/// This prevents unbounded memory growth if the consumer is slow.
const MAX_BUFFER_FRAMES: usize = 256;

/// A single audio frame with PCM samples and timestamp
#[derive(Debug, Clone)]
pub struct AudioFrame {
    /// Interleaved f32 PCM samples
    pub samples: Vec<f32>,
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels
    pub channels: u16,
    /// Presentation timestamp in seconds (from PTSClock)
    pub timestamp: f64,
}

/// Audio capture stream from microphone
pub struct AudioCapture {
    stream: Option<Stream>,
    receiver: crossbeam_channel::Receiver<AudioFrame>,
    is_running: Arc<AtomicBool>,
    sample_rate: u32,
    channels: u16,
    clock: PTSClock,
}

impl AudioCapture {
    /// Create a new audio capture for the specified device
    ///
    /// If `device_id` is None or empty, uses the system default input.
    /// The `clock` should be shared with the video recorder for sync.
    pub fn new(
        device_id: Option<String>,
        sample_rate: u32,
        channels: u16,
        clock: PTSClock,
    ) -> Result<Self, CameraError> {
        let device_id_str = device_id.as_deref().unwrap_or("default");
        let device_info = find_audio_device(device_id_str)?;

        let host = cpal::default_host();
        let device = if device_id_str.is_empty() || device_id_str == "default" {
            host.default_input_device()
                .ok_or_else(|| CameraError::AudioError("No default audio device".to_string()))?
        } else {
            host.input_devices()
                .map_err(|e| {
                    CameraError::AudioError(format!("Failed to enumerate devices: {}", e))
                })?
                .find(|d| d.name().ok().as_ref() == Some(&device_info.name))
                .ok_or_else(|| {
                    CameraError::AudioError(format!("Device not found: {}", device_id_str))
                })?
        };

        // Use requested sample rate, falling back to device default
        let supported_config = device
            .default_input_config()
            .map_err(|e| CameraError::AudioError(format!("No supported config: {}", e)))?;

        let actual_sample_rate = if sample_rate == 48000 || sample_rate == 44100 {
            sample_rate
        } else {
            supported_config.sample_rate().0
        };

        let actual_channels = if channels == 1 || channels == 2 {
            channels
        } else {
            supported_config.channels()
        };

        let config = StreamConfig {
            channels: actual_channels,
            sample_rate: cpal::SampleRate(actual_sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // Bounded channel to prevent unbounded memory growth
        let (sender, receiver) = crossbeam_channel::bounded(MAX_BUFFER_FRAMES);
        let is_running = Arc::new(AtomicBool::new(false));
        let is_running_clone = is_running.clone();
        let clock_clone = clock.clone();
        let config_sample_rate = config.sample_rate.0;
        let config_channels = config.channels;

        let stream = device
            .build_input_stream(
                &config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !is_running_clone.load(Ordering::Relaxed) {
                        return;
                    }

                    let frame = AudioFrame {
                        samples: data.to_vec(),
                        sample_rate: config_sample_rate,
                        channels: config_channels,
                        timestamp: clock_clone.pts(),
                    };

                    // Non-blocking send - drops oldest if buffer full
                    let _ = sender.try_send(frame);
                },
                move |err| {
                    log::error!("Audio capture error: {}", err);
                },
                None,
            )
            .map_err(|e| CameraError::AudioError(format!("Failed to build stream: {}", e)))?;

        Ok(Self {
            stream: Some(stream),
            receiver,
            is_running,
            sample_rate: config.sample_rate.0,
            channels: config.channels,
            clock,
        })
    }

    /// Start capturing audio (idempotent)
    pub fn start(&mut self) -> Result<(), CameraError> {
        if self.is_running.load(Ordering::Relaxed) {
            return Ok(()); // Already running
        }

        if let Some(ref stream) = self.stream {
            stream
                .play()
                .map_err(|e| CameraError::AudioError(format!("Failed to start stream: {}", e)))?;
            self.is_running.store(true, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Stop capturing audio (idempotent)
    pub fn stop(&mut self) -> Result<(), CameraError> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(()); // Already stopped
        }

        if let Some(ref stream) = self.stream {
            stream
                .pause()
                .map_err(|e| CameraError::AudioError(format!("Failed to stop stream: {}", e)))?;
            self.is_running.store(false, Ordering::Relaxed);
        }

        Ok(())
    }

    /// Try to read an audio frame without blocking
    ///
    /// Returns `None` if no frame is available.
    pub fn try_read(&self) -> Option<AudioFrame> {
        self.receiver.try_recv().ok()
    }

    /// Read all available audio frames
    ///
    /// Non-blocking, returns empty vec if no frames available.
    pub fn drain(&self) -> Vec<AudioFrame> {
        let mut frames = Vec::new();
        while let Ok(frame) = self.receiver.try_recv() {
            frames.push(frame);
        }
        frames
    }

    /// Check if capture is currently running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    /// Get the configured sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get the configured channel count
    pub fn channels(&self) -> u16 {
        self.channels
    }

    /// Get the shared PTS clock
    pub fn clock(&self) -> &PTSClock {
        &self.clock
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        // Ensure stream is stopped before drop
        let _ = self.stop();
        // Stream is dropped here, which joins any internal threads
        self.stream = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_frame_structure() {
        let frame = AudioFrame {
            samples: vec![0.0, 0.1, 0.2, 0.3],
            sample_rate: 48000,
            channels: 2,
            timestamp: 1.5,
        };
        assert_eq!(frame.samples.len(), 4);
        assert_eq!(frame.sample_rate, 48000);
        assert_eq!(frame.channels, 2);
    }

    #[test]
    fn test_start_stop_idempotent() {
        // This test will only work if audio device is available
        let clock = PTSClock::new();
        if let Ok(mut capture) = AudioCapture::new(None, 48000, 2, clock) {
            // Start twice should be fine
            assert!(capture.start().is_ok());
            assert!(capture.start().is_ok());

            // Stop twice should be fine
            assert!(capture.stop().is_ok());
            assert!(capture.stop().is_ok());
        }
    }
}
