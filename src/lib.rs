//! CrabCamera: Advanced cross-platform camera integration for Tauri applications
//!
//! This crate provides unified camera access across desktop platforms
//! with real-time processing capabilities and professional camera controls.
//!
//! # Features
//! - Cross-platform camera access (Windows, macOS, Linux)
//! - Real-time camera streaming and capture
//! - Platform-specific optimizations
//! - Professional camera controls
//! - Thread-safe camera management
//! - Multiple camera format support
//!
//! # Usage
//! Add this to your `tauri.conf.json`:
//! ```json
//! {
//!   "plugins": {
//!     "crabcamera": {}
//!   }
//! }
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
pub mod commands;pub mod config;
pub mod errors;
pub mod focus_stack;
#[cfg(feature = "headless")]
pub mod headless;
pub mod permissions;
pub mod platform;
pub mod quality;
#[cfg(any(feature = "headless", feature = "audio"))]
pub mod timing;
pub mod types;
pub mod webrtc;

#[cfg(feature = "recording")]
pub mod recording;

#[cfg(feature = "audio")]
pub mod audio;

// Tests module - available for external tests
pub mod tests;

// Testing utilities - synthetic data for offline testing
pub mod testing;

// Re-exports for convenience
pub use errors::CameraError;
pub use platform::{CameraSystem, PlatformCamera};
pub use types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams, FrameMetadata, Platform};

#[cfg(feature = "headless")]
pub use headless::{list_devices, list_formats, list_controls, HeadlessSession};

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

/// Initialize the CrabCamera plugin with all commands
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("crabcamera")
        .invoke_handler(tauri::generate_handler![
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
            commands::capture::start_camera_preview,
            commands::capture::stop_camera_preview,
            commands::capture::release_camera,
            commands::capture::get_capture_stats,
            commands::capture::save_frame_to_disk,
            commands::capture::save_frame_compressed,
            // Advanced camera commands
            commands::advanced::set_camera_controls,
            commands::advanced::get_camera_controls,
            commands::advanced::capture_burst_sequence,
            commands::advanced::set_manual_focus,
            commands::advanced::set_manual_exposure,
            commands::advanced::set_white_balance,
            commands::advanced::capture_hdr_sequence,
            commands::advanced::capture_focus_stack_legacy,
            commands::advanced::get_camera_performance,
            commands::advanced::test_camera_capabilities,
            // WebRTC streaming commands
            #[cfg(feature = "webrtc")]
            commands::webrtc::start_webrtc_stream,
            #[cfg(feature = "webrtc")]
            commands::webrtc::stop_webrtc_stream,
            #[cfg(feature = "webrtc")]
            commands::webrtc::get_webrtc_stream_status,
            #[cfg(feature = "webrtc")]
            commands::webrtc::update_webrtc_config,
            #[cfg(feature = "webrtc")]
            commands::webrtc::list_webrtc_streams,
            #[cfg(feature = "webrtc")]
            commands::webrtc::create_peer_connection,
            #[cfg(feature = "webrtc")]
            commands::webrtc::create_webrtc_offer,
            #[cfg(feature = "webrtc")]
            commands::webrtc::create_webrtc_answer,
            #[cfg(feature = "webrtc")]
            commands::webrtc::set_remote_description,
            #[cfg(feature = "webrtc")]
            commands::webrtc::add_ice_candidate,
            #[cfg(feature = "webrtc")]
            commands::webrtc::get_local_ice_candidates,
            #[cfg(feature = "webrtc")]
            commands::webrtc::add_video_transceivers,
            #[cfg(feature = "webrtc")]
            commands::webrtc::create_data_channel,
            #[cfg(feature = "webrtc")]
            commands::webrtc::send_data_channel_message,
            #[cfg(feature = "webrtc")]
            commands::webrtc::get_peer_connection_status,
            #[cfg(feature = "webrtc")]
            commands::webrtc::close_peer_connection,
            #[cfg(feature = "webrtc")]
            commands::webrtc::list_peer_connections,
            #[cfg(feature = "webrtc")]
            commands::webrtc::get_webrtc_system_status,
            #[cfg(feature = "webrtc")]
            commands::webrtc::pause_webrtc_stream,
            #[cfg(feature = "webrtc")]
            commands::webrtc::resume_webrtc_stream,
            #[cfg(feature = "webrtc")]
            commands::webrtc::set_webrtc_stream_bitrate,
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
pub const NAME: &str = env!("CARGO_PKG_NAME");
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
    pub name: String,
    pub version: String,
    pub description: String,
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
