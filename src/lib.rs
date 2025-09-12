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

pub mod types;
pub mod commands;
pub mod camera;
pub mod permissions;
pub mod errors;
pub mod platform;

#[cfg(feature = "contextlite")]
pub mod contextlite;

// Tests module - available for external tests
pub mod tests;

// Re-exports for convenience
pub use types::{CameraDeviceInfo, CameraFormat, CameraFrame, CameraInitParams, Platform};
pub use errors::CameraError;
pub use platform::{PlatformCamera, CameraSystem};

use tauri::{plugin::{Builder, TauriPlugin}, Runtime};

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
            
            // Permission commands
            commands::permissions::request_camera_permission,
            commands::permissions::check_camera_permission_status,
            
            // Capture commands
            commands::capture::capture_single_photo,
            commands::capture::capture_photo_sequence,
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
            commands::advanced::capture_focus_stack,
            commands::advanced::get_camera_performance,
            commands::advanced::test_camera_capabilities,
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
