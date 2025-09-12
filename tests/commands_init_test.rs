#[cfg(test)]
mod commands_init_tests {
    use crabcamera::commands::init::{
        check_camera_availability, get_available_cameras, get_camera_formats, get_current_platform,
        get_optimal_settings, get_platform_info, get_recommended_format, initialize_camera_system,
        test_camera_system,
    };

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
                assert!(
                    test_result.cameras_found >= 0,
                    "Camera count should be non-negative"
                );

                let platform_str = test_result.platform.as_str();
                assert!(!platform_str.is_empty(), "Platform should not be empty");

                // Test results should be a valid HashMap
                for (camera_id, test_result) in &test_result.test_results {
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

        // Should be a reasonable photography format
        assert!(
            format.width >= 1920,
            "Photography format should be high resolution"
        );
        assert!(
            format.height >= 1080,
            "Photography format should be high resolution"
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
            let inner_result = result.unwrap();
            assert!(
                inner_result.is_ok(),
                "Function calls should succeed or fail gracefully"
            );
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
}
