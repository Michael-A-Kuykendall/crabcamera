//! Video recorder combining encoder and muxer
//!
//! # Spell: RecorderIntegrateAudio
//! ^ Intent: integrate audio capture and encoding into the existing Recorder
//!           without destabilizing video recording
//!
//! @Recorder
//!   : config -> recorder
//!   ! supports_audio_optional
//!   ! configures_muxer_audio_track_when_enabled
//!   ! continues_video_if_audio_fails
//!   - writing_audio_without_track_declaration
//!   - blocking_video_on_audio

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use muxide::api::{Metadata, MuxerBuilder, VideoCodec};

#[cfg(feature = "audio")]
use muxide::api::AudioCodec;

use super::config::{RecordingConfig, RecordingStats};
use super::encoder::H264Encoder;
use crate::errors::CameraError;
use crate::types::CameraFrame;

#[cfg(feature = "audio")]
use crate::audio::{EncodedAudio, OpusEncoder, PTSClock};
#[cfg(feature = "audio")]
use std::thread::JoinHandle;

/// Video recorder that captures frames, encodes to H.264, and muxes to MP4
/// Per #RecorderIntegrateAudio: ! supports_audio_optional
pub struct Recorder {
    encoder: H264Encoder,
    muxer: muxide::api::Muxer<BufWriter<File>>,
    config: RecordingConfig,
    output_path: String,
    frame_count: u64,
    dropped_frames: u64,
    start_time: Option<Instant>,
    last_frame_time: Option<Instant>,
    frame_duration_secs: f64,
    /// Shared PTS clock for audio/video sync
    #[cfg(feature = "audio")]
    pts_clock: Option<PTSClock>,
    /// Channel to receive encoded audio from audio thread
    #[cfg(feature = "audio")]
    audio_receiver: Option<crossbeam_channel::Receiver<EncodedAudio>>,
    /// Audio thread handle
    #[cfg(feature = "audio")]
    audio_thread: Option<JoinHandle<()>>,
    /// Signal to stop audio thread
    #[cfg(feature = "audio")]
    audio_stop: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// Shared flag for audio thread to report errors
    /// Per #AudioErrorRecovery: ! error_logged, ! session_status_reflects_audio_state
    #[cfg(feature = "audio")]
    audio_error_flag: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
    /// Whether audio is enabled for this recording
    #[cfg(feature = "audio")]
    audio_enabled: bool,
    /// Audio error state (cached from shared flag)
    /// Per #AudioErrorRecovery: ! continues_video_if_audio_fails
    #[cfg(feature = "audio")]
    audio_failed: bool,
}

impl Recorder {
    /// Create a new recorder that writes to the specified file
    /// Per #RecorderIntegrateAudio: ! configures_muxer_audio_track_when_enabled
    pub fn new<P: AsRef<Path>>(
        output_path: P,
        config: RecordingConfig,
    ) -> Result<Self, CameraError> {
        let output_path_str = output_path.as_ref().to_string_lossy().to_string();

        // Create the output file
        let file = File::create(&output_path)
            .map_err(|e| CameraError::IoError(format!("Failed to create output file: {}", e)))?;
        let writer = BufWriter::new(file);

        // Create the H.264 encoder
        let encoder = H264Encoder::new(config.width, config.height, config.fps, config.bitrate)?;

        // Build the muxer with optional metadata
        let mut builder = MuxerBuilder::new(writer)
            .video(VideoCodec::H264, config.width, config.height, config.fps)
            .with_fast_start(config.fast_start);

        // Configure audio track if enabled
        // Per #RecorderIntegrateAudio: ! configures_muxer_audio_track_when_enabled
        #[cfg(feature = "audio")]
        let audio_config = config.audio.clone();
        #[cfg(feature = "audio")]
        if let Some(ref audio_cfg) = audio_config {
            builder = builder.audio(AudioCodec::Opus, audio_cfg.sample_rate, audio_cfg.channels);
        }

        if let Some(ref title) = config.title {
            let metadata = Metadata::new().with_title(title).with_current_time();
            builder = builder.with_metadata(metadata);
        } else {
            let metadata = Metadata::new().with_current_time();
            builder = builder.with_metadata(metadata);
        }

        let muxer = builder
            .build()
            .map_err(|e| CameraError::MuxingError(format!("Failed to create muxer: {}", e)))?;

        let frame_duration_secs = 1.0 / config.fps;

        // Audio subsystem is started lazily on first video frame
        // to ensure video starts first (muxide requirement)
        #[cfg(feature = "audio")]
        let pts_clock = audio_config.as_ref().map(|_| PTSClock::new());

        Ok(Self {
            encoder,
            muxer,
            config,
            output_path: output_path_str,
            frame_count: 0,
            dropped_frames: 0,
            start_time: None,
            last_frame_time: None,
            frame_duration_secs,
            #[cfg(feature = "audio")]
            pts_clock,
            #[cfg(feature = "audio")]
            audio_receiver: None,
            #[cfg(feature = "audio")]
            audio_thread: None,
            #[cfg(feature = "audio")]
            audio_stop: None,
            #[cfg(feature = "audio")]
            audio_error_flag: None,
            #[cfg(feature = "audio")]
            audio_enabled: audio_config.is_some(),
            #[cfg(feature = "audio")]
            audio_failed: false,
        })
    }

    /// Start audio capture thread (call after first video frame)
    /// Per #RecorderIntegrateAudio: ! continues_video_if_audio_fails
    /// Per #AudioErrorRecovery: ! error_logged, - panic, - silent_data_loss
    /// Audio runs in its own thread to avoid Send issues with cpal::Stream
    #[cfg(feature = "audio")]
    fn start_audio_capture(&mut self) {
        use crate::audio::AudioCapture;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        // Already started or not enabled
        if self.audio_thread.is_some() || !self.audio_enabled {
            return;
        }

        let Some(ref audio_cfg) = self.config.audio else {
            return;
        };

        let Some(ref clock) = self.pts_clock else {
            return;
        };

        // Channel for encoded audio packets
        let (sender, receiver) = crossbeam_channel::bounded::<EncodedAudio>(256);
        let stop_flag = Arc::new(AtomicBool::new(false));
        // Per #AudioErrorRecovery: ! session_status_reflects_audio_state
        let error_flag = Arc::new(AtomicBool::new(false));

        let device_id = audio_cfg.device_id.clone();
        let sample_rate = audio_cfg.sample_rate;
        let channels = audio_cfg.channels;
        let bitrate = audio_cfg.bitrate;
        let clock_clone = clock.clone();
        let stop_clone = stop_flag.clone();
        let error_clone = error_flag.clone();

        // Spawn audio thread
        // Per #AudioErrorRecovery: ! video_continues_on_audio_failure (thread errors don't affect video)
        let handle = std::thread::spawn(move || {
            // Helper to set error flag and log
            let report_error = |msg: &str| {
                log::error!("Audio thread error: {}", msg);
                error_clone.store(true, Ordering::SeqCst);
            };

            // Create capture and encoder in this thread (they stay here)
            let mut capture = match AudioCapture::new(device_id, sample_rate, channels, clock_clone)
            {
                Ok(c) => c,
                Err(e) => {
                    report_error(&format!("Audio capture init failed: {}", e));
                    return;
                }
            };

            let mut encoder = match OpusEncoder::new(sample_rate, channels, bitrate) {
                Ok(e) => e,
                Err(e) => {
                    report_error(&format!("Opus encoder init failed: {}", e));
                    return;
                }
            };

            if let Err(e) = capture.start() {
                report_error(&format!("Audio capture start failed: {}", e));
                return;
            }

            // Process audio until stop signal
            while !stop_clone.load(Ordering::Relaxed) {
                if let Some(frame) = capture.try_read() {
                    if let Ok(packets) = encoder.encode(&frame) {
                        for packet in packets {
                            if sender.try_send(packet).is_err() {
                                // Channel full, drop packet (not a fatal error)
                                log::debug!("Audio channel full, dropping packet");
                            }
                        }
                    }
                } else {
                    // No audio available, brief sleep to avoid busy-wait
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }

            // Flush remaining
            let _ = capture.stop();
            if let Ok(packets) = encoder.flush() {
                for packet in packets {
                    let _ = sender.try_send(packet);
                }
            }
        });

        self.audio_receiver = Some(receiver);
        self.audio_thread = Some(handle);
        self.audio_error_flag = Some(error_flag);
        self.audio_stop = Some(stop_flag);
    }

    /// Write a camera frame to the recording
    /// Per #RecorderIntegrateAudio: @WriteFrame
    ///   ! writes_video_pts_from_PTSClock
    ///   ! drains_audio_non_blocking
    ///   ! writes_audio_pts_from_audio_frames
    ///   - busy_wait
    ///   - unbounded_audio_drain
    pub fn write_frame(&mut self, frame: &CameraFrame) -> Result<(), CameraError> {
        let now = Instant::now();

        // Initialize start time on first frame and start audio
        let is_first_frame = self.start_time.is_none();
        if is_first_frame {
            self.start_time = Some(now);
            #[cfg(feature = "audio")]
            self.start_audio_capture();
        }

        // Check if we should drop this frame (frame rate limiting)
        // The 0.8 factor allows some jitter tolerance (frames up to 20% early are accepted)
        if let Some(last_time) = self.last_frame_time {
            let elapsed = now.duration_since(last_time).as_secs_f64();
            if elapsed < self.frame_duration_secs * 0.8 {
                // Frame came too fast, skip it
                self.dropped_frames += 1;
                if self.dropped_frames % 10 == 1 {
                    // Log every 10th dropped frame to avoid spam
                    log::debug!(
                        "Frame rate limiting: dropped {} frames (interval {:.1}ms < {:.1}ms threshold)",
                        self.dropped_frames,
                        elapsed * 1000.0,
                        self.frame_duration_secs * 0.8 * 1000.0
                    );
                }
                return Ok(());
            }
        }

        // Validate frame dimensions match config
        if frame.width != self.config.width || frame.height != self.config.height {
            return Err(CameraError::EncodingError(format!(
                "Frame dimensions {}x{} don't match recording config {}x{}",
                frame.width, frame.height, self.config.width, self.config.height
            )));
        }

        // Encode the frame to H.264
        let encoded = self.encoder.encode_rgb(&frame.data)?;

        // Skip empty frames (encoder may return no data for some frames)
        if encoded.data.is_empty() {
            self.dropped_frames += 1;
            return Ok(());
        }

        // Calculate PTS
        // Per #AVSyncPolicy: ! shared_baseline, - dual_clock_sources
        // When audio is enabled, use PTSClock for both A/V to ensure sync.
        // When video-only, use frame-count based PTS (no sync needed).
        #[cfg(feature = "audio")]
        let pts = if let Some(ref clock) = self.pts_clock {
            clock.pts() // Real elapsed time from shared clock
        } else {
            self.frame_count as f64 * self.frame_duration_secs
        };
        #[cfg(not(feature = "audio"))]
        let pts = self.frame_count as f64 * self.frame_duration_secs;

        // Write to muxer (use the keyframe info from the encoder)
        self.muxer
            .write_video(pts, &encoded.data, encoded.is_keyframe)
            .map_err(|e| CameraError::MuxingError(format!("Failed to write frame: {}", e)))?;

        self.frame_count += 1;
        self.last_frame_time = Some(now);

        // Drain and write audio (non-blocking)
        // Per spell: ! drains_audio_non_blocking, - busy_wait, - unbounded_audio_drain
        #[cfg(feature = "audio")]
        self.drain_audio();

        Ok(())
    }

    /// Drain available audio frames and write to muxer (non-blocking)
    /// Per #RecorderIntegrateAudio: ! drains_audio_non_blocking
    /// Bounded drain: processes at most MAX_AUDIO_DRAIN_PER_FRAME packets
    /// to prevent blocking video on slow audio processing
    #[cfg(feature = "audio")]
    fn drain_audio(&mut self) {
        const MAX_AUDIO_DRAIN_PER_FRAME: usize = 10;

        // Skip if audio failed or not enabled
        if self.audio_failed || !self.audio_enabled {
            return;
        }

        let Some(ref receiver) = self.audio_receiver else {
            return;
        };

        // Non-blocking drain with bounded iteration
        let mut drained = 0;
        while drained < MAX_AUDIO_DRAIN_PER_FRAME {
            match receiver.try_recv() {
                Ok(packet) => {
                    // Write to muxer with PTS from audio frame
                    // Per spell: ! writes_audio_pts_from_audio_frames
                    if let Err(e) = self.muxer.write_audio(packet.timestamp, &packet.data) {
                        log::warn!("Audio write failed (video continues): {}", e);
                        self.audio_failed = true;
                        return;
                    }
                    drained += 1;
                }
                Err(_) => break, // No more audio available (non-blocking)
            }
        }
    }

    /// Write raw RGB data as a frame
    pub fn write_rgb_frame(
        &mut self,
        rgb_data: &[u8],
        width: u32,
        height: u32,
    ) -> Result<(), CameraError> {
        // Validate dimensions
        if width != self.config.width || height != self.config.height {
            return Err(CameraError::EncodingError(format!(
                "Frame dimensions {}x{} don't match recording config {}x{}",
                width, height, self.config.width, self.config.height
            )));
        }

        let now = Instant::now();

        let is_first_frame = self.start_time.is_none();
        if is_first_frame {
            self.start_time = Some(now);
            #[cfg(feature = "audio")]
            self.start_audio_capture();
        }

        // Encode the frame
        let encoded = self.encoder.encode_rgb(rgb_data)?;

        // Skip empty frames (encoder may return no data for some frames)
        if encoded.data.is_empty() {
            self.dropped_frames += 1;
            return Ok(());
        }

        // Calculate PTS - same logic as write_frame
        // Per #AVSyncPolicy: ! shared_baseline
        #[cfg(feature = "audio")]
        let pts = if let Some(ref clock) = self.pts_clock {
            clock.pts()
        } else {
            self.frame_count as f64 * self.frame_duration_secs
        };
        #[cfg(not(feature = "audio"))]
        let pts = self.frame_count as f64 * self.frame_duration_secs;

        self.muxer
            .write_video(pts, &encoded.data, encoded.is_keyframe)
            .map_err(|e| CameraError::MuxingError(format!("Failed to write frame: {}", e)))?;

        self.frame_count += 1;
        self.last_frame_time = Some(now);

        // Drain and write audio (non-blocking)
        #[cfg(feature = "audio")]
        self.drain_audio();

        Ok(())
    }

    /// Finish the recording and return statistics
    pub fn finish(mut self) -> Result<RecordingStats, CameraError> {
        // Stop audio capture and flush remaining audio
        #[cfg(feature = "audio")]
        self.finish_audio();

        // Use finish_with_stats() which returns Result<MuxerStats, MuxerError>
        let muxer_stats = self.muxer.finish_with_stats().map_err(|e| {
            CameraError::MuxingError(format!("Failed to finalize recording: {}", e))
        })?;

        let actual_duration = self
            .start_time
            .map(|start| start.elapsed().as_secs_f64())
            .unwrap_or(muxer_stats.duration_secs);

        let actual_fps = if actual_duration > 0.0 {
            self.frame_count as f64 / actual_duration
        } else {
            0.0
        };

        Ok(RecordingStats {
            video_frames: muxer_stats.video_frames,
            audio_frames: muxer_stats.audio_frames,
            duration_secs: muxer_stats.duration_secs,
            bytes_written: muxer_stats.bytes_written,
            actual_fps,
            dropped_frames: self.dropped_frames,
            output_path: self.output_path,
        })
    }

    /// Stop audio capture thread and flush remaining audio
    #[cfg(feature = "audio")]
    fn finish_audio(&mut self) {
        use std::sync::atomic::Ordering;

        // Signal audio thread to stop
        if let Some(ref stop) = self.audio_stop {
            stop.store(true, Ordering::Relaxed);
        }

        // Wait for audio thread to finish (it will flush its encoder)
        if let Some(handle) = self.audio_thread.take() {
            let _ = handle.join();
        }

        // Drain any remaining packets from the channel
        if let Some(ref receiver) = self.audio_receiver {
            while let Ok(packet) = receiver.try_recv() {
                let _ = self.muxer.write_audio(packet.timestamp, &packet.data);
            }
        }
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the number of dropped frames
    pub fn dropped_frames(&self) -> u64 {
        self.dropped_frames
    }

    /// Get the recording duration so far
    pub fn duration(&self) -> f64 {
        self.start_time
            .map(|start| start.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    }

    /// Check if recording has started
    pub fn is_recording(&self) -> bool {
        self.start_time.is_some()
    }

    /// Force the next frame to be a keyframe
    pub fn force_keyframe(&mut self) {
        self.encoder.force_keyframe();
    }

    /// Check if audio capture has failed
    /// Per #AudioErrorRecovery: ! session_status_reflects_audio_state
    #[cfg(feature = "audio")]
    pub fn audio_failed(&self) -> bool {
        // Check the shared error flag from audio thread
        if let Some(ref flag) = self.audio_error_flag {
            use std::sync::atomic::Ordering;
            flag.load(Ordering::SeqCst)
        } else {
            // No flag = audio not started or not enabled
            self.audio_failed
        }
    }

    /// Check if audio is enabled for this recording
    #[cfg(feature = "audio")]
    pub fn audio_enabled(&self) -> bool {
        self.audio_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_recorder_creation() {
        let output = temp_dir().join("test_recording.mp4");
        let config = RecordingConfig::new(640, 480, 30.0);

        let result = Recorder::new(&output, config);
        assert!(result.is_ok(), "Recorder should be created successfully");

        // Clean up
        let _ = std::fs::remove_file(&output);
    }

    #[test]
    fn test_record_frames() {
        let output = temp_dir().join("test_frames_recording.mp4");
        let config = RecordingConfig::new(640, 480, 30.0).with_title("Test Recording");

        let mut recorder = Recorder::new(&output, config).expect("Recorder creation failed");

        // Create test frames (gray gradient)
        for i in 0..30 {
            let gray = (i * 8) as u8;
            let rgb = vec![gray; 640 * 480 * 3];

            recorder
                .write_rgb_frame(&rgb, 640, 480)
                .expect("Frame write should succeed");
        }

        let stats = recorder.finish().expect("Finish should succeed");

        assert_eq!(stats.video_frames, 30, "Should have 30 frames");
        assert!(stats.bytes_written > 0, "Should have written bytes");
        assert!(stats.duration_secs > 0.0, "Should have duration");

        // Verify file exists and has content
        let metadata = std::fs::metadata(&output).expect("File should exist");
        assert!(metadata.len() > 0, "File should have content");

        // Clean up
        let _ = std::fs::remove_file(&output);
    }
}
