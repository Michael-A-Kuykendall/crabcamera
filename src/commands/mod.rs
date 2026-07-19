/// Advanced camera controls.
pub mod advanced;
/// Photo capture commands.
pub mod capture;
/// Configuration commands.
pub mod config;
/// Device monitoring events.
pub mod device_monitor;
/// Focus stacking operations.
pub mod focus_stack;
/// Initialization and diagnostics.
pub mod init;
/// Permission handling.
pub mod permissions;
/// Preview stream commands (Tauri only).
#[cfg(feature = "tauri")]
pub mod preview;
/// Image quality analysis.
pub mod quality;

#[cfg(feature = "recording")]
pub mod recording;

#[cfg(feature = "audio")]
pub mod audio;
