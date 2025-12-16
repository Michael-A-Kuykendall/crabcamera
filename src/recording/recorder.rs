//! Video recorder combining encoder and muxer

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use std::time::Instant;

use muxide::api::{MuxerBuilder, VideoCodec, Metadata};

use super::encoder::H264Encoder;
use super::config::{RecordingConfig, RecordingStats};
use crate::errors::CameraError;
use crate::types::CameraFrame;

/// Video recorder that captures frames, encodes to H.264, and muxes to MP4
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
}

impl Recorder {
    /// Create a new recorder that writes to the specified file
    pub fn new<P: AsRef<Path>>(output_path: P, config: RecordingConfig) -> Result<Self, CameraError> {
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

        if let Some(ref title) = config.title {
            let metadata = Metadata::new()
                .with_title(title)
                .with_current_time();
            builder = builder.with_metadata(metadata);
        } else {
            let metadata = Metadata::new().with_current_time();
            builder = builder.with_metadata(metadata);
        }

        let muxer = builder.build()
            .map_err(|e| CameraError::MuxingError(format!("Failed to create muxer: {}", e)))?;

        let frame_duration_secs = 1.0 / config.fps;

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
        })
    }

    /// Write a camera frame to the recording
    pub fn write_frame(&mut self, frame: &CameraFrame) -> Result<(), CameraError> {
        let now = Instant::now();
        
        // Initialize start time on first frame
        if self.start_time.is_none() {
            self.start_time = Some(now);
        }

        // Check if we should drop this frame (frame rate limiting)
        if let Some(last_time) = self.last_frame_time {
            let elapsed = now.duration_since(last_time).as_secs_f64();
            if elapsed < self.frame_duration_secs * 0.8 {
                // Frame came too fast, skip it
                self.dropped_frames += 1;
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

        // Calculate PTS based on frame count and target fps
        let pts = self.frame_count as f64 * self.frame_duration_secs;

        // Write to muxer (use the keyframe info from the encoder)
        self.muxer.write_video(pts, &encoded.data, encoded.is_keyframe)
            .map_err(|e| CameraError::MuxingError(format!("Failed to write frame: {}", e)))?;

        self.frame_count += 1;
        self.last_frame_time = Some(now);

        Ok(())
    }

    /// Write raw RGB data as a frame
    pub fn write_rgb_frame(&mut self, rgb_data: &[u8], width: u32, height: u32) -> Result<(), CameraError> {
        // Validate dimensions
        if width != self.config.width || height != self.config.height {
            return Err(CameraError::EncodingError(format!(
                "Frame dimensions {}x{} don't match recording config {}x{}",
                width, height, self.config.width, self.config.height
            )));
        }

        let now = Instant::now();
        
        if self.start_time.is_none() {
            self.start_time = Some(now);
        }

        // Encode the frame
        let encoded = self.encoder.encode_rgb(rgb_data)?;

        // Skip empty frames (encoder may return no data for some frames)
        if encoded.data.is_empty() {
            self.dropped_frames += 1;
            return Ok(());
        }

        let pts = self.frame_count as f64 * self.frame_duration_secs;

        self.muxer.write_video(pts, &encoded.data, encoded.is_keyframe)
            .map_err(|e| CameraError::MuxingError(format!("Failed to write frame: {}", e)))?;

        self.frame_count += 1;
        self.last_frame_time = Some(now);

        Ok(())
    }

    /// Finish the recording and return statistics
    pub fn finish(self) -> Result<RecordingStats, CameraError> {
        // Use finish_with_stats() which returns Result<MuxerStats, MuxerError>
        let muxer_stats = self.muxer.finish_with_stats()
            .map_err(|e| CameraError::MuxingError(format!("Failed to finalize recording: {}", e)))?;

        let actual_duration = self.start_time
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
        let config = RecordingConfig::new(640, 480, 30.0)
            .with_title("Test Recording");
        
        let mut recorder = Recorder::new(&output, config)
            .expect("Recorder creation failed");
        
        // Create test frames (gray gradient)
        for i in 0..30 {
            let gray = (i * 8) as u8;
            let rgb = vec![gray; 640 * 480 * 3];
            
            recorder.write_rgb_frame(&rgb, 640, 480)
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
