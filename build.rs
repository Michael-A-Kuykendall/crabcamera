#[cfg(feature = "tauri")]
const COMMANDS: &[&str] = &[
    "initialize_camera_system",
    "get_available_cameras",
    "get_platform_info",
    "test_camera_system",
    "get_current_platform",
    "check_camera_availability",
    "get_camera_formats",
    "get_recommended_format",
    "get_optimal_settings",
    "get_system_diagnostics",
    "request_camera_permission",
    "check_camera_permission_status",
    "get_permission_status_string",
    "capture_single_photo",
    "capture_photo_sequence",
    "capture_with_quality_retry",
    "start_camera_preview",
    "stop_camera_preview",
    "release_camera",
    "get_capture_stats",
    "save_frame_to_disk",
    "save_frame_compressed",
    "set_frame_callback",
    "set_camera_controls",
    "get_camera_controls",
    "capture_burst_sequence",
    "set_manual_focus",
    "set_manual_exposure",
    "set_white_balance",
    "capture_hdr_sequence",
    "capture_focus_stack_legacy",
    "get_camera_performance",
    "test_camera_capabilities",
    "validate_frame_quality",
    "validate_provided_frame",
    "analyze_frame_blur",
    "analyze_frame_exposure",
    "update_quality_config",
    "get_quality_config",
    "capture_best_quality_frame",
    "auto_capture_with_quality",
    "analyze_quality_trends",
    "get_config",
    "update_config",
    "reset_config",
    "get_camera_config",
    "get_full_quality_config",
    "get_storage_config",
    "get_advanced_config",
    "update_camera_config",
    "update_full_quality_config",
    "update_storage_config",
    "update_advanced_config",
    "start_device_monitoring",
    "stop_device_monitoring",
    "poll_device_event",
    "get_monitored_devices",
    "capture_focus_stack",
    "capture_focus_brackets_command",
    "get_default_focus_config",
    "validate_focus_config",
];

fn main() {
    #[cfg(feature = "tauri")]
    tauri_plugin::Builder::new(COMMANDS).build();

    // When the audio feature is enabled, we need to ensure the opus library is linked.
    // opus-static-sys builds opus and sets up link paths, but we need to propagate them.
    #[cfg(feature = "audio")]
    {
        // Find the opus-static-sys build output directory
        // The OUT_DIR pattern for dependencies is: target/{profile}/build/{crate-name}-{hash}/out
        if let Ok(out_dir) = std::env::var("OUT_DIR") {
            // out_dir is something like: target/debug/build/crabcamera-xxx/out
            // We need to find: target/debug/build/opus-static-sys-xxx/out/lib
            let target_dir = std::path::Path::new(&out_dir)
                .parent() // build/crabcamera-xxx
                .and_then(|p| p.parent()) // build
                .expect("Could not find build directory");

            // Search for opus-static-sys output directory
            if let Ok(entries) = std::fs::read_dir(target_dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with("opus-static-sys-") {
                        let opus_lib_dir = entry.path().join("out").join("lib");
                        if opus_lib_dir.exists() {
                            println!("cargo:rustc-link-search=native={}", opus_lib_dir.display());
                            println!("cargo:rustc-link-lib=static=opus");
                            println!("cargo:rerun-if-changed={}", opus_lib_dir.display());
                            return;
                        }
                    }
                }
            }
        }

        // Fallback: try DEP_ variable (works for some build configurations)
        if let Ok(lib_path) = std::env::var("DEP_OPUS_LIB_DIR") {
            println!("cargo:rustc-link-search=native={}", lib_path);
            println!("cargo:rustc-link-lib=static=opus");
        }
    }
}
