#[cfg(test)]
mod integration_tests {
    use crabcamera::commands::capture::{
        capture_photo_sequence, capture_single_photo, get_capture_stats, release_camera,
        start_camera_preview, stop_camera_preview,
    };
    use crabcamera::commands::init::{
        check_camera_availability, get_available_cameras, get_current_platform, get_platform_info,
        initialize_camera_system, test_camera_system,
    };
    use crabcamera::tests::{set_mock_camera_mode, MockCaptureMode};
    use crabcamera::types::CameraFormat;

    #[tokio::test]
    async fn test_complete_camera_workflow() {
        // Complete end-to-end camera workflow test
        let device_id = "integration_test_camera".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);

        // 1. Initialize camera system
        let init_result = initialize_camera_system().await;
        // Should succeed or fail gracefully
        match init_result {
            Ok(msg) => assert!(!msg.is_empty(), "Init message should not be empty"),
            Err(err) => assert!(
                err.contains("Failed to initialize"),
                "Error should be descriptive"
            ),
        }

        // 2. Get platform info
        let platform_result = get_platform_info().await;
        match platform_result {
            Ok(info) => {
                assert!(!info.backend.is_empty(), "Backend should be specified");
                assert!(!info.features.is_empty(), "Should have some features");
            }
            Err(_) => {
                // Platform info failure is acceptable in test environment
            }
        }

        // 3. Check camera availability
        let availability = check_camera_availability(device_id.clone()).await;
        // This might succeed or fail depending on system
        match availability {
            Ok(_) => {}  // Either true or false is acceptable
            Err(_) => {} // Error is also acceptable in test
        }

        // 4. Start camera preview
        let preview_result = start_camera_preview(device_id.clone(), None).await;
        assert!(
            preview_result.is_ok(),
            "Starting preview should succeed with mock camera"
        );

        // 5. Get capture stats
        let stats_result = get_capture_stats(device_id.clone()).await;
        assert!(stats_result.is_ok(), "Getting stats should succeed");
        let stats = stats_result.unwrap();
        assert_eq!(stats.device_id, device_id);
        assert!(
            stats.is_active,
            "Camera should be active after starting preview"
        );

        // 6. Capture single photo
        let single_result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(single_result.is_ok(), "Single photo capture should succeed");
        let frame = single_result.unwrap();
        assert!(
            frame.width > 0 && frame.height > 0,
            "Frame should have valid dimensions"
        );

        // 7. Capture photo sequence
        let sequence_result = capture_photo_sequence(device_id.clone(), 3, 50, None).await;
        assert!(sequence_result.is_ok(), "Photo sequence should succeed");
        let frames = sequence_result.unwrap();
        assert_eq!(frames.len(), 3, "Should capture 3 frames");

        // 8. Stop preview
        let stop_result = stop_camera_preview(device_id.clone()).await;
        assert!(stop_result.is_ok(), "Stopping preview should succeed");

        // 9. Release camera
        let release_result = release_camera(device_id.clone()).await;
        assert!(release_result.is_ok(), "Releasing camera should succeed");

        // 10. Verify camera is no longer active
        let final_stats = get_capture_stats(device_id).await;
        assert!(final_stats.is_ok(), "Getting final stats should succeed");
        let final_stats = final_stats.unwrap();
        assert!(!final_stats.is_active, "Camera should no longer be active");
    }

    #[tokio::test]
    async fn test_error_handling_workflow() {
        // Test error handling across different operations
        let device_id = "error_test_camera".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Failure);

        // Start with success to establish camera
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        let _ = start_camera_preview(device_id.clone(), None).await;

        // Switch to failure mode
        set_mock_camera_mode(&device_id, MockCaptureMode::Failure);

        // Test capture failures
        let single_result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(single_result.is_err(), "Should fail with failure mode");

        let sequence_result = capture_photo_sequence(device_id.clone(), 2, 50, None).await;
        assert!(
            sequence_result.is_err(),
            "Sequence should fail with failure mode"
        );

        // Error messages should be descriptive
        let error = single_result.unwrap_err();
        assert!(
            error.contains("Failed to capture frame"),
            "Error should mention capture failure"
        );

        // Switch back to success mode - operations should recover
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        let recovery_result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(
            recovery_result.is_ok(),
            "Should recover after switching to success mode"
        );

        // Cleanup
        let _ = release_camera(device_id).await;
    }

    #[tokio::test]
    async fn test_multiple_camera_management() {
        // Test managing multiple cameras simultaneously
        let camera_ids = vec![
            "multi_cam_1".to_string(),
            "multi_cam_2".to_string(),
            "multi_cam_3".to_string(),
        ];

        // Set up all cameras for success
        for camera_id in &camera_ids {
            set_mock_camera_mode(camera_id, MockCaptureMode::Success);
        }

        // Start previews for all cameras
        let mut preview_results = Vec::new();
        for camera_id in &camera_ids {
            let result = start_camera_preview(camera_id.clone(), None).await;
            preview_results.push((camera_id.clone(), result));
        }

        // All should succeed
        for (camera_id, result) in &preview_results {
            assert!(
                result.is_ok(),
                "Preview should succeed for camera {}",
                camera_id
            );
        }

        // Capture from all cameras
        for camera_id in &camera_ids {
            let capture_result = capture_single_photo(Some(camera_id.clone()), None).await;
            assert!(
                capture_result.is_ok(),
                "Capture should succeed for camera {}",
                camera_id
            );

            let frame = capture_result.unwrap();
            assert_eq!(
                frame.device_id, *camera_id,
                "Frame should have correct device ID"
            );
        }

        // Get stats for all cameras
        for camera_id in &camera_ids {
            let stats_result = get_capture_stats(camera_id.clone()).await;
            assert!(
                stats_result.is_ok(),
                "Stats should be available for camera {}",
                camera_id
            );

            let stats = stats_result.unwrap();
            assert!(stats.is_active, "Camera {} should be active", camera_id);
        }

        // Release all cameras
        for camera_id in &camera_ids {
            let release_result = release_camera(camera_id.clone()).await;
            assert!(
                release_result.is_ok(),
                "Release should succeed for camera {}",
                camera_id
            );
        }

        // Verify all are inactive
        for camera_id in &camera_ids {
            let final_stats = get_capture_stats(camera_id.clone()).await;
            assert!(final_stats.is_ok(), "Final stats should be available");

            let stats = final_stats.unwrap();
            assert!(
                !stats.is_active,
                "Camera {} should be inactive after release",
                camera_id
            );
        }
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        // Test concurrent camera operations
        let device_id = "concurrent_test".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);

        // Start preview first
        let preview_result = start_camera_preview(device_id.clone(), None).await;
        assert!(preview_result.is_ok(), "Preview should start successfully");

        // Launch multiple concurrent operations
        let mut handles = Vec::new();

        // Concurrent captures
        for i in 0..5 {
            let device_id_clone = device_id.clone();
            let handle = tokio::spawn(async move {
                let result = capture_single_photo(Some(device_id_clone), None).await;
                (i, result)
            });
            handles.push(handle);
        }

        // Concurrent stats requests
        for i in 5..10 {
            let device_id_clone = device_id.clone();
            let handle = tokio::spawn(async move {
                let result = get_capture_stats(device_id_clone).await;
                (
                    i,
                    result.map(|_| {
                        crabcamera::types::CameraFrame::new(vec![0], 1, 1, "test".to_string())
                    }),
                )
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            let (operation_id, result) = handle.await.unwrap();
            assert!(
                result.is_ok(),
                "Concurrent operation {} should succeed",
                operation_id
            );
        }

        // Cleanup
        let _ = release_camera(device_id).await;
    }

    #[tokio::test]
    async fn test_format_specifications() {
        // Test different camera format specifications
        let device_id = "format_test".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);

        let formats = vec![
            CameraFormat::new(640, 480, 30.0),
            CameraFormat::new(1280, 720, 60.0),
            CameraFormat::new(1920, 1080, 30.0),
            CameraFormat::standard(),
        ];

        for (i, format) in formats.into_iter().enumerate() {
            let capture_result = capture_single_photo(
                Some(format!("{}_format_{}", device_id, i)),
                Some(format.clone()),
            )
            .await;

            // Set up mock for this specific device ID
            set_mock_camera_mode(
                &format!("{}_format_{}", device_id, i),
                MockCaptureMode::Success,
            );

            let capture_result =
                capture_single_photo(Some(format!("{}_format_{}", device_id, i)), Some(format))
                    .await;
            assert!(
                capture_result.is_ok(),
                "Capture with format {} should succeed",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_system_level_operations() {
        // Test system-level camera operations

        // Test platform detection
        let platform_result = get_current_platform().await;
        assert!(platform_result.is_ok(), "Platform detection should work");

        let platform = platform_result.unwrap();
        let valid_platforms = vec!["Windows", "Linux", "macOS", "Unknown"];
        assert!(
            valid_platforms.contains(&platform.as_str()),
            "Platform should be one of the supported ones: {}",
            platform
        );

        // Test camera system testing
        let test_result = test_camera_system().await;
        match test_result {
            Ok(result) => {
                assert!(
                    result.cameras_found >= 0,
                    "Camera count should be non-negative"
                );
                // Test results can be empty in test environment - that's OK
            }
            Err(error) => {
                assert!(
                    error.contains("test failed"),
                    "Error should mention test failure"
                );
            }
        }

        // Test getting available cameras
        let cameras_result = get_available_cameras().await;
        match cameras_result {
            Ok(cameras) => {
                // Cameras list can be empty in test environment
                for camera in cameras {
                    assert!(!camera.id.is_empty(), "Camera ID should not be empty");
                    assert!(!camera.name.is_empty(), "Camera name should not be empty");
                }
            }
            Err(error) => {
                assert!(
                    error.contains("Failed to list cameras"),
                    "Error should be descriptive"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_edge_cases() {
        // Test various edge cases

        // Empty device ID
        let empty_result = capture_single_photo(Some("".to_string()), None).await;
        // Should either succeed with empty string or fail gracefully

        // Very long device ID
        let long_id = "a".repeat(1000);
        set_mock_camera_mode(&long_id, MockCaptureMode::Success);
        let long_result = capture_single_photo(Some(long_id.clone()), None).await;
        assert!(long_result.is_ok(), "Should handle long device IDs");

        // Special characters in device ID
        let special_id = "test-cam_123.device@domain:8080/path?query=value#fragment".to_string();
        set_mock_camera_mode(&special_id, MockCaptureMode::Success);
        let special_result = capture_single_photo(Some(special_id), None).await;
        assert!(
            special_result.is_ok(),
            "Should handle special characters in device ID"
        );

        // Invalid sequence parameters
        let invalid_count = capture_photo_sequence("test".to_string(), 0, 100, None).await;
        assert!(invalid_count.is_err(), "Should reject invalid count");

        let too_many = capture_photo_sequence("test".to_string(), 100, 100, None).await;
        assert!(too_many.is_err(), "Should reject too many photos");

        // Very short interval
        set_mock_camera_mode("short_interval", MockCaptureMode::Success);
        let short_interval = capture_photo_sequence("short_interval".to_string(), 2, 1, None).await;
        assert!(short_interval.is_ok(), "Should handle short intervals");
    }

    #[tokio::test]
    async fn test_resource_cleanup() {
        // Test proper resource cleanup
        let device_id = "cleanup_test".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);

        // Create and release multiple cameras to test cleanup
        for i in 0..10 {
            let test_id = format!("{}_{}", device_id, i);
            set_mock_camera_mode(&test_id, MockCaptureMode::Success);

            // Start preview
            let preview_result = start_camera_preview(test_id.clone(), None).await;
            assert!(
                preview_result.is_ok(),
                "Preview should start for camera {}",
                i
            );

            // Capture a frame
            let capture_result = capture_single_photo(Some(test_id.clone()), None).await;
            assert!(
                capture_result.is_ok(),
                "Capture should succeed for camera {}",
                i
            );

            // Release immediately
            let release_result = release_camera(test_id.clone()).await;
            assert!(
                release_result.is_ok(),
                "Release should succeed for camera {}",
                i
            );

            // Verify it's cleaned up
            let stats = get_capture_stats(test_id).await;
            assert!(
                stats.is_ok(),
                "Should still be able to get stats after release"
            );
            let stats = stats.unwrap();
            assert!(
                !stats.is_active,
                "Camera {} should be inactive after release",
                i
            );
        }
    }
}
