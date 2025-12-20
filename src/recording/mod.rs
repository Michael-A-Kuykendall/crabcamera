//! Video recording module for CrabCamera
//!
//! This module provides video recording capabilities using:
//! - openh264 for H.264 encoding
//! - muxide for MP4 muxing
//!
//! # Example
//! ```rust,ignore
//! use crabcamera::recording::{Recorder, RecordingConfig};
//!
//! let config = RecordingConfig::new(1920, 1080, 30.0);
//! let mut recorder = Recorder::new("output.mp4", config)?;
//!
//! // In your frame capture loop:
//! recorder.write_frame(&frame)?;
//!
//! // When done:
//! let stats = recorder.finish()?;
//! ```

mod config;
mod encoder;
mod recorder;

#[cfg(feature = "audio")]
pub use config::AudioConfig;
pub use config::{RecordingConfig, RecordingQuality, RecordingStats};
pub use encoder::{EncodedFrame, H264Encoder};
pub use recorder::Recorder;

#[cfg(test)]
mod tests;
