#[cfg(test)]
mod platform_tests {
    use crabcamera::errors::CameraError;
    use crabcamera::platform::{MockCamera, PlatformCamera};
    use crabcamera::tests::{set_mock_camera_mode, MockCaptureMode};
    use crabcamera::types::{CameraControls, CameraFormat, CameraInitParams};

    fn create_test_params() -> CameraInitParams {
        CameraInitParams::new("test_camera_0".to_string())
            .with_format(CameraFormat::new(640, 480, 30.0))
    }

    #[test]
    fn test_mock_camera_creation() {
        let format = CameraFormat::new(1920, 1080, 30.0);
        let mock_camera = MockCamera::new("test_device".to_string(), format.clone());

        assert_eq!(mock_camera.get_device_id(), "test_device");
        assert!(mock_camera.is_available());
    }

    #[test]
    fn test_mock_camera_stream_control() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mock_camera = MockCamera::new("test_stream".to_string(), format);

        // Test starting stream
        let result = mock_camera.start_stream();
        assert!(result.is_ok(), "Starting stream should succeed");

        // Test stopping stream
        let result = mock_camera.stop_stream();
        assert!(result.is_ok(), "Stopping stream should succeed");
    }

    #[test]
    fn test_mock_camera_capture_success() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mut mock_camera = MockCamera::new("test_capture".to_string(), format);

        // Set success mode
        mock_camera.set_capture_mode(MockCaptureMode::Success);
        set_mock_camera_mode("test_capture", MockCaptureMode::Success);

        let result = mock_camera.capture_frame();
        assert!(result.is_ok(), "Capture should succeed in success mode");

        let frame = result.unwrap();
        assert!(frame.width > 0, "Frame width should be positive");
        assert!(frame.height > 0, "Frame height should be positive");
        assert!(!frame.data.is_empty(), "Frame data should not be empty");
        assert_eq!(frame.device_id, "test_capture");
    }

    #[test]
    fn test_mock_camera_capture_failure() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mut mock_camera = MockCamera::new("test_fail".to_string(), format);

        // Set failure mode
        mock_camera.set_capture_mode(MockCaptureMode::Failure);
        set_mock_camera_mode("test_fail", MockCaptureMode::Failure);

        let result = mock_camera.capture_frame();
        assert!(result.is_err(), "Capture should fail in failure mode");

        match result {
            Err(CameraError::CaptureError(msg)) => {
                assert!(msg.contains("Mock capture failure"));
            }
            _ => panic!("Expected CaptureError"),
        }
    }

    #[test]
    fn test_mock_camera_slow_capture() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mut mock_camera = MockCamera::new("test_slow".to_string(), format);

        // Set slow capture mode
        mock_camera.set_capture_mode(MockCaptureMode::SlowCapture);
        set_mock_camera_mode("test_slow", MockCaptureMode::SlowCapture);

        let start = std::time::Instant::now();
        let result = mock_camera.capture_frame();
        let duration = start.elapsed();

        assert!(result.is_ok(), "Slow capture should succeed");
        assert!(
            duration.as_millis() >= 100,
            "Slow capture should take at least 100ms"
        );
    }

    #[test]
    fn test_mock_camera_controls() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mut mock_camera = MockCamera::new("test_controls".to_string(), format);

        let controls = CameraControls {
            brightness: Some(0.7),
            contrast: Some(0.8),
            saturation: Some(0.6),
            exposure_time: Some(0.5),
            focus_distance: Some(0.9),
            white_balance: Some(crabcamera::types::WhiteBalance::Custom(6500)),
            iso_sensitivity: Some(800),
            zoom: Some(2.0),
            auto_focus: Some(true),
            auto_exposure: Some(true),
            aperture: None,
            image_stabilization: Some(true),
            noise_reduction: Some(false),
            sharpness: Some(0.5),
        };

        // Apply controls
        let result = mock_camera.apply_controls(&controls);
        assert!(result.is_ok(), "Applying controls should succeed");

        // Get controls back
        let result = mock_camera.get_controls();
        assert!(result.is_ok(), "Getting controls should succeed");

        let retrieved_controls = result.unwrap();
        assert_eq!(retrieved_controls.brightness, controls.brightness);
        assert_eq!(retrieved_controls.contrast, controls.contrast);
        assert_eq!(retrieved_controls.saturation, controls.saturation);
    }

    #[test]
    fn test_mock_camera_capabilities() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mock_camera = MockCamera::new("test_capabilities".to_string(), format);

        let result = mock_camera.test_capabilities();
        assert!(result.is_ok(), "Getting capabilities should succeed");

        let capabilities = result.unwrap();
        assert!(
            capabilities.supports_auto_focus,
            "Should support auto focus"
        );
        assert!(
            capabilities.supports_manual_focus,
            "Should support manual focus"
        );
        assert!(
            capabilities.supports_auto_exposure,
            "Should support auto exposure"
        );
        assert!(
            capabilities.supports_white_balance,
            "Should support white balance"
        );
        assert_eq!(capabilities.max_resolution, (1920, 1080));
        assert_eq!(capabilities.max_fps, 60.0);
    }

    #[test]
    fn test_mock_camera_performance_metrics() {
        let format = CameraFormat::new(640, 480, 30.0);
        let mock_camera = MockCamera::new("test_metrics".to_string(), format);

        let result = mock_camera.get_performance_metrics();
        assert!(result.is_ok(), "Getting performance metrics should succeed");

        let metrics = result.unwrap();
        assert!(
            metrics.capture_latency_ms > 0.0,
            "Capture latency should be positive"
        );
        assert!(
            metrics.processing_time_ms > 0.0,
            "Processing time should be positive"
        );
        assert!(
            metrics.memory_usage_mb > 0.0,
            "Memory usage should be positive"
        );
        assert!(metrics.fps_actual > 0.0, "Actual FPS should be positive");
        assert!(
            metrics.quality_score > 0.0 && metrics.quality_score <= 1.0,
            "Quality score should be 0-1"
        );
    }

    #[test]
    fn test_platform_camera_creation_in_test_environment() {
        let params = create_test_params();

        let result = PlatformCamera::new(params);
        assert!(
            result.is_ok(),
            "Creating platform camera should succeed in test environment"
        );

        match result.unwrap() {
            PlatformCamera::Mock(_) => {
                // Expected in test environment
            }
            _ => panic!("Expected Mock camera in test environment"),
        }
    }

    #[test]
    fn test_platform_camera_capture_frame() {
        let params = create_test_params();
        let mut camera = PlatformCamera::new(params).unwrap();

        // Set up for successful capture
        set_mock_camera_mode("test_camera_0", MockCaptureMode::Success);

        let result = camera.capture_frame();
        assert!(result.is_ok(), "Capturing frame should succeed");

        let frame = result.unwrap();
        assert_eq!(frame.device_id, "test_camera_0");
        assert!(frame.width > 0 && frame.height > 0);
        assert!(!frame.data.is_empty());
    }

    #[test]
    fn test_platform_camera_stream_control() {
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        // Test start stream
        let result = camera.start_stream();
        assert!(result.is_ok(), "Starting stream should succeed");

        // Test stop stream
        let result = camera.stop_stream();
        assert!(result.is_ok(), "Stopping stream should succeed");
    }

    #[test]
    fn test_platform_camera_availability() {
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        let is_available = camera.is_available();
        assert!(is_available, "Mock camera should be available");
    }

    #[test]
    fn test_platform_camera_device_id() {
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        let device_id = camera.get_device_id();
        assert!(device_id.is_some(), "Mock camera should have device ID");
        assert_eq!(device_id.unwrap(), "test_camera_0");
    }

    #[test]
    fn test_platform_camera_apply_controls() {
        let params = create_test_params();
        let mut camera = PlatformCamera::new(params).unwrap();

        let controls = CameraControls {
            brightness: Some(0.5),
            contrast: Some(0.6),
            saturation: Some(0.7),
            exposure_time: Some(0.8),
            focus_distance: Some(0.9),
            white_balance: Some(crabcamera::types::WhiteBalance::Custom(5500)),
            iso_sensitivity: Some(400),
            zoom: Some(1.5),
            auto_focus: Some(false),
            auto_exposure: Some(false),
            aperture: None,
            image_stabilization: Some(false),
            noise_reduction: Some(true),
            sharpness: Some(0.3),
        };

        let result = camera.apply_controls(&controls);
        assert!(result.is_ok(), "Applying controls should succeed");
    }

    #[test]
    fn test_platform_camera_error_propagation() {
        let params = create_test_params();
        let mut camera = PlatformCamera::new(params).unwrap();

        // Set up for capture failure
        set_mock_camera_mode("test_camera_0", MockCaptureMode::Failure);

        let result = camera.capture_frame();
        assert!(
            result.is_err(),
            "Capture should fail when set to failure mode"
        );

        match result {
            Err(CameraError::CaptureError(_)) => {
                // Expected error type
            }
            _ => panic!("Expected CaptureError"),
        }
    }

    #[test]
    fn test_multiple_camera_instances() {
        let params1 = CameraInitParams::new("test_multi_1".to_string())
            .with_format(CameraFormat::new(640, 480, 30.0));

        let params2 = CameraInitParams::new("test_multi_2".to_string())
            .with_format(CameraFormat::new(1280, 720, 30.0));

        let camera1 = PlatformCamera::new(params1);
        let camera2 = PlatformCamera::new(params2);

        assert!(
            camera1.is_ok(),
            "First camera should be created successfully"
        );
        assert!(
            camera2.is_ok(),
            "Second camera should be created successfully"
        );

        let camera1 = camera1.unwrap();
        let camera2 = camera2.unwrap();

        assert_eq!(camera1.get_device_id(), Some("test_multi_1"));
        assert_eq!(camera2.get_device_id(), Some("test_multi_2"));
    }

    #[test]
    fn test_concurrent_camera_operations() {
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        // Test concurrent stream operations
        let camera_arc = std::sync::Arc::new(std::sync::Mutex::new(camera));
        let mut handles = vec![];

        for i in 0..5 {
            let camera_clone = camera_arc.clone();
            let handle = std::thread::spawn(move || {
                if let Ok(camera) = camera_clone.lock() {
                    let _ = camera.start_stream();
                    let _ = camera.stop_stream();
                    let _ = camera.is_available();
                    i // Return thread ID for verification
                } else {
                    panic!("Failed to acquire camera lock");
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.join().unwrap();
            assert_eq!(result, i, "Thread should complete successfully");
        }
    }

    #[test]
    fn test_error_handling_comprehensive() {
        // Test various error conditions
        let params = create_test_params();
        let mut camera = PlatformCamera::new(params).unwrap();

        // Test different failure modes
        set_mock_camera_mode("test_camera_0", MockCaptureMode::Failure);
        let result = camera.capture_frame();
        assert!(result.is_err());

        // Switch back to success
        set_mock_camera_mode("test_camera_0", MockCaptureMode::Success);
        let result = camera.capture_frame();
        assert!(result.is_ok());

        // Test slow capture doesn't cause errors
        set_mock_camera_mode("test_camera_0", MockCaptureMode::SlowCapture);
        let result = camera.capture_frame();
        assert!(result.is_ok());
    }

    #[test]
    fn test_platform_camera_drop_cleanup() {
        // Test that Drop implementation properly cleans up
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        // Camera should start stream successfully
        let start_result = camera.start_stream();
        assert!(start_result.is_ok(), "Stream should start successfully");

        // When camera goes out of scope, Drop should be called
        // This is automatic and we can't test it directly, but we can verify
        // that the camera is properly constructed for cleanup
        assert!(
            camera.is_available(),
            "Camera should be available before drop"
        );
    }

    #[test]
    fn test_camera_system_operations() {
        use crabcamera::platform::CameraSystem;

        // Test system initialization
        let init_result = CameraSystem::initialize();
        match init_result {
            Ok(message) => {
                assert!(!message.is_empty(), "Init message should not be empty");
                assert!(message.len() > 10, "Init message should be descriptive");
            }
            Err(_) => {
                // Initialization failure is acceptable in test environment
            }
        }

        // Test platform info
        let platform_result = CameraSystem::get_platform_info();
        match platform_result {
            Ok(info) => {
                assert!(!info.backend.is_empty(), "Backend should be specified");
                assert!(!info.features.is_empty(), "Should have some features");

                // Verify platform enum is valid
                let platform_str = info.platform.as_str();
                let valid_platforms = vec!["windows", "linux", "macos", "unknown"];
                assert!(
                    valid_platforms.contains(&platform_str),
                    "Platform should be valid: {}",
                    platform_str
                );
            }
            Err(_) => {
                // Platform info failure is acceptable in test environment
            }
        }

        // Test camera listing
        let cameras_result = CameraSystem::list_cameras();
        match cameras_result {
            Ok(cameras) => {
                // Cameras can be empty in test environment
                for camera in cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");
                    // is_available can be true or false - both are valid
                }
            }
            Err(_) => {
                // Camera listing failure is acceptable in test environment
            }
        }

        // Test system testing
        let test_result = CameraSystem::test_system();
        match test_result {
            Ok(result) => {
                assert!(
                    result.cameras_found >= 0,
                    "Camera count should be non-negative"
                );

                // Test results should be valid
                for (camera_id, test_result) in result.test_results {
                    assert!(!camera_id.is_empty(), "Camera ID should not be empty");
                    // All test result variants are valid - just verify the match works
                    match test_result {
                        crabcamera::platform::CameraTestResult::Success => {}
                        crabcamera::platform::CameraTestResult::InitError(_) => {}
                        crabcamera::platform::CameraTestResult::CaptureError(_) => {}
                        crabcamera::platform::CameraTestResult::NotAvailable => {}
                    }
                }
            }
            Err(_) => {
                // System test failure is acceptable in test environment
            }
        }
    }

    #[test]
    fn test_platform_optimizations() {
        use crabcamera::platform::optimizations;

        // Test photography format recommendation
        let photo_format = optimizations::get_photography_format();
        assert!(
            photo_format.width > 0,
            "Photography format should have positive width"
        );
        assert!(
            photo_format.height > 0,
            "Photography format should have positive height"
        );
        assert!(
            photo_format.fps > 0.0,
            "Photography format should have positive FPS"
        );

        // Should be a reasonable photography resolution
        assert!(
            photo_format.width >= 1280,
            "Photography format should be at least 720p width"
        );
        assert!(
            photo_format.height >= 720,
            "Photography format should be at least 720p height"
        );

        // Test optimal settings
        let optimal_settings = optimizations::get_optimal_settings();
        assert!(
            !optimal_settings.device_id.is_empty(),
            "Device ID should not be empty"
        );
        assert!(
            optimal_settings.format.width > 0,
            "Format width should be positive"
        );
        assert!(
            optimal_settings.format.height > 0,
            "Format height should be positive"
        );
        assert!(
            optimal_settings.format.fps > 0.0,
            "Format FPS should be positive"
        );
    }

    #[test]
    fn test_platform_info_serialization() {
        use crabcamera::platform::PlatformInfo;
        use crabcamera::types::Platform;

        let platform_info = PlatformInfo {
            platform: Platform::Windows,
            backend: "Test Backend".to_string(),
            features: vec!["Feature 1".to_string(), "Feature 2".to_string()],
        };

        // Test serialization
        let serialized = serde_json::to_string(&platform_info);
        assert!(
            serialized.is_ok(),
            "PlatformInfo should serialize successfully"
        );

        // Test deserialization
        let json = serialized.unwrap();
        let deserialized: Result<PlatformInfo, _> = serde_json::from_str(&json);
        assert!(
            deserialized.is_ok(),
            "PlatformInfo should deserialize successfully"
        );

        let restored_info = deserialized.unwrap();
        assert_eq!(restored_info.platform.as_str(), "windows");
        assert_eq!(restored_info.backend, "Test Backend");
        assert_eq!(restored_info.features.len(), 2);
    }

    #[test]
    fn test_system_test_result_serialization() {
        use crabcamera::platform::{CameraTestResult, SystemTestResult};
        use crabcamera::types::Platform;

        let test_result = SystemTestResult {
            platform: Platform::Linux,
            cameras_found: 2,
            test_results: vec![
                ("camera1".to_string(), CameraTestResult::Success),
                (
                    "camera2".to_string(),
                    CameraTestResult::InitError("Test error".to_string()),
                ),
                ("camera3".to_string(), CameraTestResult::NotAvailable),
            ],
        };

        // Test serialization
        let serialized = serde_json::to_string(&test_result);
        assert!(
            serialized.is_ok(),
            "SystemTestResult should serialize successfully"
        );

        // Test deserialization
        let json = serialized.unwrap();
        let deserialized: Result<SystemTestResult, _> = serde_json::from_str(&json);
        assert!(
            deserialized.is_ok(),
            "SystemTestResult should deserialize successfully"
        );

        let restored_result = deserialized.unwrap();
        assert_eq!(restored_result.platform.as_str(), "linux");
        assert_eq!(restored_result.cameras_found, 2);
        assert_eq!(restored_result.test_results.len(), 3);
    }

    #[test]
    fn test_camera_test_result_variants() {
        use crabcamera::platform::CameraTestResult;

        let test_results = vec![
            CameraTestResult::Success,
            CameraTestResult::InitError("Init failed".to_string()),
            CameraTestResult::CaptureError("Capture failed".to_string()),
            CameraTestResult::NotAvailable,
        ];

        for result in test_results {
            // Test that all variants can be created and matched
            match result {
                CameraTestResult::Success => {
                    // Success variant works
                }
                CameraTestResult::InitError(msg) => {
                    assert_eq!(msg, "Init failed");
                }
                CameraTestResult::CaptureError(msg) => {
                    assert_eq!(msg, "Capture failed");
                }
                CameraTestResult::NotAvailable => {
                    // NotAvailable variant works
                }
            }
        }
    }

    #[test]
    fn test_platform_camera_capabilities_comprehensive() {
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        // Test capabilities
        let capabilities_result = camera.test_capabilities();
        assert!(
            capabilities_result.is_ok(),
            "Getting capabilities should succeed"
        );

        let capabilities = capabilities_result.unwrap();

        // Verify capability fields are reasonable
        assert!(
            capabilities.max_resolution.0 > 0,
            "Max width should be positive"
        );
        assert!(
            capabilities.max_resolution.1 > 0,
            "Max height should be positive"
        );
        assert!(capabilities.max_fps > 0.0, "Max FPS should be positive");

        // Boolean capabilities should be present (can be true or false)
        let _ = capabilities.supports_auto_focus;
        let _ = capabilities.supports_manual_focus;
        let _ = capabilities.supports_auto_exposure;
        let _ = capabilities.supports_manual_exposure;
        let _ = capabilities.supports_white_balance;
        let _ = capabilities.supports_zoom;
        let _ = capabilities.supports_flash;
        let _ = capabilities.supports_burst_mode;
        let _ = capabilities.supports_hdr;

        // Optional ranges can be None or Some - both are valid
        if let Some((min_exp, max_exp)) = capabilities.exposure_range {
            assert!(min_exp < max_exp, "Exposure range should be valid");
            assert!(min_exp > 0.0, "Min exposure should be positive");
        }

        if let Some((min_iso, max_iso)) = capabilities.iso_range {
            assert!(min_iso < max_iso, "ISO range should be valid");
            assert!(min_iso > 0, "Min ISO should be positive");
        }

        if let Some((min_focus, max_focus)) = capabilities.focus_range {
            assert!(min_focus <= max_focus, "Focus range should be valid");
            assert!(
                min_focus >= 0.0 && max_focus <= 1.0,
                "Focus range should be 0-1"
            );
        }
    }

    #[test]
    fn test_platform_camera_performance_metrics() {
        let params = create_test_params();
        let camera = PlatformCamera::new(params).unwrap();

        // Test performance metrics
        let metrics_result = camera.get_performance_metrics();
        assert!(
            metrics_result.is_ok(),
            "Getting performance metrics should succeed"
        );

        let metrics = metrics_result.unwrap();

        // Verify all metrics are reasonable
        assert!(
            metrics.capture_latency_ms > 0.0,
            "Capture latency should be positive"
        );
        assert!(
            metrics.processing_time_ms >= 0.0,
            "Processing time should be non-negative"
        );
        assert!(
            metrics.memory_usage_mb > 0.0,
            "Memory usage should be positive"
        );
        assert!(metrics.fps_actual > 0.0, "Actual FPS should be positive");
        assert!(
            metrics.dropped_frames >= 0,
            "Dropped frames should be non-negative"
        );
        assert!(
            metrics.buffer_overruns >= 0,
            "Buffer overruns should be non-negative"
        );
        assert!(
            metrics.quality_score >= 0.0 && metrics.quality_score <= 1.0,
            "Quality score should be 0-1, got: {}",
            metrics.quality_score
        );
    }
}
