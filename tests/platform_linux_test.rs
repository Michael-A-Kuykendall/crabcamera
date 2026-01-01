#![cfg(target_os = "linux")]

//! Comprehensive Linux platform-specific tests for CrabCamera
//!
//! Tests Video4Linux (V4L2) backend integration, Linux-specific camera features,
//! device enumeration, and platform-specific edge cases.

#[cfg(test)]
mod platform_linux_tests {
    use crabcamera::errors::CameraError;
    use crabcamera::platform::linux::{initialize_camera, list_cameras, utils, LinuxCamera};
    use crabcamera::types::{CameraFormat, CameraInitParams};
    use std::time::{Duration, Instant};

    /// Helper function to create test camera initialization parameters
    fn create_test_params(device_id: &str) -> CameraInitParams {
        CameraInitParams::new(device_id.to_string())
            .with_format(CameraFormat::new(640, 480, 30.0))
    }

    /// Helper function to check if V4L2 devices are available
    fn has_v4l2_devices() -> bool {
        utils::is_v4l2_available() && !utils::list_v4l2_devices().unwrap_or_default().is_empty()
    }

    #[test]
    fn test_linux_v4l2_availability_check() {
        // Test the fundamental V4L2 availability check
        let v4l2_available = utils::is_v4l2_available();
        
        if v4l2_available {
            // If V4L2 is available, /dev/video0 should exist
            assert!(
                std::path::Path::new("/dev/video0").exists(),
                "V4L2 available but /dev/video0 missing"
            );
        } else {
            // If V4L2 not available, we might be in a container or system without cameras
            println!("Warning: V4L2 not available on this Linux system");
        }
    }

    #[test]
    fn test_linux_v4l2_device_enumeration() {
        let devices_result = utils::list_v4l2_devices();
        
        match devices_result {
            Ok(devices) => {
                // Validate device paths
                for device_path in &devices {
                    assert!(
                        device_path.starts_with("/dev/video"),
                        "Device path should start with /dev/video: {}",
                        device_path
                    );
                    
                    // Check that path follows expected pattern
                    let device_number = device_path.strip_prefix("/dev/video");
                    if let Some(num_str) = device_number {
                        let parse_result: Result<u32, _> = num_str.parse();
                        assert!(
                            parse_result.is_ok(),
                            "Device number should be parseable: {}",
                            num_str
                        );
                        
                        let device_num = parse_result.unwrap();
                        assert!(
                            device_num < 100,
                            "Device number should be reasonable: {}",
                            device_num
                        );
                    }
                    
                    // If running with appropriate permissions, device should exist
                    if std::path::Path::new(device_path).exists() {
                        println!("Found V4L2 device: {}", device_path);
                    }
                }
                
                if devices.is_empty() {
                    println!("Warning: No V4L2 devices found");
                } else {
                    assert!(
                        devices.len() <= 20,
                        "Should not find an unreasonable number of devices"
                    );
                }
            }
            Err(e) => {
                panic!("Device enumeration should not fail: {:?}", e);
            }
        }
    }

    #[test]
    fn test_linux_device_capabilities_query() {
        let devices_result = utils::list_v4l2_devices();
        
        if let Ok(devices) = devices_result {
            for device_path in devices.iter().take(3) {  // Test first 3 devices
                let caps_result = utils::get_device_caps(device_path);
                
                match caps_result {
                    Ok(capabilities) => {
                        // Validate capability strings
                        for cap in &capabilities {
                            assert!(!cap.is_empty(), "Capability string should not be empty");
                            assert!(cap.len() > 3, "Capability should be descriptive");
                        }
                        
                        // Check for common V4L2 capabilities
                        let caps_joined = capabilities.join(" ").to_lowercase();
                        let has_video_cap = caps_joined.contains("video") || 
                                           caps_joined.contains("capture") ||
                                           caps_joined.contains("streaming");
                        
                        if !has_video_cap && !capabilities.is_empty() {
                            println!("Warning: No standard video capabilities found for {}: {:?}", 
                                   device_path, capabilities);
                        }
                    }
                    Err(e) => {
                        // Device capability query might fail if device is busy or inaccessible
                        println!("Could not query capabilities for {}: {:?}", device_path, e);
                    }
                }
            }
        }
    }

    #[test]
    fn test_linux_list_cameras_returns_valid_result() {
        let result = list_cameras();
        
        match result {
            Ok(cameras) => {
                // Validate each camera device info structure
                for camera in &cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");
                    
                    // Verify device ID is numeric (required for Linux implementation)
                    let parse_result: Result<u32, _> = camera.id.parse();
                    assert!(
                        parse_result.is_ok(),
                        "Camera ID should be numeric on Linux: {}",
                        camera.id
                    );
                    
                    // Check supported formats include Linux-specific formats
                    assert!(!camera.supports_formats.is_empty(), "Should have supported formats");
                    
                    // Verify common Linux V4L2 formats
                    let has_yuyv = camera.supports_formats.iter().any(|f| {
                        f.format_type.as_ref().map_or(false, |ft| ft == "YUYV")
                    });
                    let has_mjpeg = camera.supports_formats.iter().any(|f| {
                        f.format_type.as_ref().map_or(false, |ft| ft == "MJPEG")
                    });
                    
                    assert!(
                        has_yuyv || has_mjpeg,
                        "Linux camera should support YUYV or MJPEG formats"
                    );
                    
                    // Verify frame rates are reasonable for V4L2
                    for format in &camera.supports_formats {
                        assert!(format.fps > 0.0, "FPS should be positive");
                        assert!(format.fps <= 120.0, "FPS should be reasonable for Linux cameras");
                        
                        // Common V4L2 resolutions
                        let is_common_resolution = 
                            (format.width == 1920 && format.height == 1080) ||
                            (format.width == 1280 && format.height == 720) ||
                            (format.width == 640 && format.height == 480);
                        
                        if !is_common_resolution {
                            println!("Uncommon resolution found: {}x{}", format.width, format.height);
                        }
                    }
                }
                
                if cameras.is_empty() {
                    println!("Warning: No cameras found on Linux system");
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no cameras available or V4L2 access issues
            }
            Err(e) => panic!("Unexpected error type from list_cameras: {:?}", e),
        }
    }

    #[test]
    fn test_linux_camera_initialization_with_yuyv_format() {
        // Test YUYV format which is very common on Linux
        let yuyv_format = CameraFormat::new(640, 480, 30.0)
            .with_format_type("YUYV".to_string());
        let params = CameraInitParams::new("0".to_string()).with_format(yuyv_format.clone());
        
        let result = initialize_camera(params);
        
        match result {
            Ok(camera) => {
                // Verify camera was created successfully
                assert_eq!(camera.get_device_id(), "0");
                assert_eq!(camera.get_format().width, yuyv_format.width);
                assert_eq!(camera.get_format().height, yuyv_format.height);
                assert_eq!(camera.get_format().fps, yuyv_format.fps);
                
                // Test initial availability
                let available = camera.is_available();
                // Availability depends on stream state on Linux
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera or permissions insufficient
            }
            Err(e) => panic!("Unexpected error for YUYV format: {:?}", e),
        }
    }

    #[test]
    fn test_linux_camera_stream_lifecycle() {
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
                        assert!(streaming_available, "Camera should be available when streaming");
                        
                        // Test stopping stream
                        let stop_result = camera.stop_stream();
                        assert!(stop_result.is_ok(), "Stopping stream should succeed");
                        
                        // Test availability after stopping
                        let final_available = camera.is_available();
                        assert!(!final_available, "Camera should not be available after stopping stream");
                    }
                    Err(CameraError::InitializationError(_)) => {
                        // Expected if camera access denied or device busy
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
    fn test_linux_camera_capture_frame_functionality() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(camera) => {
                // Start stream first (required for V4L2)
                if camera.start_stream().is_ok() {
                    let capture_result = camera.capture_frame();
                    
                    match capture_result {
                        Ok(frame) => {
                            // Validate captured frame
                            assert!(frame.width > 0, "Frame width should be positive");
                            assert!(frame.height > 0, "Frame height should be positive");
                            assert!(!frame.data.is_empty(), "Frame data should not be empty");
                            assert_eq!(frame.device_id, "0");
                            
                            // Should be converted to RGB8 in our implementation
                            assert!(
                                frame.format.as_ref().map_or(true, |f| f == "RGB8"),
                                "Frame should be converted to RGB8"
                            );
                            
                            // Verify frame data size is reasonable
                            let expected_min_size = (frame.width * frame.height) as usize; // At least 1 byte per pixel
                            assert!(
                                frame.data.len() >= expected_min_size,
                                "Frame data size should be reasonable"
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
    fn test_linux_camera_invalid_device_ids() {
        let invalid_ids = vec![
            "invalid_string",
            "-1",
            "999",
            "",
            "abc123",
            "0x1",
            "1.5",
            " 0 ",  // Spaces
        ];
        
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
                    // Other errors might be acceptable depending on V4L2 behavior
                    println!("Got error for invalid device ID {}: {:?}", invalid_id, e);
                }
                Ok(_) => {
                    panic!("Invalid device ID {} should not work", invalid_id);
                }
            }
        }
    }

    #[test]
    fn test_linux_camera_supported_formats() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(camera) => {
                let formats_result = camera.get_supported_formats();
                
                match formats_result {
                    Ok(formats) => {
                        assert!(!formats.is_empty(), "Should have supported formats");
                        
                        // Verify common Linux formats are present
                        let has_yuyv = formats.iter().any(|f| {
                            f.format_type.as_ref().map_or(false, |ft| ft == "YUYV")
                        });
                        let has_mjpeg = formats.iter().any(|f| {
                            f.format_type.as_ref().map_or(false, |ft| ft == "MJPEG")
                        });
                        
                        assert!(
                            has_yuyv || has_mjpeg,
                            "Should support YUYV or MJPEG formats"
                        );
                        
                        // Check format validity
                        for format in &formats {
                            assert!(format.width > 0, "Width should be positive");
                            assert!(format.height > 0, "Height should be positive");
                            assert!(format.fps > 0.0, "FPS should be positive");
                            assert!(format.fps <= 120.0, "FPS should be reasonable");
                            
                            // Verify aspect ratio is reasonable
                            let aspect_ratio = format.width as f64 / format.height as f64;
                            assert!(
                                aspect_ratio > 0.5 && aspect_ratio < 5.0,
                                "Aspect ratio should be reasonable: {}",
                                aspect_ratio
                            );
                        }
                    }
                    Err(e) => panic!("Getting supported formats should not fail: {:?}", e),
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in supported formats test: {:?}", e),
        }
    }

    #[test]
    fn test_linux_v4l2_control_operations() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(camera) => {
                // Test setting individual V4L2 controls
                let control_tests = vec![
                    ("brightness", 50),
                    ("contrast", 75),
                    ("saturation", 60),
                    ("hue", 0),
                ];
                
                for (control_name, value) in control_tests {
                    let result = camera.set_control(control_name, value);
                    match result {
                        Ok(()) => {
                            println!("Successfully set {} to {}", control_name, value);
                        }
                        Err(CameraError::InitializationError(msg)) => {
                            // Expected for unsupported controls
                            assert!(
                                msg.contains("Unsupported control") || msg.contains(control_name),
                                "Error message should mention unsupported control: {}",
                                msg
                            );
                        }
                        Err(e) => panic!("Unexpected error setting control {}: {:?}", control_name, e),
                    }
                }
                
                // Test unsupported control
                let result = camera.set_control("nonexistent_control", 42);
                if let Err(CameraError::InitializationError(msg)) = result {
                    assert!(msg.contains("Unsupported control"));
                } else {
                    panic!("Setting unsupported control should fail");
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in V4L2 controls test: {:?}", e),
        }
    }

    #[test]
    fn test_linux_camera_controls_stub_implementation() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(mut camera) => {
                // Test getting default controls (stub implementation)
                let get_result = camera.get_controls();
                match get_result {
                    Ok(controls) => {
                        // Should return default controls (all None in stub)
                        assert!(controls.brightness.is_none(), "Stub should return None for brightness");
                        assert!(controls.contrast.is_none(), "Stub should return None for contrast");
                        assert!(controls.saturation.is_none(), "Stub should return None for saturation");
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
                assert!(apply_result.is_ok(), "Applying controls should succeed (stub)");
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in controls test: {:?}", e),
        }
    }

    #[test]
    fn test_linux_camera_capabilities_stub_implementation() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(camera) => {
                let capabilities_result = camera.test_capabilities();
                
                match capabilities_result {
                    Ok(capabilities) => {
                        // Should return default capabilities (stub implementation)
                        assert!(capabilities.max_resolution.0 > 0, "Max width should be positive");
                        assert!(capabilities.max_resolution.1 > 0, "Max height should be positive");
                        assert!(capabilities.max_fps > 0.0, "Max FPS should be positive");
                        
                        // Test all boolean capabilities exist
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
    fn test_linux_camera_performance_metrics_stub() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(camera) => {
                let metrics_result = camera.get_performance_metrics();
                
                match metrics_result {
                    Ok(metrics) => {
                        // Validate default performance metrics
                        assert!(metrics.capture_latency_ms >= 0.0, "Latency should be non-negative");
                        assert!(metrics.processing_time_ms >= 0.0, "Processing time should be non-negative");
                        assert!(metrics.memory_usage_mb >= 0.0, "Memory usage should be non-negative");
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
    fn test_linux_camera_thread_safety() {
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
                            let _ = camera.get_supported_formats();
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
    fn test_linux_camera_drop_cleanup() {
        let params = create_test_params("0");
        
        match initialize_camera(params) {
            Ok(camera) => {
                // Start stream to test cleanup
                let _ = camera.start_stream();
                
                // Camera should be properly cleaned up when dropped
                assert_eq!(camera.get_device_id(), "0");
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in drop test: {:?}", e),
        }
        // Camera is dropped here and should clean up stream properly
    }

    #[test]
    fn test_linux_multiple_format_support() {
        // Test various format combinations supported on Linux
        let test_formats = vec![
            CameraFormat::new(1920, 1080, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(640, 480, 30.0).with_format_type("YUYV".to_string()),
            CameraFormat::new(1920, 1080, 15.0).with_format_type("MJPEG".to_string()),
            CameraFormat::new(1280, 720, 30.0).with_format_type("MJPEG".to_string()),
        ];
        
        for format in test_formats {
            let params = CameraInitParams::new("0".to_string()).with_format(format.clone());
            let result = initialize_camera(params);
            
            match result {
                Ok(camera) => {
                    // Verify camera was created with expected format
                    assert_eq!(camera.get_format().width, format.width);
                    assert_eq!(camera.get_format().height, format.height);
                    assert_eq!(camera.get_format().fps, format.fps);
                    
                    // Verify format type is preserved
                    if let Some(expected_type) = &format.format_type {
                        // Format might be converted internally, but should be tracked
                        println!("Testing format type: {}", expected_type);
                    }
                }
                Err(CameraError::InitializationError(_)) => {
                    // Expected if camera or format not supported
                }
                Err(e) => panic!("Unexpected error for format {:?}: {:?}", format, e),
            }
        }
    }

    #[test]
    fn test_linux_error_message_quality() {
        // Test that error messages are informative for debugging
        let invalid_params = CameraInitParams::new("invalid".to_string())
            .with_format(CameraFormat::new(0, 0, 0.0));
        
        let result = initialize_camera(invalid_params);
        
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
    fn test_linux_camera_state_consistency() {
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
                assert_eq!(format1.height, format2.height, "Format should be consistent");
                assert_eq!(format1.fps, format2.fps, "Format should be consistent");
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no camera available
            }
            Err(e) => panic!("Unexpected error in state consistency test: {:?}", e),
        }
    }

    #[test]
    fn test_linux_v4l2_backend_specific_behavior() {
        // Test Linux/V4L2 specific behaviors
        let result = list_cameras();
        
        match result {
            Ok(cameras) => {
                for camera in cameras {
                    // V4L2 devices should have numeric IDs
                    let parse_result: Result<u32, _> = camera.id.parse();
                    assert!(
                        parse_result.is_ok(),
                        "V4L2 camera ID should be numeric: {}",
                        camera.id
                    );
                    
                    // Should support V4L2-specific formats
                    let formats = &camera.supports_formats;
                    let has_linux_format = formats.iter().any(|f| {
                        f.format_type.as_ref().map_or(false, |ft| {
                            ft == "YUYV" || ft == "MJPEG"
                        })
                    });
                    
                    if !has_linux_format && !formats.is_empty() {
                        println!("Warning: No typical Linux formats found for camera {}", camera.id);
                    }
                }
            }
            Err(CameraError::InitializationError(_)) => {
                // Expected if no V4L2 devices
            }
            Err(e) => panic!("Unexpected error testing V4L2 backend: {:?}", e),
        }
    }
}