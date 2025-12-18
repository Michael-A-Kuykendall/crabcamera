//! Testing utilities for CrabCamera
//!
//! Provides synthetic test data based on real hardware captures
//! from OBSBOT Tiny 4K camera and microphone.

pub mod synthetic_data;

pub use synthetic_data::{
    synthetic_video_frame, 
    synthetic_audio_frame, 
    ObsbotCharacteristics
};
