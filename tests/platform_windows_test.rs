#![cfg(windows)]

//! Comprehensive Windows platform-specific tests for CrabCamera
//!
//! Tests MediaFoundation backend integration, DirectShow compatibility,
//! Windows-specific camera controls, and platform-specific edge cases.

#[cfg(test)]
mod platform_windows_tests {
    use crabcamera::errors::CameraError;
    use crabcamera::platform::windows::{initialize_camera, list_cameras, WindowsCamera};
    use crabcamera::types::{CameraFormat, CameraInitParams, WhiteBalance, CameraControls};
    use std::time::Duration;

    /// Helper function to create test camera initialization parameters
    fn create_test_params(device_id: &str) -> CameraInitParams {
        CameraInitParams::new(device_id.to_string())
            .with_format(CameraFormat::new(640, 480, 30.0))
    }

    /// Helper function to check if we're running in a CI/container environment
    fn is_ci_environment() -> bool {
        std::env::var("CI").is_ok() || 
        std::env::var("GITHUB_ACTIONS").is_ok() ||
        std::env::var("APPVEYOR").is_ok()
    }

    #[test]
    fn test_windows_list_cameras_returns_result() {
        // This test may fail on systems without cameras, but should not panic
        let result = list_cameras();

        // The function should return a Result
        match result {
            Ok(cameras) => {
                // If successful, cameras should be a valid Vec
                // cameras.len() is usize, always >= 0; could be empty if no cameras

                // Test each camera device info
                for camera in &cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");
                    
                    // Verify device ID is numeric (required for Windows implementation)
                    let parse_result: Result<u32, _> = camera.id.parse();
                    assert!(
                        parse_result.is_ok(),
                        "Camera ID should be numeric on Windows: {}",
                        camera.id
                    );

                    // Windows should have standard formats
                    assert!(
                        camera.supports_formats.len() >= 3,
                        "Should have at least 3 standard formats"
                    );

                    // Verify standard Windows formats are present
                    let has_1080p = camera
                        .supports_formats
                        .iter()
                        .any(|f| f.width == 1920 && f.height == 1080);
                    let has_720p = camera
                        .supports_formats
                        .iter()
                        .any(|f| f.width == 1280 && f.height == 720);
                    let has_480p = camera
                        .supports_formats
                        .iter()
                        .any(|f| f.width == 640 && f.height == 480);

                    assert!(
                        has_1080p || has_720p || has_480p,
                        "Should have at least one standard format"
                    );

                    // Verify frame rates are reasonable for Windows
                    for format in &camera.supports_formats {
                        assert!(format.fps > 0.0, "FPS should be positive");
                        assert!(format.fps <= 120.0, "FPS should be reasonable for Windows cameras");
                        
                        // Verify aspect ratios
                        let aspect_ratio = format.width as f64 / format.height as f64;
                        assert!(
                            aspect_ratio > 0.5 && aspect_ratio < 5.0,
                            "Aspect ratio should be reasonable: {}",
                            aspect_ratio
                        );
                    }

                    // Check for Windows-specific camera names
                    let name_lower = camera.name.to_lowercase();
                    if name_lower.contains("obs") {
                        println!("Found OBS Virtual Camera: {}", camera.name);
                    } else if name_lower.contains("integrated") || name_lower.contains("webcam") {
                        println!("Found integrated webcam: {}", camera.name);
                    }
                }

                if cameras.is_empty() && !is_ci_environment() {
                    println!("Warning: No cameras found on Windows system");
                }
            }
            Err(e) => {
                // If it fails, should be a proper CameraError
                match e {
                    CameraError::InitializationError(_) => {
                        // This is expected if no cameras are available
                    }
                    _ => panic!("Unexpected error type: {:?}", e),
                }
            }
        }
    }

    #[test]
    fn test_initialize_camera_with_invalid_device_id() {
        let format = CameraFormat::new(640, 480, 30.0);

        // Test with invalid device ID (non-numeric)
        let result = initialize_camera("invalid", format.clone());
        assert!(result.is_err());

        if let Err(CameraError::InitializationError(msg)) = result {
            assert!(msg.contains("Invalid device ID"));
        } else {
            panic!("Expected InitializationError for invalid device ID");
        }

        // Test with out-of-range device ID
        let result = initialize_camera("999", format);
        // This might succeed or fail depending on system, but should not panic
        match result {
            Ok(_) => {}                                    // Camera might exist
            Err(CameraError::InitializationError(_)) => {} // Expected failure
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_initialize_camera_with_valid_format() {
        let format = CameraFormat::new(640, 480, 30.0);

        // Try with device ID "0" (most common default camera)
        let result = initialize_camera("0", format);

        // This may succeed or fail depending on hardware, but should be handled gracefully
        match result {
            Ok(_camera) => {
                // If successful, we got a valid camera object
                // We can't test much without actually using the camera
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera is available
            }
            Err(e) => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_camera_formats_are_standard() {
        // Test that our standard formats are reasonable
        let formats = vec![
            CameraFormat::new(1920, 1080, 30.0),
            CameraFormat::new(1280, 720, 30.0),
            CameraFormat::new(640, 480, 30.0),
        ];

        for format in formats {
            assert!(format.width > 0, "Width should be positive");
            assert!(format.height > 0, "Height should be positive");
            assert!(format.fps > 0.0, "FPS should be positive");
            assert!(format.fps <= 60.0, "FPS should be reasonable (<=60)");

            // Test aspect ratios make sense
            let aspect_ratio = format.width as f64 / format.height as f64;
            assert!(
                aspect_ratio > 0.5 && aspect_ratio < 5.0,
                "Aspect ratio should be reasonable"
            );
        }
    }

    #[test]
    fn test_device_id_parsing() {
        // Test valid device IDs
        let valid_ids = vec!["0", "1", "2", "10"];
        for id in valid_ids {
            let parsed: Result<u32, _> = id.parse();
            assert!(parsed.is_ok(), "Device ID '{}' should be parseable", id);
        }

        // Test invalid device IDs
        let invalid_ids = vec!["abc", "-1", "", "1.5", "0x1"];
        for id in invalid_ids {
            let parsed: Result<u32, _> = id.parse();
            assert!(
                parsed.is_err(),
                "Device ID '{}' should not be parseable",
                id
            );
        }
    }

    #[test]
    fn test_error_messages_are_informative() {
        // Test error message formatting
        let format = CameraFormat::new(640, 480, 30.0);

        let result = initialize_camera("invalid_id", format);
        if let Err(CameraError::InitializationError(msg)) = result {
            assert!(!msg.is_empty(), "Error message should not be empty");
            assert!(
                msg.contains("Invalid device ID"),
                "Error message should mention invalid device ID"
            );
        }
    }

    #[test]
    fn test_windows_camera_initialization_with_various_formats() {
        // Test different common Windows camera formats
        let test_formats = vec![
            CameraFormat::new(1920, 1080, 30.0),  // Full HD
            CameraFormat::new(1280, 720, 60.0),   // HD 60fps  
            CameraFormat::new(640, 480, 30.0),    // VGA
            CameraFormat::new(320, 240, 30.0),    // QVGA
        ];
        
        for format in test_formats {
            let params = CameraInitParams::new("0".to_string()).with_format(format.clone());
            let result = WindowsCamera::new(params.device_id, params.format);
            
            match result {
                Ok(camera) => {
                    // Verify camera was created successfully
                    assert_eq!(camera.get_device_id(), "0");
                    assert!(!camera.device_id.is_empty());
                }
                Err(CameraError::InitializationError(_)) => {
                    // Expected if no camera or MediaFoundation issues
                }
                Err(e) => panic!("Unexpected error for format {:?}: {:?}", format, e),
            }
        }
    }

    #[test] 
    fn test_windows_camera_stream_lifecycle() {
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(mut camera) => {
                // Test initial state
                let initial_available = camera.is_available();
                assert!(initial_available, "Windows camera should be available initially");
                
                // Test starting stream
                let start_result = camera.start_stream();
                match start_result {
                    Ok(()) => {
                        // Stream started successfully 
                        assert!(camera.is_stream_open(), "Stream should be open");
                        
                        // Test stopping stream
                        let stop_result = camera.stop_stream();
                        assert!(stop_result.is_ok(), "Stopping stream should succeed");
                        
                        // Test stream state after stopping
                        assert!(!camera.is_stream_open(), "Stream should be closed");
                    }
                    Err(CameraError::StreamError(_)) => {
                        // Expected if camera access denied or hardware issues
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
    fn test_windows_camera_capture_frame_functionality() {
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(mut camera) => {
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
                            
                            // Should be converted to RGB8 on Windows
                            assert!(
                                frame.format == "RGB8",
                                "Frame should be RGB8 format"
                            );
                            
                            // Verify frame data size is reasonable
                            let expected_min_size = (frame.width * frame.height * 3) as usize; // RGB8
                            assert!(
                                frame.data.len() >= expected_min_size / 2, // Allow some compression
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
    fn test_windows_media_foundation_controls() {
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(mut camera) => {
                // Test MediaFoundation controls interface
                let test_controls = CameraControls {
                    brightness: Some(0.5),
                    contrast: Some(0.7),
                    saturation: Some(0.6),
                    exposure_time: Some(0.033),  // ~30fps
                    focus_distance: Some(0.8),
                    white_balance: Some(WhiteBalance::Daylight),
                    iso_sensitivity: Some(400),
                    zoom: Some(1.0),
                    auto_focus: Some(true),
                    auto_exposure: Some(true),
                    aperture: None,
                    image_stabilization: Some(true),
                    noise_reduction: Some(false),
                    sharpness: Some(0.5),
                };
                
                // Test applying controls
                let apply_result = camera.apply_controls(&test_controls);
                match apply_result {
                    Ok(unsupported) => {
                        // Some controls might not be supported - that's expected
                        println!("Unsupported Windows controls: {:?}", unsupported);
                    }
                    Err(e) => {
                        // Control errors are acceptable if MediaFoundation interfaces unavailable
                        println!("MediaFoundation controls error (expected): {:?}", e);
                    }
                }
                
                // Test getting current controls
                let get_result = camera.get_controls();
                match get_result {
                    Ok(controls) => {
                        // Should return some control values (might be defaults)
                        println!("Retrieved Windows camera controls: {:?}", controls);
                    }
                    Err(e) => {
                        // Control retrieval might fail if interfaces unavailable
                        println!("Could not retrieve controls (expected): {:?}", e);
                    }
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in MediaFoundation controls test: {:?}", e),
        }
    }

    #[test]
    fn test_windows_camera_capabilities() {
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(camera) => {
                let capabilities_result = camera.test_capabilities();
                
                match capabilities_result {
                    Ok(capabilities) => {
                        // Validate capability fields
                        assert!(capabilities.max_resolution.0 > 0, "Max width should be positive");
                        assert!(capabilities.max_resolution.1 > 0, "Max height should be positive");
                        assert!(capabilities.max_fps > 0.0, "Max FPS should be positive");
                        
                        // Windows should typically support burst mode
                        assert!(capabilities.supports_burst_mode, "Windows should support burst mode");
                        
                        // Log Windows-specific capabilities  
                        println!("Windows camera capabilities:");
                        println!("  Auto Focus: {}", capabilities.supports_auto_focus);
                        println!("  Manual Focus: {}", capabilities.supports_manual_focus);
                        println!("  Auto Exposure: {}", capabilities.supports_auto_exposure);
                        println!("  Manual Exposure: {}", capabilities.supports_manual_exposure);
                        println!("  White Balance: {}", capabilities.supports_white_balance);
                        println!("  Zoom: {}", capabilities.supports_zoom);
                        println!("  Flash: {}", capabilities.supports_flash);
                        println!("  HDR: {}", capabilities.supports_hdr);
                    }
                    Err(e) => {
                        println!("Could not retrieve capabilities (expected): {:?}", e);
                    }
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in capabilities test: {:?}", e),
        }
    }

    #[test]
    fn test_windows_camera_invalid_device_ids_extended() {
        let invalid_ids = vec![
            "invalid_string",
            "-1", 
            "999",
            "",
            "abc123",
            "0x1",
            "1.5",
            " 0 ",  // Spaces
            "9999999", // Very large number
        ];
        
        for invalid_id in invalid_ids {
            let params = CameraInitParams::new(invalid_id.to_string())
                .with_format(CameraFormat::new(640, 480, 30.0));
            
            let result = WindowsCamera::new(params.device_id, params.format);
            
            match result {
                Err(CameraError::InitializationError(msg)) => {
                    assert!(
                        msg.contains("Invalid device ID") || msg.contains("Failed to initialize"),
                        "Error message should be informative for invalid ID: {}",
                        invalid_id
                    );
                }
                Err(e) => {
                    // Other errors might be acceptable depending on MediaFoundation behavior  
                    println!("Got error for invalid device ID {}: {:?}", invalid_id, e);
                }
                Ok(_) => {
                    panic!("Invalid device ID {} should not work", invalid_id);
                }
            }
        }
    }

    #[test]
    fn test_windows_multiple_backend_support() {
        // Test that Windows implementation can handle multiple backends
        let result = list_cameras();
        
        match result {
            Ok(cameras) => {
                // Windows implementation should try MediaFoundation + Auto backends
                // Verify no duplicate cameras from different backends
                let mut seen_names = std::collections::HashSet::new();
                let mut duplicates = Vec::new();
                
                for camera in &cameras {
                    if !seen_names.insert(camera.name.clone()) {
                        duplicates.push(&camera.name);
                    }
                }
                
                assert!(
                    duplicates.is_empty(),
                    "Should not have duplicate cameras from different backends: {:?}",
                    duplicates
                );
                
                // Log backend discovery results
                if !cameras.is_empty() {
                    println!("Found {} Windows cameras without duplicates", cameras.len());
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no cameras on any backend
            }
            Err(e) => panic!("Unexpected error testing multiple backends: {:?}", e),
        }
    }

    #[test]
    fn test_windows_mjpeg_to_rgb_conversion() {
        // Test Windows-specific MJPEG to RGB8 conversion
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(mut camera) => {
                if camera.start_stream().is_ok() {
                    let capture_result = camera.capture_frame();
                    
                    match capture_result {
                        Ok(frame) => {
                            // Windows should convert MJPEG to RGB8
                            assert!(
                                frame.format == "RGB8",
                                "Windows should convert frames to RGB8"
                            );
                            
                            // Data should not look like MJPEG (no FF D8 FF header)
                            if frame.data.len() >= 3 {
                                let is_mjpeg = frame.data[0] == 0xFF && 
                                              frame.data[1] == 0xD8 && 
                                              frame.data[2] == 0xFF;
                                assert!(!is_mjpeg, "Frame data should not be MJPEG after conversion");
                            }
                        }
                        Err(_) => {
                            // Capture errors acceptable without hardware
                        }
                    }
                    
                    let _ = camera.stop_stream();
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error testing MJPEG conversion: {:?}", e),
        }
    }

    #[test]
    fn test_windows_camera_thread_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
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
                            let _ = camera.is_stream_open();
                            let _ = camera.get_controls();
                            let _ = camera.test_capabilities();
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
    fn test_windows_camera_drop_cleanup() {
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(mut camera) => {
                // Start stream to test cleanup
                let _ = camera.start_stream();
                
                // Camera should be properly cleaned up when dropped
                assert_eq!(camera.get_device_id(), "0");
                assert!(camera.is_available());
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in drop test: {:?}", e),
        }
        // Camera is dropped here and should clean up MediaFoundation resources
    }

    #[test]
    fn test_windows_error_message_quality() {
        // Test that error messages are informative for debugging
        let invalid_params = CameraInitParams::new("invalid".to_string())
            .with_format(CameraFormat::new(0, 0, 0.0));
        
        let result = WindowsCamera::new(invalid_params.device_id, invalid_params.format);
        
        if let Err(CameraError::InitializationError(msg)) = result {
            assert!(!msg.is_empty(), "Error message should not be empty");
            assert!(msg.len() > 10, "Error message should be descriptive");
            
            // Should mention the specific issue
            let msg_lower = msg.to_lowercase();
            assert!(
                msg_lower.contains("invalid") || 
                msg_lower.contains("failed") || 
                msg_lower.contains("error"),
                "Error message should be informative: {}", msg
            );
        }
    }

    #[test]
    fn test_windows_camera_state_consistency() {
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(camera) => {
                // Test consistent device ID
                let device_id1 = camera.get_device_id();
                let device_id2 = camera.get_device_id();
                assert_eq!(device_id1, device_id2, "Device ID should be consistent");
                
                // Test availability consistency
                let available1 = camera.is_available();
                std::thread::sleep(Duration::from_millis(10));
                let available2 = camera.is_available();
                assert_eq!(available1, available2, "Availability should be consistent");
                
                // Test stream state consistency
                let stream1 = camera.is_stream_open();
                let stream2 = camera.is_stream_open();
                assert_eq!(stream1, stream2, "Stream state should be consistent");
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in state consistency test: {:?}", e),
        }
    }

    #[test]
    fn test_windows_specific_media_foundation_backend() {
        // Test Windows MediaFoundation specific behavior
        let result = list_cameras();
        
        match result {
            Ok(cameras) => {
                for camera in cameras {
                    // Windows devices should have numeric IDs
                    let parse_result: Result<u32, _> = camera.id.parse();
                    assert!(
                        parse_result.is_ok(),
                        "Windows camera ID should be numeric: {}",
                        camera.id
                    );
                    
                    // Should have detailed descriptions from MediaFoundation
                    assert!(
                        camera.description.as_ref().map_or(false, |s| !s.is_empty()),
                        "MediaFoundation should provide camera descriptions"
                    );
                    
                    // Should support typical Windows resolutions
                    let has_hd = camera.supports_formats.iter().any(|f| {
                        f.width >= 1280 && f.height >= 720
                    });
                    assert!(has_hd, "Windows cameras should typically support HD resolutions");
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no MediaFoundation devices
            }
            Err(e) => panic!("Unexpected error testing MediaFoundation: {:?}", e),
        }
    }

    #[test] 
    fn test_windows_control_range_normalization() {
        // Test Windows-specific control range normalization
        let params = create_test_params("0");
        
        match WindowsCamera::new(params.device_id, params.format) {
            Ok(mut camera) => {
                // Test normalized control values (0.0-1.0)
                let normalized_controls = CameraControls {
                    brightness: Some(0.0),    // Minimum
                    contrast: Some(0.5),      // Middle
                    saturation: Some(1.0),    // Maximum
                    focus_distance: Some(0.75), // 3/4
                    exposure_time: Some(0.033), // 1/30 second
                    white_balance: Some(WhiteBalance::Custom(5500)),
                    auto_focus: Some(false),
                    auto_exposure: Some(false),
                    ..Default::default()
                };
                
                let result = camera.apply_controls(&normalized_controls);
                match result {
                    Ok(unsupported) => {
                        // Verify unsupported controls are reported
                        println!("Unsupported controls: {:?}", unsupported);
                    }
                    Err(e) => {
                        // Control application might fail if hardware unavailable
                        println!("Control application failed (expected): {:?}", e);
                    }
                }
                
                // Test extreme values don't crash
                let extreme_controls = CameraControls {
                    brightness: Some(-2.0),    // Beyond range
                    contrast: Some(5.0),       // Beyond range
                    saturation: Some(-1.0),    // Beyond range
                    ..Default::default()
                };
                
                let extreme_result = camera.apply_controls(&extreme_controls);
                // Should handle out-of-range values gracefully
                match extreme_result {
                    Ok(_) | Err(_) => {
                        // Both outcomes are acceptable - should not panic
                    }
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in range normalization test: {:?}", e),
        }
    }

    // Note: We can't easily test capture_frame without a real camera and without
    // major refactoring to support mocking. The existing function signature requires
    // a mutable reference to a real Camera object.
    // This would be a good candidate for dependency injection in future refactoring.
}
