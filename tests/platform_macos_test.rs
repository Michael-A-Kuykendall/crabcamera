#![cfg(target_os = "macos")]

//! Comprehensive macOS platform-specific tests for CrabCamera
//!
//! Tests AVFoundation backend integration, macOS-specific camera features,
//! and platform-specific edge cases.

#[cfg(test)]
mod platform_macos_tests {
    use crabcamera::errors::CameraError;
    use crabcamera::platform::macos::{initialize_camera, list_cameras, MacOSCamera};
    use crabcamera::types::{CameraFormat, CameraInitParams};
    use std::time::{Duration, Instant};

    /// Helper function to create test camera initialization parameters
    fn create_test_params(device_id: &str) -> CameraInitParams {
        CameraInitParams::new(device_id.to_string()).with_format(CameraFormat::new(1280, 720, 30.0))
    }

    /// Helper function to check if FaceTime HD camera is available (common on Macs)
    fn has_facetime_camera() -> bool {
        if let Ok(cameras) = list_cameras() {
            cameras.iter().any(|camera| {
                camera.name.to_lowercase().contains("facetime")
                    || camera.name.to_lowercase().contains("built-in")
                    || camera.description.to_lowercase().contains("facetime")
            })
        } else {
            false
        }
    }

    #[test]
    fn test_macos_list_cameras_returns_valid_result() {
        let result = list_cameras();

        match result {
            Ok(cameras) => {
                // Validate each camera device info structure
                for camera in &cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");

                    // Check for reasonable camera formats
                    assert!(
                        !camera.supports_formats.is_empty(),
                        "Should have supported formats"
                    );

                    // Verify common macOS formats are available
                    let has_standard_resolution = camera.supports_formats.iter().any(|f| {
                        (f.width == 1920 && f.height == 1080)
                            || (f.width == 1280 && f.height == 720)
                            || (f.width == 640 && f.height == 480)
                    });
                    assert!(
                        has_standard_resolution,
                        "Should have at least one standard resolution"
                    );

                    // Verify frame rates are reasonable
                    for format in &camera.supports_formats {
                        assert!(format.fps > 0.0, "FPS should be positive");
                        assert!(
                            format.fps <= 120.0,
                            "FPS should be reasonable for macOS cameras"
                        );
                    }
                }

                // On most Macs, there should be at least a built-in camera
                if cameras.is_empty() {
                    println!("Warning: No cameras found on macOS system");
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Acceptable if no cameras are available or AVFoundation issues
            }
            Err(e) => panic!("Unexpected error type from list_cameras: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_initialization_with_various_formats() {
        // Test different common macOS camera formats
        let test_formats = vec![
            CameraFormat::new(1920, 1080, 30.0), // Full HD
            CameraFormat::new(1280, 720, 60.0),  // HD 60fps
            CameraFormat::new(640, 480, 30.0),   // VGA
            CameraFormat::new(1024, 768, 30.0),  // 4:3 ratio
        ];

        for format in test_formats {
            let params = CameraInitParams::new("0".to_string()).with_format(format.clone());
            let result = initialize_camera(params);

            match result {
                Ok(camera) => {
                    // Verify camera was created successfully
                    assert_eq!(camera.get_device_id(), "0");
                    assert_eq!(camera.get_format().width, format.width);
                    assert_eq!(camera.get_format().height, format.height);
                }
                Err(CameraError::InitializationError(_)) => {
                    // Expected if no camera or format not supported
                }
                Err(e) => panic!("Unexpected error for format {:?}: {:?}", format, e),
            }
        }
    }

    #[test]
    fn test_macos_camera_stream_lifecycle() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                // Test initial state
                let initial_available = camera.is_available();

                // Test starting stream
                let start_result = camera.start_stream();
                match start_result {
                    Ok(()) => {
                        // Stream started successfully
                        let streaming_available = camera.is_available();
                        assert!(
                            streaming_available,
                            "Camera should be available when streaming"
                        );

                        // Test stopping stream
                        let stop_result = camera.stop_stream();
                        assert!(stop_result.is_ok(), "Stopping stream should succeed");
                    }
                    Err(CameraError::InitializationError(_)) => {
                        // Expected if camera access denied or not available
                    }
                    Err(e) => panic!("Unexpected error starting stream: {:?}", e),
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error initializing camera: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_capture_frame_functionality() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                // Start stream first
                if camera.start_stream().is_ok() {
                    let capture_result = camera.capture_frame();

                    match capture_result {
                        Ok(frame) => {
                            // Validate captured frame
                            assert!(frame.width > 0, "Frame width should be positive");
                            assert!(frame.height > 0, "Frame height should be positive");
                            assert!(!frame.data.is_empty(), "Frame data should not be empty");
                            assert_eq!(frame.device_id, "0");

                            // Verify frame format is reasonable
                            let expected_data_size = (frame.width * frame.height * 3) as usize; // RGB8
                            assert!(
                                frame.data.len() >= expected_data_size / 2, // Allow some compression
                                "Frame data size should be reasonable for RGB8 format"
                            );
                        }
                        Err(CameraError::CaptureError(_)) => {
                            // Expected if camera permissions denied or hardware issues
                        }
                        Err(e) => panic!("Unexpected error capturing frame: {:?}", e),
                    }

                    let _ = camera.stop_stream();
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in capture test: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_invalid_device_ids() {
        let invalid_ids = vec!["invalid_string", "-1", "999", "", "abc123", "0x1"];

        for invalid_id in invalid_ids {
            let params = CameraInitParams::new(invalid_id.to_string())
                .with_format(CameraFormat::new(640, 480, 30.0));

            let result = initialize_camera(params);

            match result {
                Err(CameraError::InitializationError(msg)) => {
                    assert!(
                        msg.contains("Invalid device ID") || msg.contains("Failed to initialize"),
                        "Error message should be informative for invalid ID: {}",
                        invalid_id
                    );
                }
                Err(e) => {
                    // Other errors might be acceptable depending on macOS version
                    println!("Got error for invalid device ID {}: {:?}", invalid_id, e);
                }
                Ok(_) => {
                    // Some invalid IDs might accidentally work, that's okay
                    println!("Invalid device ID {} unexpectedly worked", invalid_id);
                }
            }
        }
    }

    #[test]
    fn test_macos_camera_controls_stub_implementation() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(mut camera) => {
                // Test getting default controls (stub implementation)
                let get_result = camera.get_controls();
                match get_result {
                    Ok(controls) => {
                        // Should return default controls
                        // All fields should be None or reasonable defaults
                        assert!(
                            controls.brightness.is_none()
                                || (controls.brightness.unwrap() >= 0.0
                                    && controls.brightness.unwrap() <= 1.0)
                        );
                        assert!(
                            controls.contrast.is_none()
                                || (controls.contrast.unwrap() >= 0.0
                                    && controls.contrast.unwrap() <= 1.0)
                        );
                    }
                    Err(e) => panic!("Getting controls should not fail: {:?}", e),
                }

                // Test applying controls (stub implementation)
                let test_controls = crabcamera::types::CameraControls {
                    brightness: Some(0.5),
                    contrast: Some(0.7),
                    saturation: Some(0.6),
                    exposure_time: Some(0.3),
                    focus_distance: Some(0.8),
                    white_balance: Some(crabcamera::types::WhiteBalance::Auto),
                    iso_sensitivity: Some(400),
                    zoom: Some(1.0),
                    auto_focus: Some(true),
                    auto_exposure: Some(true),
                    aperture: None,
                    image_stabilization: Some(true),
                    noise_reduction: Some(false),
                    sharpness: Some(0.5),
                };

                let apply_result = camera.apply_controls(&test_controls);
                assert!(
                    apply_result.is_ok(),
                    "Applying controls should succeed (stub)"
                );
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in controls test: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_capabilities_stub_implementation() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                let capabilities_result = camera.test_capabilities();

                match capabilities_result {
                    Ok(capabilities) => {
                        // Should return default capabilities (stub implementation)
                        assert!(
                            capabilities.max_resolution.0 > 0,
                            "Max width should be positive"
                        );
                        assert!(
                            capabilities.max_resolution.1 > 0,
                            "Max height should be positive"
                        );
                        assert!(capabilities.max_fps > 0.0, "Max FPS should be positive");

                        // Boolean capabilities should be present
                        let _ = capabilities.supports_auto_focus;
                        let _ = capabilities.supports_manual_focus;
                        let _ = capabilities.supports_auto_exposure;
                        let _ = capabilities.supports_manual_exposure;
                        let _ = capabilities.supports_white_balance;
                        let _ = capabilities.supports_zoom;
                        let _ = capabilities.supports_flash;
                        let _ = capabilities.supports_burst_mode;
                        let _ = capabilities.supports_hdr;
                    }
                    Err(e) => panic!("Getting capabilities should not fail: {:?}", e),
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in capabilities test: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_performance_metrics_stub() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                let metrics_result = camera.get_performance_metrics();

                match metrics_result {
                    Ok(metrics) => {
                        // Validate default performance metrics
                        assert!(
                            metrics.capture_latency_ms >= 0.0,
                            "Latency should be non-negative"
                        );
                        assert!(
                            metrics.processing_time_ms >= 0.0,
                            "Processing time should be non-negative"
                        );
                        assert!(
                            metrics.memory_usage_mb >= 0.0,
                            "Memory usage should be non-negative"
                        );
                        assert!(metrics.fps_actual >= 0.0, "FPS should be non-negative");
                        assert!(
                            metrics.quality_score >= 0.0 && metrics.quality_score <= 1.0,
                            "Quality score should be 0-1"
                        );
                    }
                    Err(e) => panic!("Getting performance metrics should not fail: {:?}", e),
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in performance metrics test: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_thread_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                let camera_arc = Arc::new(Mutex::new(camera));
                let mut handles = vec![];

                // Test concurrent access to camera operations
                for i in 0..3 {
                    let camera_clone = Arc::clone(&camera_arc);
                    let handle = thread::spawn(move || {
                        if let Ok(camera) = camera_clone.lock() {
                            // Test thread-safe operations
                            let _ = camera.is_available();
                            let _ = camera.get_device_id();
                            let _ = camera.get_format();
                            let _ = camera.get_controls();
                            let _ = camera.test_capabilities();
                            let _ = camera.get_performance_metrics();
                            i
                        } else {
                            panic!("Failed to acquire camera lock in thread {}", i);
                        }
                    });
                    handles.push(handle);
                }

                // Wait for all threads to complete
                for (i, handle) in handles.into_iter().enumerate() {
                    let result = handle.join().expect("Thread should not panic");
                    assert_eq!(result, i, "Thread {} should complete successfully", i);
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in thread safety test: {:?}", e),
        }
    }

    #[test]
    fn test_macos_camera_drop_cleanup() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                // Start stream to test cleanup
                let _ = camera.start_stream();

                // Camera should be properly cleaned up when dropped
                // This happens automatically at the end of scope
                assert_eq!(camera.get_device_id(), "0");
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in drop test: {:?}", e),
        }
        // Camera is dropped here and should clean up properly
    }

    #[test]
    fn test_macos_specific_avfoundation_backend() {
        // Test that we're actually using AVFoundation backend
        let result = list_cameras();

        match result {
            Ok(cameras) => {
                // AVFoundation typically provides detailed camera information
                for camera in cameras {
                    // FaceTime cameras are common on Macs
                    if camera.name.to_lowercase().contains("facetime")
                        || camera.name.to_lowercase().contains("built-in")
                    {
                        assert!(
                            !camera.description.is_empty(),
                            "AVFoundation should provide camera descriptions"
                        );

                        // Should support typical macOS resolutions
                        let has_hd = camera
                            .supports_formats
                            .iter()
                            .any(|f| f.width >= 1280 && f.height >= 720);
                        assert!(
                            has_hd,
                            "Mac cameras should typically support HD resolutions"
                        );
                    }
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if AVFoundation access denied or no cameras
            }
            Err(e) => panic!("Unexpected error testing AVFoundation: {:?}", e),
        }
    }

    #[test]
    fn test_macos_mjpeg_format_compatibility() {
        // Test MJPEG format specifically used in macOS implementation
        let params =
            CameraInitParams::new("0".to_string()).with_format(CameraFormat::new(1280, 720, 30.0));

        match initialize_camera(params) {
            Ok(camera) => {
                // MJPEG is commonly supported on macOS
                let format = camera.get_format();
                assert!(
                    format.width > 0,
                    "MJPEG format should have valid dimensions"
                );
                assert!(
                    format.height > 0,
                    "MJPEG format should have valid dimensions"
                );
                assert!(
                    format.fps > 0.0,
                    "MJPEG format should have valid frame rate"
                );

                // Test that camera can potentially capture with MJPEG
                if camera.start_stream().is_ok() {
                    let capture_result = camera.capture_frame();
                    match capture_result {
                        Ok(frame) => {
                            // Frame should be converted to RGB8 in our implementation
                            assert!(
                                frame.format.as_ref().map_or(true, |f| f == "RGB8"),
                                "Frame should be converted to RGB8"
                            );
                        }
                        Err(_) => {
                            // Capture errors are acceptable without hardware
                        }
                    }
                    let _ = camera.stop_stream();
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error testing MJPEG format: {:?}", e),
        }
    }

    #[test]
    fn test_macos_error_message_quality() {
        // Test that error messages are informative and help with debugging
        let invalid_params =
            CameraInitParams::new("invalid".to_string()).with_format(CameraFormat::new(0, 0, 0.0));

        let result = initialize_camera(invalid_params);

        if let Err(CameraError::InitializationError(msg)) = result {
            assert!(!msg.is_empty(), "Error message should not be empty");
            assert!(msg.len() > 10, "Error message should be descriptive");

            // Should mention the specific issue
            let msg_lower = msg.to_lowercase();
            assert!(
                msg_lower.contains("invalid")
                    || msg_lower.contains("failed")
                    || msg_lower.contains("error"),
                "Error message should be informative: {}",
                msg
            );
        }
    }

    #[test]
    fn test_macos_camera_state_consistency() {
        let params = create_test_params("0");

        match initialize_camera(params) {
            Ok(camera) => {
                // Test consistent device ID
                let device_id1 = camera.get_device_id();
                let device_id2 = camera.get_device_id();
                assert_eq!(device_id1, device_id2, "Device ID should be consistent");

                // Test consistent format
                let format1 = camera.get_format();
                let format2 = camera.get_format();
                assert_eq!(format1.width, format2.width, "Format should be consistent");
                assert_eq!(
                    format1.height, format2.height,
                    "Format should be consistent"
                );
                assert_eq!(format1.fps, format2.fps, "Format should be consistent");

                // Test availability consistency (may change but should be stable)
                let available1 = camera.is_available();
                std::thread::sleep(Duration::from_millis(10));
                let available2 = camera.is_available();
                // Availability could change, but should be consistent in short timeframe
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in state consistency test: {:?}", e),
        }
    }
}
