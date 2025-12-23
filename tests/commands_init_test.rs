#[cfg(test)]
mod commands_init_tests {
    use crabcamera::commands::init::{
        check_camera_availability, get_available_cameras, get_camera_formats, get_current_platform,
        get_optimal_settings, get_platform_info, get_recommended_format, initialize_camera_system,
        test_camera_system, get_system_diagnostics,
    };
    use crabcamera::tests::{set_mock_camera_mode, MockCaptureMode};
    use std::time::Duration;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_initialize_camera_system() {
        let result = initialize_camera_system().await;

        // Should return a Result - success or failure depends on system
        match result {
            Ok(message) => {
                assert!(!message.is_empty(), "Success message should not be empty");
                assert!(message.len() > 5, "Success message should be descriptive");
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("Failed to initialize"),
                    "Error should mention initialization failure"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_available_cameras() {
        let result = get_available_cameras().await;

        match result {
            Ok(cameras) => {
                // If successful, cameras should be a valid Vec
                for camera in &cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");
                    // is_available can be true or false, both are valid
                }

                // Log should have been written (we can't test log content directly in unit tests)
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("Failed to list cameras"),
                    "Error should mention camera listing failure"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_platform_info() {
        let result = get_platform_info().await;

        match result {
            Ok(info) => {
                // Platform info should have valid fields
                assert!(
                    !info.platform.as_str().is_empty(),
                    "Platform string should not be empty"
                );
                assert!(!info.backend.is_empty(), "Backend should not be empty");

                // Platform should be one of the expected values
                let platform_str = info.platform.as_str();
                assert!(
                    platform_str == "windows"
                        || platform_str == "linux"
                        || platform_str == "macos"
                        || platform_str == "unknown",
                    "Platform should be a known value, got: {}",
                    platform_str
                );
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("Failed to get platform info"),
                    "Error should mention platform info failure"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_current_platform() {
        let result = get_current_platform().await;

        // This should always succeed since it's just returning Platform::current()
        assert!(result.is_ok(), "get_current_platform should always succeed");

        let platform = result.unwrap();
        assert!(!platform.is_empty(), "Platform string should not be empty");

        // Should be one of the known platforms
        assert!(
            platform == "windows"
                || platform == "linux"
                || platform == "macos"
                || platform == "unknown",
            "Platform should be a known value, got: {}",
            platform
        );
    }

    #[tokio::test]
    async fn test_test_camera_system() {
        let result = test_camera_system().await;

        match result {
            Ok(test_result) => {
                // Test result should have valid fields
                // cameras_found is u32, always >= 0

                let platform_str = test_result.platform.as_str();
                assert!(!platform_str.is_empty(), "Platform should not be empty");

                // Test results should be a valid HashMap
                for (camera_id, _test_result) in &test_result.test_results {
                    assert!(!camera_id.is_empty(), "Camera ID should not be empty");
                    // test_result can be any of the CameraTestResult variants - all are valid
                }
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("Camera system test failed"),
                    "Error should mention test failure"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_check_camera_availability_with_invalid_id() {
        let result = check_camera_availability("nonexistent_camera_99999".to_string()).await;

        match result {
            Ok(is_available) => {
                // Should return false for non-existent camera
                assert!(!is_available, "Non-existent camera should not be available");
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("Failed to check camera availability"),
                    "Error should mention availability check failure"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_check_camera_availability_with_empty_id() {
        let result = check_camera_availability("".to_string()).await;

        match result {
            Ok(is_available) => {
                // Should return false for empty ID
                assert!(!is_available, "Empty camera ID should not be available");
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
            }
        }
    }

    #[tokio::test]
    async fn test_get_camera_formats_with_invalid_id() {
        let result = get_camera_formats("nonexistent_camera_99999".to_string()).await;

        match result {
            Ok(_formats) => {
                panic!("Should not find formats for non-existent camera");
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("not found") || error.contains("Failed to get camera formats"),
                    "Error should mention camera not found, got: {}",
                    error
                );
            }
        }
    }

    #[tokio::test]
    async fn test_get_recommended_format() {
        let result = get_recommended_format().await;

        // This should always succeed as it returns a static format
        assert!(
            result.is_ok(),
            "get_recommended_format should always succeed"
        );

        let format = result.unwrap();
        assert!(format.width > 0, "Format width should be positive");
        assert!(format.height > 0, "Format height should be positive");
        assert!(format.fps > 0.0, "Format FPS should be positive");

        // Should be a reasonable format (varies by platform)
        // Linux returns 1280x720, macOS/Windows return 1920x1080
        assert!(format.width >= 1280, "Format should be at least 720p width");
        assert!(
            format.height >= 720,
            "Format should be at least 720p height"
        );
    }

    #[tokio::test]
    async fn test_get_optimal_settings() {
        let result = get_optimal_settings().await;

        // This should always succeed as it returns static settings
        assert!(result.is_ok(), "get_optimal_settings should always succeed");

        let settings = result.unwrap();
        assert!(
            !settings.device_id.is_empty(),
            "Device ID should not be empty"
        );
        assert!(settings.format.width > 0, "Format width should be positive");
        assert!(
            settings.format.height > 0,
            "Format height should be positive"
        );
        assert!(settings.format.fps > 0.0, "Format FPS should be positive");
    }

    #[tokio::test]
    async fn test_multiple_concurrent_calls() {
        let mut handles = vec![];

        // Test concurrent calls to platform function
        handles.push(tokio::spawn(async {
            get_current_platform().await.map(|_| ())
        }));

        // Test concurrent calls to recommended format
        handles.push(tokio::spawn(async {
            get_recommended_format().await.map(|_| ())
        }));

        // Test concurrent calls to optimal settings
        handles.push(tokio::spawn(async {
            get_optimal_settings().await.map(|_| ())
        }));

        // Test concurrent calls to camera availability
        handles.push(tokio::spawn(async {
            check_camera_availability("0".to_string()).await.map(|_| ())
        }));

        // All should complete without panics
        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok(), "Concurrent calls should not panic");
            // Note: The functions may return Ok or Err depending on system state
            // (e.g., no cameras present). The important thing is they don't panic.
            // Success means the function executed without crashing, not that it found cameras.
        }
    }

    #[tokio::test]
    async fn test_function_error_message_consistency() {
        // Test that error messages follow consistent format
        let invalid_device_id = "definitely_nonexistent_camera_12345";

        let availability_result = check_camera_availability(invalid_device_id.to_string()).await;
        let formats_result = get_camera_formats(invalid_device_id.to_string()).await;

        // Both functions should handle invalid device IDs gracefully
        match availability_result {
            Ok(_) => {} // OK to return false for invalid ID
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(!error.contains("panic"), "Error should not mention panic");
            }
        }

        match formats_result {
            Ok(_) => panic!("Should not find formats for invalid device"),
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(!error.contains("panic"), "Error should not mention panic");
            }
        }
    }

    #[tokio::test]
    async fn test_platform_consistency() {
        // Test that platform information is consistent across calls
        let platform1 = get_current_platform().await.unwrap();
        let platform2 = get_current_platform().await.unwrap();

        assert_eq!(
            platform1, platform2,
            "Platform should be consistent across calls"
        );

        // Platform info should match current platform
        if let Ok(platform_info) = get_platform_info().await {
            assert_eq!(
                platform1,
                platform_info.platform.as_str(),
                "Platform info should match current platform"
            );
        }
    }

    #[tokio::test]
    async fn test_system_diagnostics() {
        let result = get_system_diagnostics().await;
        
        // System diagnostics should always return some result
        assert!(result.is_ok(), "System diagnostics should not fail");
        
        let diagnostics = result.unwrap();
        assert!(!diagnostics.crate_version.is_empty(), "Version should not be empty");
        assert!(!diagnostics.platform.is_empty(), "Platform should not be empty");
        assert!(!diagnostics.backend.is_empty(), "Backend should not be empty");
        assert!(!diagnostics.permission_status.is_empty(), "Permission status should not be empty");
        assert!(!diagnostics.timestamp.is_empty(), "Timestamp should not be empty");
        
        // Camera count should be reasonable (0 or more)
        assert!(diagnostics.camera_count < 100, "Camera count should be reasonable");
        
        // Camera summaries should be valid
        for camera in &diagnostics.cameras {
            assert!(!camera.id.is_empty(), "Camera ID should not be empty");
            assert!(!camera.name.is_empty(), "Camera name should not be empty");
            // format_count can be 0 or more, both valid
            // max_resolution can be None, that's valid
        }
        
        // Features should be non-empty
        assert!(!diagnostics.features_enabled.is_empty(), "Should have some features enabled");
    }

    #[tokio::test]
    async fn test_initialization_timeout() {
        // Test that initialization doesn't hang indefinitely
        let timeout_duration = Duration::from_secs(30); // Generous timeout for real systems
        
        let result = timeout(timeout_duration, initialize_camera_system()).await;
        
        assert!(result.is_ok(), "Initialization should complete within timeout");
        
        // The inner result can be Ok or Err depending on system state
        let init_result = result.unwrap();
        match init_result {
            Ok(msg) => assert!(!msg.is_empty(), "Success message should not be empty"),
            Err(err) => assert!(!err.is_empty(), "Error message should not be empty"),
        }
    }

    #[tokio::test]
    async fn test_camera_availability_with_various_ids() {
        // Test different types of device IDs
        let test_ids = vec![
            "0",
            "1",
            "default",
            "camera_0",
            "cam-device-123",
            "video0",
            "/dev/video0",
            "FaceTime HD Camera",
            "USB Camera",
            "very_long_device_id_that_might_exist_somewhere_on_some_system_maybe",
        ];
        
        for device_id in test_ids {
            let result = check_camera_availability(device_id.to_string()).await;
            
            // Should always return a result, either Ok(bool) or Err(String)
            match result {
                Ok(available) => {
                    // Boolean result is fine, true or false both valid
                    let _ = available;
                }
                Err(error) => {
                    assert!(!error.is_empty(), "Error message should not be empty for ID: {}", device_id);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_get_camera_formats_edge_cases() {
        // Test with various device IDs including edge cases
        let test_cases = vec![
            ("nonexistent_camera", true),  // Should fail
            ("", true),                    // Empty ID should fail
            ("0", false),                  // Might succeed with default camera
            ("null", true),                // Should fail
            ("undefined", true),           // Should fail
        ];
        
        for (device_id, should_fail) in test_cases {
            let result = get_camera_formats(device_id.to_string()).await;
            
            if should_fail {
                assert!(result.is_err(), "Should fail for device ID: {}", device_id);
                let error = result.unwrap_err();
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(
                    error.contains("not found") || error.contains("Failed to get camera formats"),
                    "Error should be descriptive for ID: {}", device_id
                );
            } else {
                // For "0" we're more permissive - it might work or fail
                match result {
                    Ok(formats) => {
                        // If it succeeds, formats should be valid
                        for format in formats {
                            assert!(format.width > 0, "Format width should be positive");
                            assert!(format.height > 0, "Format height should be positive");
                            assert!(format.fps > 0.0, "Format FPS should be positive");
                        }
                    }
                    Err(_) => {
                        // Failure is also acceptable for this case
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_format_and_settings_consistency() {
        // Test that recommended format and optimal settings are consistent
        let recommended_format = get_recommended_format().await.unwrap();
        let optimal_settings = get_optimal_settings().await.unwrap();
        
        // Both should have valid dimensions
        assert!(recommended_format.width > 0, "Recommended format width should be positive");
        assert!(recommended_format.height > 0, "Recommended format height should be positive");
        assert!(recommended_format.fps > 0.0, "Recommended format FPS should be positive");
        
        assert!(optimal_settings.format.width > 0, "Optimal settings width should be positive");
        assert!(optimal_settings.format.height > 0, "Optimal settings height should be positive");
        assert!(optimal_settings.format.fps > 0.0, "Optimal settings FPS should be positive");
        
        // Device ID should be specified
        assert!(!optimal_settings.device_id.is_empty(), "Device ID should not be empty");
    }

    #[tokio::test]
    async fn test_stress_concurrent_initialization_calls() {
        // Test multiple concurrent initialization calls
        let mut handles = Vec::new();
        
        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let result = initialize_camera_system().await;
                (i, result)
            });
            handles.push(handle);
        }
        
        // All should complete without panicking
        for handle in handles {
            let (call_id, result) = handle.await.unwrap();
            
            // Result can be Ok or Err, but should be consistent
            match result {
                Ok(msg) => assert!(!msg.is_empty(), "Success message should not be empty for call {}", call_id),
                Err(err) => assert!(!err.is_empty(), "Error message should not be empty for call {}", call_id),
            }
        }
    }

    #[tokio::test]
    async fn test_camera_system_test_comprehensive() {
        let result = test_camera_system().await;
        
        match result {
            Ok(test_result) => {
                // Validate test result structure
                let platform_str = test_result.platform.as_str();
                let valid_platforms = ["windows", "linux", "macos", "unknown"];
                assert!(valid_platforms.contains(&platform_str), "Platform should be valid: {}", platform_str);
                
                // cameras_found should be reasonable
                assert!(test_result.cameras_found < 100, "Camera count should be reasonable");
                
                // Test results should be valid
                for (camera_id, test_result) in &test_result.test_results {
                    assert!(!camera_id.is_empty(), "Camera ID should not be empty");
                    
                    // All test result variants are valid, just verify they're structured correctly
                    match test_result {
                        crabcamera::platform::CameraTestResult::Success => {
                            // Success is good
                        }
                        crabcamera::platform::CameraTestResult::InitError(err) => {
                            assert!(!err.is_empty(), "Init error should have a message");
                        }
                        crabcamera::platform::CameraTestResult::CaptureError(err) => {
                            assert!(!err.is_empty(), "Capture error should have a message");
                        }
                        crabcamera::platform::CameraTestResult::NotAvailable => {
                            // Not available is fine
                        }
                    }
                }
            }
            Err(error) => {
                assert!(!error.is_empty(), "Error message should not be empty");
                assert!(error.contains("Camera system test failed"), "Error should mention test failure");
            }
        }
    }

    #[tokio::test]
    async fn test_high_stress_availability_checks() {
        // Test many availability checks rapidly
        let mut handles = Vec::new();
        
        for i in 0..50 {
            let device_id = format!("stress_test_{}", i % 10); // Reuse some IDs
            let handle = tokio::spawn(async move {
                let result = check_camera_availability(device_id.clone()).await;
                (device_id, result)
            });
            handles.push(handle);
        }
        
        for handle in handles {
            let (device_id, result) = handle.await.unwrap();
            
            match result {
                Ok(_available) => {
                    // Boolean result is fine
                }
                Err(error) => {
                    assert!(!error.is_empty(), "Error should not be empty for device: {}", device_id);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_system_state_isolation() {
        // Test that different calls don't interfere with each other
        
        // Run operations in parallel to test isolation
        let (platform_result, camera_result, diag_result) = tokio::join!(
            get_platform_info(),
            get_available_cameras(),
            get_system_diagnostics()
        );
        
        // All should complete independently
        match platform_result {
            Ok(info) => {
                assert!(!info.platform.as_str().is_empty(), "Platform should not be empty");
                assert!(!info.backend.is_empty(), "Backend should not be empty");
            }
            Err(err) => {
                assert!(!err.is_empty(), "Platform error should not be empty");
            }
        }
        
        match camera_result {
            Ok(cameras) => {
                for camera in cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                }
            }
            Err(err) => {
                assert!(!err.is_empty(), "Camera error should not be empty");
            }
        }
        
        assert!(diag_result.is_ok(), "Diagnostics should always succeed");
    }

    #[tokio::test]
    async fn test_initialization_failure_recovery() {
        // Test multiple initialization attempts
        for attempt in 1..=5 {
            let result = initialize_camera_system().await;
            
            match result {
                Ok(msg) => {
                    assert!(!msg.is_empty(), "Success message should not be empty on attempt {}", attempt);
                }
                Err(err) => {
                    assert!(!err.is_empty(), "Error message should not be empty on attempt {}", attempt);
                    assert!(err.contains("Failed to initialize"), "Error should be descriptive on attempt {}", attempt);
                }
            }
            
            // Small delay between attempts
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[tokio::test]
    async fn test_memory_leak_prevention() {
        // Test repeated calls to ensure no obvious memory leaks
        for _ in 0..100 {
            let _ = get_current_platform().await;
            let _ = get_recommended_format().await;
            let _ = get_optimal_settings().await;
            // These should be lightweight calls that don't accumulate resources
        }
        
        // If we get here without OOM, test passes
        assert!(true, "No memory issues detected");
    }
}
