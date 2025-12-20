//! Audio capture and encoding module for CrabCamera
//!
//! This module provides audio recording capabilities using:
//! - cpal for cross-platform audio capture
//! - opus for audio encoding
//!
//! Submodules:
//! - `device`: Audio device enumeration
//! - `capture`: PCM audio capture with bounded buffering
//! - `encoder`: Opus audio encoding
//! - `clock`: PTS (Presentation Timestamp) synchronization

mod capture;
mod clock;
mod device;
mod encoder;

pub use capture::{AudioCapture, AudioFrame};
pub use clock::PTSClock;
pub use device::{get_default_audio_device, list_audio_devices, AudioDevice};
pub use encoder::{EncodedAudio, OpusEncoder};
