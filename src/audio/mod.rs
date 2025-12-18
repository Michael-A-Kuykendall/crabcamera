//! Audio capture and encoding module for CrabCamera
//!
//! This module provides audio recording capabilities using:
//! - cpal for cross-platform audio capture
//! - opus for audio encoding
//!
//! # Spell Reference
//! Implements: #AudioDeviceEnumerate, #AudioCapturePCM, #AudioEncodeOpus

mod device;
mod capture;
mod encoder;
mod clock;

pub use device::{AudioDevice, list_audio_devices, get_default_audio_device};
pub use capture::{AudioCapture, AudioFrame};
pub use encoder::{OpusEncoder, EncodedAudio};
pub use clock::PTSClock;
