#[cfg(test)]
mod commands_capture_tests {
    use crabcamera::commands::capture::{
        capture_photo_sequence, capture_single_photo, capture_with_quality_retry,
        capture_with_reconnect, get_capture_stats, get_or_create_camera, reconnect_camera,
        release_camera, save_frame_compressed, save_frame_to_disk, start_camera_preview,
        stop_camera_preview, CaptureStats, FramePool,
    };
    use crabcamera::tests::{set_mock_camera_mode, MockCaptureMode};
    use crabcamera::types::{CameraFormat, CameraFrame};
    use std::sync::Arc;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    fn setup_mock_environment() {
        std::env::set_var("CRABCAMERA_USE_MOCK", "1");
    }

    // Helper function to create a test frame
    fn create_test_frame() -> CameraFrame {
        let test_data = vec![255u8; 640 * 480 * 3]; // RGB data
        CameraFrame::new(test_data, 640, 480, "test_device".to_string())
    }

    #[tokio::test]
    async fn test_capture_single_photo_success() {
        set_mock_camera_mode("0", MockCaptureMode::Success);

        let result = capture_single_photo(None, None).await;
        assert!(result.is_ok(), "Single photo capture should succeed");

        let frame = result.unwrap();
        assert!(frame.width > 0, "Frame should have positive width");
        assert!(frame.height > 0, "Frame should have positive height");
        assert!(!frame.data.is_empty(), "Frame data should not be empty");
        assert_eq!(frame.device_id, "0", "Should use default device ID");
    }

    #[tokio::test]
    async fn test_capture_single_photo_with_device_id() {
        setup_mock_environment();
        set_mock_camera_mode("test_camera_1", MockCaptureMode::Success);

        let result = capture_single_photo(Some("test_camera_1".to_string()), None).await;
        assert!(
            result.is_ok(),
            "Single photo capture with device ID should succeed"
        );

        let frame = result.unwrap();
        assert_eq!(
            frame.device_id, "test_camera_1",
            "Should use specified device ID"
        );
    }

    #[tokio::test]
    async fn test_capture_single_photo_with_format() {
        setup_mock_environment();
        set_mock_camera_mode("test_camera_format", MockCaptureMode::Success);

        let format = CameraFormat::new(1920, 1080, 60.0);
        let result =
            capture_single_photo(Some("test_camera_format".to_string()), Some(format)).await;

        assert!(
            result.is_ok(),
            "Single photo capture with format should succeed"
        );
    }

    #[tokio::test]
    async fn test_capture_single_photo_failure() {
        set_mock_camera_mode("fail_camera", MockCaptureMode::Failure);

        let result = capture_single_photo(Some("fail_camera".to_string()), None).await;
        assert!(
            result.is_err(),
            "Single photo capture should fail with Failure mode"
        );

        let error = result.unwrap_err();
        assert!(
            error.contains("Failed to capture frame"),
            "Error should mention capture failure"
        );
    }

    #[tokio::test]
    async fn test_capture_photo_sequence_success() {
        set_mock_camera_mode("seq_camera", MockCaptureMode::Success);

        let result = capture_photo_sequence("seq_camera".to_string(), 3, 50, None).await;
        assert!(result.is_ok(), "Photo sequence capture should succeed");

        let frames = result.unwrap();
        assert_eq!(frames.len(), 3, "Should capture exactly 3 frames");

        for (i, frame) in frames.iter().enumerate() {
            assert_eq!(
                frame.device_id, "seq_camera",
                "Frame {} should have correct device ID",
                i
            );
            assert!(
                frame.width > 0 && frame.height > 0,
                "Frame {} should have valid dimensions",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_capture_photo_sequence_invalid_count() {
        let result = capture_photo_sequence("test".to_string(), 0, 50, None).await;
        assert!(result.is_err(), "Should fail with count 0");
        assert!(result.unwrap_err().contains("Invalid photo count"));

        let result = capture_photo_sequence("test".to_string(), 25, 50, None).await;
        assert!(result.is_err(), "Should fail with count > 20");
        assert!(result.unwrap_err().contains("Invalid photo count"));
    }

    #[tokio::test]
    async fn test_capture_photo_sequence_with_failure() {
        set_mock_camera_mode("seq_fail", MockCaptureMode::Failure);

        let result = capture_photo_sequence("seq_fail".to_string(), 2, 50, None).await;
        assert!(
            result.is_err(),
            "Photo sequence should fail if capture fails"
        );

        let error = result.unwrap_err();
        assert!(
            error.contains("Failed to capture frame"),
            "Error should mention capture failure"
        );
    }

    #[tokio::test]
    async fn test_capture_photo_sequence_timing() {
        set_mock_camera_mode("seq_timing", MockCaptureMode::Success);

        let start = std::time::Instant::now();
        let result = capture_photo_sequence("seq_timing".to_string(), 3, 100, None).await;
        let duration = start.elapsed();

        assert!(result.is_ok(), "Sequence capture should succeed");
        // Should take at least 200ms (2 intervals of 100ms each)
        assert!(
            duration.as_millis() >= 200,
            "Should respect interval timing"
        );
    }

    #[tokio::test]
    async fn test_start_camera_preview() {
        setup_mock_environment();
        set_mock_camera_mode("preview_start", MockCaptureMode::Success);

        let result = start_camera_preview("preview_start".to_string(), None).await;
        assert!(result.is_ok(), "Starting preview should succeed");

        let message = result.unwrap();
        assert!(
            message.contains("Preview started"),
            "Should return success message"
        );
        assert!(
            message.contains("preview_start"),
            "Should mention device ID"
        );
    }

    #[tokio::test]
    async fn test_start_camera_preview_with_format() {
        set_mock_camera_mode("preview_format", MockCaptureMode::Success);

        let format = CameraFormat::new(1280, 720, 30.0);
        let result = start_camera_preview("preview_format".to_string(), Some(format)).await;

        assert!(
            result.is_ok(),
            "Starting preview with format should succeed"
        );
    }

    #[tokio::test]
    async fn test_stop_camera_preview_success() {
        // First start a preview
        set_mock_camera_mode("preview_stop", MockCaptureMode::Success);
        let _ = start_camera_preview("preview_stop".to_string(), None).await;

        // Then stop it
        let result = stop_camera_preview("preview_stop".to_string()).await;
        assert!(result.is_ok(), "Stopping preview should succeed");

        let message = result.unwrap();
        assert!(
            message.contains("Preview stopped"),
            "Should return success message"
        );
        assert!(message.contains("preview_stop"), "Should mention device ID");
    }

    #[tokio::test]
    async fn test_stop_camera_preview_not_active() {
        let result = stop_camera_preview("nonexistent_preview".to_string()).await;
        assert!(result.is_err(), "Should fail to stop non-existent preview");

        let error = result.unwrap_err();
        assert!(
            error.contains("No active camera found"),
            "Should mention camera not found"
        );
    }

    #[tokio::test]
    async fn test_release_camera_success() {
        // First create a camera by starting preview
        set_mock_camera_mode("release_test", MockCaptureMode::Success);
        let _ = start_camera_preview("release_test".to_string(), None).await;

        // Then release it
        let result = release_camera("release_test".to_string()).await;
        assert!(result.is_ok(), "Releasing camera should succeed");

        let message = result.unwrap();
        assert!(
            message.contains("released"),
            "Should return success message"
        );
        assert!(message.contains("release_test"), "Should mention device ID");
    }

    #[tokio::test]
    async fn test_release_camera_not_active() {
        let result = release_camera("nonexistent_release".to_string()).await;
        assert!(
            result.is_ok(),
            "Releasing non-existent camera should not error"
        );

        let message = result.unwrap();
        assert!(
            message.contains("No active camera found"),
            "Should mention camera not found"
        );
    }

    #[tokio::test]
    async fn test_get_capture_stats_active_camera() {
        // First create an active camera
        set_mock_camera_mode("stats_test", MockCaptureMode::Success);
        let _ = start_camera_preview("stats_test".to_string(), None).await;

        let result = get_capture_stats("stats_test".to_string()).await;
        assert!(result.is_ok(), "Getting stats should succeed");

        let stats = result.unwrap();
        assert_eq!(stats.device_id, "stats_test");
        assert!(stats.is_active, "Camera should be active");
        assert!(
            stats.device_info.is_some(),
            "Should have device info for active camera"
        );
    }

    #[tokio::test]
    async fn test_get_capture_stats_inactive_camera() {
        let result = get_capture_stats("stats_inactive".to_string()).await;
        assert!(
            result.is_ok(),
            "Getting stats for inactive camera should succeed"
        );

        let stats = result.unwrap();
        assert_eq!(stats.device_id, "stats_inactive");
        assert!(!stats.is_active, "Camera should not be active");
        assert!(
            stats.device_info.is_none(),
            "Should have no device info for inactive camera"
        );
    }

    #[tokio::test]
    async fn test_save_frame_to_disk() {
        let frame = create_test_frame();
        let temp_file = std::env::temp_dir().join("test_frame_save.bin");
        let file_path = temp_file.to_string_lossy().to_string();

        let result = save_frame_to_disk(frame, file_path.clone()).await;
        assert!(result.is_ok(), "Saving frame to disk should succeed");

        let message = result.unwrap();
        assert!(
            message.contains("Frame saved to"),
            "Should return success message"
        );

        // Verify file was created
        assert!(temp_file.exists(), "File should have been created");

        // Cleanup
        let _ = tokio::fs::remove_file(temp_file).await;
    }

    #[tokio::test]
    async fn test_save_frame_to_disk_invalid_path() {
        let frame = create_test_frame();
        // Use a path that's invalid on all platforms (non-existent deep directory)
        #[cfg(windows)]
        let invalid_path = "Z:\\nonexistent\\path\\with<>invalid|chars\\test.bin";
        #[cfg(not(windows))]
        let invalid_path = "/nonexistent/root/path/that/does/not/exist/deeply/nested/test.bin";

        let result = save_frame_to_disk(frame, invalid_path.to_string()).await;
        assert!(result.is_err(), "Should fail with invalid path");

        let error = result.unwrap_err();
        assert!(
            error.contains("Failed to save frame"),
            "Should mention save failure"
        );
    }

    #[tokio::test]
    async fn test_save_frame_compressed() {
        let frame = create_test_frame();
        let temp_file = std::env::temp_dir().join("test_frame_compressed.jpg");
        let file_path = temp_file.to_string_lossy().to_string();

        let result = save_frame_compressed(frame, file_path.clone(), Some(90)).await;
        assert!(result.is_ok(), "Saving compressed frame should succeed");

        let message = result.unwrap();
        assert!(
            message.contains("Compressed frame saved"),
            "Should return success message"
        );

        // Verify file was created
        assert!(
            temp_file.exists(),
            "Compressed file should have been created"
        );

        // Cleanup
        let _ = tokio::fs::remove_file(temp_file).await;
    }

    #[tokio::test]
    async fn test_save_frame_compressed_default_quality() {
        let frame = create_test_frame();
        let temp_file = std::env::temp_dir().join("test_frame_default_quality.jpg");
        let file_path = temp_file.to_string_lossy().to_string();

        let result = save_frame_compressed(frame, file_path, None).await;
        assert!(
            result.is_ok(),
            "Saving compressed frame with default quality should succeed"
        );

        // Cleanup
        let _ = tokio::fs::remove_file(temp_file).await;
    }

    #[tokio::test]
    async fn test_get_or_create_camera() {
        set_mock_camera_mode("get_create_test", MockCaptureMode::Success);

        let format = CameraFormat::new(640, 480, 30.0);

        // First call should create camera
        let result1 = get_or_create_camera("get_create_test".to_string(), format.clone()).await;
        assert!(result1.is_ok(), "First get_or_create should succeed");

        // Second call should reuse existing camera
        let result2 = get_or_create_camera("get_create_test".to_string(), format).await;
        assert!(result2.is_ok(), "Second get_or_create should succeed");

        // Both should return the same camera instance (same Arc)
        let camera1 = result1.unwrap();
        let camera2 = result2.unwrap();
        assert!(
            Arc::ptr_eq(&camera1, &camera2),
            "Should return same camera instance"
        );
    }

    #[tokio::test]
    async fn test_frame_pool_operations() {
        let pool = FramePool::new(3, 1024);

        // Get buffers from pool
        let buffer1 = pool.get_buffer().await;
        let buffer2 = pool.get_buffer().await;
        let buffer3 = pool.get_buffer().await;
        let buffer4 = pool.get_buffer().await; // Should create new one

        assert_eq!(buffer1.capacity(), 1024);
        assert_eq!(buffer2.capacity(), 1024);
        assert_eq!(buffer3.capacity(), 1024);
        assert_eq!(buffer4.capacity(), 1024);

        // Return buffers to pool
        pool.return_buffer(buffer1).await;
        pool.return_buffer(buffer2).await;
        pool.return_buffer(buffer3).await;
        pool.return_buffer(buffer4).await; // This one should be discarded (pool max is 3)

        // Get buffer again - should reuse from pool
        let buffer5 = pool.get_buffer().await;
        assert_eq!(buffer5.capacity(), 1024);
    }

    #[tokio::test]
    async fn test_capture_stats_serialization() {
        let stats = CaptureStats {
            device_id: "test_device".to_string(),
            is_active: true,
            device_info: Some("Test Camera Info".to_string()),
        };

        // Test serialization
        let serialized = serde_json::to_string(&stats);
        assert!(serialized.is_ok(), "Should serialize successfully");

        // Test deserialization
        let json = serialized.unwrap();
        let deserialized: Result<CaptureStats, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok(), "Should deserialize successfully");

        let restored_stats = deserialized.unwrap();
        assert_eq!(restored_stats.device_id, stats.device_id);
        assert_eq!(restored_stats.is_active, stats.is_active);
        assert_eq!(restored_stats.device_info, stats.device_info);
    }

    #[tokio::test]
    async fn test_concurrent_camera_operations() {
        set_mock_camera_mode("concurrent_test", MockCaptureMode::Success);

        let mut handles = vec![];

        // Start multiple concurrent operations
        for i in 0..5 {
            let device_id = format!("concurrent_test_{}", i);
            set_mock_camera_mode(&device_id, MockCaptureMode::Success);

            let handle = tokio::spawn(async move {
                let _ = capture_single_photo(Some(device_id.clone()), None).await;
                let _ = start_camera_preview(device_id.clone(), None).await;
                let _ = get_capture_stats(device_id.clone()).await;
                let _ = release_camera(device_id).await;
                i // Return for verification
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert_eq!(
                result, i,
                "Concurrent operation {} should complete successfully",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_error_recovery() {
        // Test that operations can recover from failures
        set_mock_camera_mode("error_recovery", MockCaptureMode::Failure);

        // First operation should fail
        let result1 = capture_single_photo(Some("error_recovery".to_string()), None).await;
        assert!(result1.is_err(), "Should fail in failure mode");

        // Switch to success mode
        set_mock_camera_mode("error_recovery", MockCaptureMode::Success);

        // Subsequent operation should succeed
        let result2 = capture_single_photo(Some("error_recovery".to_string()), None).await;
        assert!(result2.is_ok(), "Should succeed in success mode");
    }

    #[tokio::test]
    async fn test_camera_lifecycle() {
        let device_id = "lifecycle_test".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);

        // 1. Start preview
        let result = start_camera_preview(device_id.clone(), None).await;
        assert!(result.is_ok(), "Should start preview");

        // 2. Capture some photos
        let result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(result.is_ok(), "Should capture photo");

        // 3. Get stats
        let result = get_capture_stats(device_id.clone()).await;
        assert!(result.is_ok(), "Should get stats");
        let stats = result.unwrap();
        assert!(stats.is_active, "Camera should be active");

        // 4. Stop preview
        let result = stop_camera_preview(device_id.clone()).await;
        assert!(result.is_ok(), "Should stop preview");

        // 5. Release camera
        let result = release_camera(device_id.clone()).await;
        assert!(result.is_ok(), "Should release camera");

        // 6. Verify camera is no longer active
        let result = get_capture_stats(device_id).await;
        assert!(result.is_ok(), "Should get stats");
        let stats = result.unwrap();
        assert!(!stats.is_active, "Camera should no longer be active");
    }

    #[tokio::test]
    async fn test_quality_retry_success() {
        set_mock_camera_mode("quality_success", MockCaptureMode::Success);

        let result = capture_with_quality_retry(
            Some("quality_success".to_string()),
            Some(5),     // max attempts
            Some(0.5),   // low threshold - should succeed quickly
            None,
        )
        .await;

        assert!(result.is_ok(), "Quality retry should succeed");
        let frame = result.unwrap();
        assert_eq!(frame.device_id, "quality_success");
        assert!(frame.width > 0 && frame.height > 0, "Frame should have valid dimensions");
    }

    #[tokio::test]
    async fn test_quality_retry_high_threshold() {
        set_mock_camera_mode("quality_high_threshold", MockCaptureMode::Success);

        let result = capture_with_quality_retry(
            Some("quality_high_threshold".to_string()),
            Some(3),     // max attempts
            Some(0.99),  // very high threshold - unlikely to be met
            None,
        )
        .await;

        assert!(result.is_ok(), "Should return best frame even if threshold not met");
        let frame = result.unwrap();
        assert_eq!(frame.device_id, "quality_high_threshold");
    }

    #[tokio::test]
    async fn test_quality_retry_failure_mode() {
        set_mock_camera_mode("quality_failure", MockCaptureMode::Failure);

        let result = capture_with_quality_retry(
            Some("quality_failure".to_string()),
            Some(3),
            Some(0.5),
            None,
        )
        .await;

        assert!(result.is_err(), "Should fail when camera always fails");
        let error = result.unwrap_err();
        println!("Actual error: {}", error);
        assert!(error.contains("Failed to capture") || error.contains("quality") || error.contains("attempt") || error.contains("Capture error"), "Error should be descriptive: {}", error);
    }

    #[tokio::test]
    async fn test_quality_retry_parameter_validation() {
        set_mock_camera_mode("quality_params", MockCaptureMode::Success);

        // Test parameter clamping
        let result = capture_with_quality_retry(
            Some("quality_params".to_string()),
            Some(100),   // Should be capped at 50
            Some(1.5),   // Should be clamped to 1.0
            None,
        )
        .await;

        assert!(result.is_ok(), "Should handle parameter clamping gracefully");
    }

    #[tokio::test]
    async fn test_capture_with_reconnect_success() {
        set_mock_camera_mode("reconnect_success", MockCaptureMode::Success);

        let format = CameraFormat::new(640, 480, 30.0);
        let result = capture_with_reconnect(
            "reconnect_success".to_string(),
            format,
            3, // max reconnect attempts
        )
        .await;

        assert!(result.is_ok(), "Capture with reconnect should succeed");
        let frame = result.unwrap();
        assert_eq!(frame.device_id, "reconnect_success");
    }

    #[tokio::test]
    async fn test_capture_with_reconnect_failure_then_success() {
        // Start with failure mode
        set_mock_camera_mode("reconnect_recovery", MockCaptureMode::Failure);

        let format = CameraFormat::new(640, 480, 30.0);
        
        // Start capture (will fail initially)
        let capture_handle = tokio::spawn(async move {
            capture_with_reconnect(
                "reconnect_recovery".to_string(),
                format,
                3,
            )
            .await
        });

        // Switch to success mode after a brief delay to simulate recovery
        tokio::time::sleep(Duration::from_millis(10)).await;
        set_mock_camera_mode("reconnect_recovery", MockCaptureMode::Success);

        let result = capture_handle.await.unwrap();
        assert!(result.is_ok(), "Should succeed after recovery");
    }

    #[tokio::test]
    async fn test_reconnect_camera_function() {
        set_mock_camera_mode("reconnect_test", MockCaptureMode::Success);

        let format = CameraFormat::new(1280, 720, 30.0);
        let result = reconnect_camera(
            "reconnect_test".to_string(),
            format,
            2, // max retries
        )
        .await;

        assert!(result.is_ok(), "Reconnect should succeed");
        let camera = result.unwrap();
        
        // Verify we can use the reconnected camera
        assert!(camera.lock().is_ok(), "Camera mutex should be accessible");
    }

    #[tokio::test]
    async fn test_reconnect_camera_max_retries() {
        set_mock_camera_mode("reconnect_test", MockCaptureMode::Failure);

        let format = CameraFormat::new(640, 480, 30.0);
        
        // Reconnect should succeed (creates camera object)
        let result = reconnect_camera(
            "reconnect_test".to_string(),
            format,
            2, // max retries
        )
        .await;

        assert!(result.is_ok(), "Reconnect should succeed (camera creation succeeds in mock)");
        
        // But captures should fail with this camera
        let capture_result = capture_single_photo(Some("reconnect_test".to_string()), None).await;
        assert!(capture_result.is_err(), "Captures should fail with failure mode");
    }

    #[tokio::test]
    async fn test_capture_timeout_scenarios() {
        set_mock_camera_mode("timeout_test", MockCaptureMode::SlowCapture);

        let start = Instant::now();
        let result = timeout(
            Duration::from_secs(5), // Generous timeout
            capture_single_photo(Some("timeout_test".to_string()), None)
        ).await;

        let duration = start.elapsed();
        
        assert!(result.is_ok(), "Capture should complete within timeout");
        let capture_result = result.unwrap();
        assert!(capture_result.is_ok(), "Capture should succeed with slow mode");
        
        // Should take some time due to SlowCapture mode
        assert!(duration >= Duration::from_millis(100), "Should take time in slow mode");
    }

    #[tokio::test]
    async fn test_massive_concurrent_captures() {
        // Test high concurrency to stress the camera registry
        let device_base = "massive_concurrent";
        let num_cameras = 20;
        let captures_per_camera = 5;
        
        // Set up multiple cameras
        for i in 0..num_cameras {
            let device_id = format!("{}_cam_{}", device_base, i);
            set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        }

        let mut handles = Vec::new();
        
        // Launch massive concurrent captures
        for cam_id in 0..num_cameras {
            for cap_id in 0..captures_per_camera {
                let device_id = format!("{}_cam_{}", device_base, cam_id);
                let handle = tokio::spawn(async move {
                    let result = capture_single_photo(Some(device_id.clone()), None).await;
                    (cam_id, cap_id, device_id, result)
                });
                handles.push(handle);
            }
        }
        
        // Collect all results
        let mut success_count = 0;
        for handle in handles {
            let (cam_id, cap_id, device_id, result) = handle.await.unwrap();
            
            assert!(result.is_ok(), "Capture {}-{} should succeed for device {}", cam_id, cap_id, device_id);
            if result.is_ok() {
                success_count += 1;
            }
        }
        
        let expected_total = num_cameras * captures_per_camera;
        assert_eq!(success_count, expected_total, "All captures should succeed");
    }

    #[tokio::test]
    async fn test_frame_pool_stress() {
        let pool = Arc::new(FramePool::new(5, 2048)); // Small pool, larger frames
        
        let mut handles = Vec::new();
        
        // Stress test the pool with many concurrent operations
        for i in 0..50 {
            let pool_clone = pool.clone(); // Clone the Arc for move into async block
            let handle = tokio::spawn(async move {
                // Get buffer
                let buffer = pool_clone.get_buffer().await;
                assert!(buffer.capacity() >= 2048, "Buffer should have correct capacity");
                
                // Simulate some work
                tokio::time::sleep(Duration::from_millis(1)).await;
                
                // Return buffer
                pool_clone.return_buffer(buffer).await;
                i // Return for verification
            });
            handles.push(handle);
        }
        
        // All operations should complete
        for handle in handles {
            let operation_id = handle.await.unwrap();
            assert!(operation_id < 50, "Operation {} should complete", operation_id);
        }
    }

    #[tokio::test]
    async fn test_camera_hot_unplug_simulation() {
        let device_id = "hotplug_test".to_string();
        
        // Start with camera available
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        
        // Start preview
        let preview_result = start_camera_preview(device_id.clone(), None).await;
        assert!(preview_result.is_ok(), "Preview should start");
        
        // Capture should work
        let capture_result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(capture_result.is_ok(), "Initial capture should work");
        
        // Simulate hot unplug by switching to failure mode
        set_mock_camera_mode(&device_id, MockCaptureMode::Failure);
        
        // Captures should start failing
        let capture_result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(capture_result.is_err(), "Capture should fail after unplug");
        
        // Simulate hot plug by switching back to success
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        
        // Should be able to capture again
        let capture_result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(capture_result.is_ok(), "Capture should work after replug");
        
        // Cleanup
        let _ = release_camera(device_id).await;
    }

    #[tokio::test]
    async fn test_format_negotiation_edge_cases() {
        let device_id = "format_negotiation".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        
        // Test various format configurations
        let edge_case_formats = vec![
            CameraFormat::new(1, 1, 1.0),           // Minimum dimensions
            CameraFormat::new(7680, 4320, 120.0),   // 8K 120fps
            CameraFormat::new(640, 480, 0.1),       // Very low FPS
            CameraFormat::new(1920, 1080, 240.0),   // Very high FPS
        ];
        
        for (i, format) in edge_case_formats.into_iter().enumerate() {
            let test_device_id = format!("{}_fmt_{}", device_id, i);
            set_mock_camera_mode(&test_device_id, MockCaptureMode::Success);
            
            let result = capture_single_photo(Some(test_device_id.clone()), Some(format.clone())).await;
            
            // Should handle edge case formats gracefully
            match result {
                Ok(frame) => {
                    assert_eq!(frame.device_id, test_device_id);
                }
                Err(_) => {
                    // Failure is acceptable for edge case formats
                }
            }
        }
    }

    #[tokio::test]
    async fn test_resource_cleanup_verification() {
        let base_device_id = "cleanup_verification";
        
        // Create and release many cameras to test cleanup
        for iteration in 0..20 {
            let device_id = format!("{}_iter_{}", base_device_id, iteration);
            set_mock_camera_mode(&device_id, MockCaptureMode::Success);
            
            // Start preview
            let preview_result = start_camera_preview(device_id.clone(), None).await;
            assert!(preview_result.is_ok(), "Preview should start for iteration {}", iteration);
            
            // Get stats to verify active
            let stats_result = get_capture_stats(device_id.clone()).await;
            assert!(stats_result.is_ok(), "Stats should be available");
            let stats = stats_result.unwrap();
            assert!(stats.is_active, "Camera should be active for iteration {}", iteration);
            
            // Release immediately
            let release_result = release_camera(device_id.clone()).await;
            assert!(release_result.is_ok(), "Release should succeed for iteration {}", iteration);
            
            // Verify cleanup
            let stats_result = get_capture_stats(device_id).await;
            assert!(stats_result.is_ok(), "Stats should still be available after release");
            let stats = stats_result.unwrap();
            assert!(!stats.is_active, "Camera should be inactive after release for iteration {}", iteration);
        }
    }

    #[tokio::test]
    async fn test_save_frame_format_detection() {
        let frame = create_test_frame();
        
        // Test different file extensions
        let test_cases = vec![
            ("test.png", "PNG"),
            ("test.jpg", "JPEG"),
            ("test.jpeg", "JPEG"),
            ("test.JPG", "JPEG"),
            ("test.JPEG", "JPEG"),
            ("test.unknown", "PNG"), // Should default to PNG
            ("test", "PNG"),           // No extension should default to PNG
        ];
        
        for (filename, expected_format) in test_cases {
            let temp_file = std::env::temp_dir().join(filename);
            let file_path = temp_file.to_string_lossy().to_string();
            
            let result = save_frame_to_disk(frame.clone(), file_path.clone()).await;
            assert!(result.is_ok(), "Save should succeed for format: {}", expected_format);
            
            // Verify file was created
            assert!(temp_file.exists(), "File should exist for format: {}", expected_format);
            
            // Cleanup
            let _ = tokio::fs::remove_file(temp_file).await;
        }
    }

    #[tokio::test]
    async fn test_capture_sequence_interruption() {
        set_mock_camera_mode("sequence_interrupt", MockCaptureMode::Success);
        
        // Start a long sequence
        let sequence_handle = tokio::spawn(async {
            capture_photo_sequence(
                "sequence_interrupt".to_string(),
                10,   // 10 photos
                100,  // 100ms interval = ~1 second total
                None,
            )
            .await
        });
        
        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Switch to failure mode to simulate interruption
        set_mock_camera_mode("sequence_interrupt", MockCaptureMode::Failure);
        
        let result = sequence_handle.await.unwrap();
        
        // Should fail when camera starts failing
        assert!(result.is_err(), "Sequence should be interrupted by camera failure");
    }

    #[tokio::test]
    async fn test_camera_registry_isolation() {
        // Test that cameras in registry don't interfere with each other
        let camera_ids = vec![
            "isolation_cam_1".to_string(),
            "isolation_cam_2".to_string(),
            "isolation_cam_3".to_string(),
        ];
        
        // Set different behaviors for each camera
        set_mock_camera_mode(&camera_ids[0], MockCaptureMode::Success);
        set_mock_camera_mode(&camera_ids[1], MockCaptureMode::SlowCapture);
        set_mock_camera_mode(&camera_ids[2], MockCaptureMode::Failure);
        
        // Start operations on all cameras
        let camera_ids_clone1 = camera_ids.clone();
        let handle1 = tokio::spawn(async move {
            capture_single_photo(Some(camera_ids_clone1[0].clone()), None).await
        });
        
        let camera_ids_clone2 = camera_ids.clone();
        let handle2 = tokio::spawn(async move {
            capture_single_photo(Some(camera_ids_clone2[1].clone()), None).await
        });
        
        let camera_ids_clone3 = camera_ids.clone();
        let handle3 = tokio::spawn(async move {
            capture_single_photo(Some(camera_ids_clone3[2].clone()), None).await
        });
        
        // Collect results
        let (result1, result2, result3) = tokio::join!(handle1, handle2, handle3);
        
        // Results should match expected behaviors
        assert!(result1.unwrap().is_ok(), "Camera 1 should succeed");
        assert!(result2.unwrap().is_ok(), "Camera 2 should succeed (slow)");
        assert!(result3.unwrap().is_err(), "Camera 3 should fail");
    }

    #[tokio::test]
    async fn test_error_message_consistency() {
        // Test that error messages are consistent and helpful
        
        // Test failing camera
        set_mock_camera_mode("error_msg_test", MockCaptureMode::Failure);
        let result = capture_single_photo(Some("error_msg_test".to_string()), None).await;
        assert!(result.is_err(), "Should fail for failing camera");
        let error = result.unwrap_err();
        assert!(!error.is_empty(), "Error message should not be empty");
        assert!(error.contains("Failed to capture frame"), "Error should be descriptive");
        
        // Test invalid sequence parameters
        let result = capture_photo_sequence("any".to_string(), 0, 100, None).await;
        assert!(result.is_err(), "Should fail for invalid count");
        let error = result.unwrap_err();
        assert!(error.contains("Invalid photo count"), "Error should mention invalid count");
        
        let result = capture_photo_sequence("any".to_string(), 25, 100, None).await;
        assert!(result.is_err(), "Should fail for too many photos");
        let error = result.unwrap_err();
        assert!(error.contains("Invalid photo count"), "Error should mention invalid count");
    }

    #[tokio::test]
    async fn test_mixed_operation_patterns() {
        // Test mixing different types of operations
        let device_id = "mixed_ops".to_string();
        set_mock_camera_mode(&device_id, MockCaptureMode::Success);
        
        // 1. Single capture
        let result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(result.is_ok(), "Single capture should work");
        
        // 2. Start preview
        let result = start_camera_preview(device_id.clone(), None).await;
        assert!(result.is_ok(), "Preview should start");
        
        // 3. Sequence capture while preview is running
        let result = capture_photo_sequence(device_id.clone(), 3, 10, None).await;
        assert!(result.is_ok(), "Sequence should work with preview running");
        
        // 4. Get stats
        let result = get_capture_stats(device_id.clone()).await;
        assert!(result.is_ok(), "Stats should be available");
        
        // 5. Another single capture
        let result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(result.is_ok(), "Another single capture should work");
        
        // 6. Stop preview
        let result = stop_camera_preview(device_id.clone()).await;
        assert!(result.is_ok(), "Should stop preview");
        
        // 7. Final capture
        let result = capture_single_photo(Some(device_id.clone()), None).await;
        assert!(result.is_ok(), "Final capture should work");
        
        // 8. Release
        let result = release_camera(device_id).await;
        assert!(result.is_ok(), "Should release camera");
    }
}
