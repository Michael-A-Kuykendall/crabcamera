//! `CrabCamera`: Advanced cross-platform camera integration for Tauri applications
//!
//! This crate provides unified camera access across desktop platforms
//! with real-time processing capabilities and professional camera controls.
//!
//! It aims to be "The Standard" for high-quality, professional-grade camera
//! applications on desktop platforms.
//!
//! # Strict Engineering Standards
//! This codebase enforces:
//! - Complete Documentation (`#![warn(missing_docs)]`)
//! - Safe Code (no `unwrap()` or `expect()` in library code via `clippy::unwrap_used`)
//! - Idiomatic Rust Practices (`clippy::pedantic`)
//!
//! # Features
//! - Cross-platform camera access (Windows, macOS, Linux)
//! - Real-time camera streaming and capture
//! - Platform-specific optimizations
//! - Professional camera controls
//! - Thread-safe camera management
//! - Multiple camera format support (RGB, YUYV, MJPEG)
//!
//! # Usage
//! Add this to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! crabcamera = { version = "0.8", features = ["recording", "audio"] }
//! tauri = { version = "2.0", features = ["protocol-asset"] }
//! ```
//!
//! Then in your Tauri app:
//! ```rust,ignore
//! use crabcamera;
//!
//! fn main() {
//!     tauri::Builder::default()
//!         .plugin(crabcamera::init())
//!         .run(tauri::generate_context!())
//!         .expect("error while running tauri application");
//! }
//! ```

#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::unwrap_used)]
// Common enough reasonable exceptions
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

#[cfg(feature = "tauri")]
/// Tauri command handlers.
pub mod commands;

/// Global constants.
pub mod constants;

/// Configuration management.
pub mod config;

/// Error types.
pub mod errors;

/// Automatic focus stacking.
pub mod focus_stack;

#[cfg(feature = "headless")]
/// Headless capture session management.
pub mod headless;

/// Invariant checks for PPT.
pub mod invariant_ppt;

/// Permission management.
pub mod permissions;

/// Platform abstraction layer.
pub mod platform;

/// System capabilities registry and manifest (Source of Truth).
pub mod registry;

/// Image quality analysis.
pub mod quality;

#[cfg(any(feature = "headless", feature = "audio"))]
/// Timing utilities.
pub mod timing;
/// Common data types and structures.
pub mod types;

/// Preview stream module.
pub mod preview;

#[cfg(feature = "recording")]
/// Video recording and encoding.
pub mod recording;

#[cfg(feature = "audio")]
/// Audio capture and processing.
pub mod audio;

// Tests module - available for external tests
/// Integration tests and test utilities.
pub mod tests;

// Testing utilities - synthetic data for offline testing
/// Testing utilities.
pub mod testing;

// Re-exports for convenience
pub use errors::CameraError;
pub use platform::{CameraSystem, PlatformCamera};
pub use types::{
    CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams, FrameMetadata, Platform,
};

#[cfg(feature = "headless")]
pub use headless::{list_controls, list_devices, list_formats, HeadlessSession};

#[cfg(feature = "tauri")]
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

/// Initialize the `CrabCamera` plugin with all commands
#[cfg(feature = "tauri")]
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("crabcamera")
        .invoke_handler(tauri::generate_handler![
            commands::init::get_system_manifest,
            // Initialization commands
            commands::init::initialize_camera_system,
            commands::init::get_available_cameras,
            commands::init::get_platform_info,
            commands::init::test_camera_system,
            commands::init::get_current_platform,
            commands::init::check_camera_availability,
            commands::init::get_camera_formats,
            commands::init::get_recommended_format,
            commands::init::get_optimal_settings,
            commands::init::get_system_diagnostics,
            // Permission commands
            commands::permissions::request_camera_permission,
            commands::permissions::check_camera_permission_status,
            commands::permissions::get_permission_status_string,
            // Capture commands
            commands::capture::capture_single_photo,
            commands::capture::capture_photo_sequence,
            commands::capture::capture_with_quality_retry,
            commands::capture::capture,
            commands::capture::start_camera_preview,
            commands::capture::stop_camera_preview,
            commands::capture::release_camera,
            commands::capture::get_capture_stats,
            commands::capture::save_frame_to_disk,
            commands::capture::save_frame_compressed,
            commands::capture::set_frame_callback,
            // Advanced camera commands
            commands::advanced::set_camera_controls,
            commands::advanced::get_camera_controls,
            commands::advanced::capture_burst_sequence,
            commands::advanced::apply_camera_settings,
            commands::advanced::set_manual_focus,
            commands::advanced::set_manual_exposure,
            commands::advanced::set_white_balance,
            commands::advanced::capture_hdr_sequence,
            commands::advanced::capture_focus_stack_legacy,
            commands::advanced::get_camera_performance,
            commands::advanced::test_camera_capabilities,
            // Quality validation commands
            commands::quality::validate_frame_quality,
            commands::quality::validate_provided_frame,
            commands::quality::analyze_frame_blur,
            commands::quality::analyze_frame_exposure,
            commands::quality::update_quality_config,
            commands::quality::get_quality_config,
            commands::quality::capture_best_quality_frame,
            commands::quality::auto_capture_with_quality,
            commands::quality::analyze_quality_trends,
            // Configuration commands
            commands::config::get_config,
            commands::config::update_config,
            commands::config::reset_config,
            commands::config::get_camera_config,
            commands::config::get_full_quality_config,
            commands::config::get_storage_config,
            commands::config::get_advanced_config,
            commands::config::update_camera_config,
            commands::config::update_full_quality_config,
            commands::config::update_storage_config,
            commands::config::update_advanced_config,
            // Device monitoring commands
            commands::device_monitor::start_device_monitoring,
            commands::device_monitor::stop_device_monitoring,
            commands::device_monitor::poll_device_event,
            commands::device_monitor::get_monitored_devices,
            // Focus stacking commands
            commands::focus_stack::capture_focus_stack,
            commands::focus_stack::capture_focus_brackets_command,
            commands::focus_stack::get_default_focus_config,
            commands::focus_stack::validate_focus_config,
            // Preview stream commands
            commands::preview::start_preview_stream,
            commands::preview::stop_preview_stream,
        ])
        .build()
}

/// Detect the current platform using the Platform enum
pub fn current_platform() -> Platform {
    Platform::current()
}

/// Get current platform as string (legacy compatibility)
pub fn current_platform_string() -> String {
    Platform::current().as_str().to_string()
}

/// Initialize logging for the camera system
pub fn init_logging() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "crabcamera=info");
    }
    let _ = env_logger::try_init();
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The name of the crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");
/// A brief description of the crate.
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// Get crate information
pub fn get_info() -> CrateInfo {
    CrateInfo {
        name: NAME.to_string(),
        version: VERSION.to_string(),
        description: DESCRIPTION.to_string(),
        platform: Platform::current(),
    }
}

/// Crate information structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CrateInfo {
    /// Crate name.
    pub name: String,
    /// Crate version.
    pub version: String,
    /// Crate description.
    pub description: String,
    /// Host platform.
    pub platform: Platform,
}

#[cfg(test)]
mod lib_tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = current_platform();
        assert_ne!(platform, Platform::Unknown);
    }

    #[test]
    fn test_platform_string() {
        let platform_str = current_platform_string();
        assert!(!platform_str.is_empty());
    }

    #[test]
    fn test_crate_info() {
        let info = get_info();
        assert_eq!(info.name, "crabcamera");
        assert!(!info.version.is_empty());
        assert!(!info.description.is_empty());
    }
}
