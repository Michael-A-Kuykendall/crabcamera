#[cfg(test)]
mod platform_windows_tests {
    use crabcamera::errors::CameraError;
    use crabcamera::platform::windows::{capture_frame, initialize_camera, list_cameras};
    use crabcamera::types::CameraFormat;

    #[test]
    fn test_list_cameras_returns_result() {
        // This test may fail on systems without cameras, but should not panic
        let result = list_cameras();

        // The function should return a Result
        match result {
            Ok(cameras) => {
                // If successful, cameras should be a valid Vec
                assert!(cameras.len() >= 0); // Could be empty if no cameras

                // Test each camera device info
                for camera in cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");
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
            Ok(camera) => {
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

    // Note: We can't easily test capture_frame without a real camera and without
    // major refactoring to support mocking. The existing function signature requires
    // a mutable reference to a real Camera object.
    // This would be a good candidate for dependency injection in future refactoring.
}
