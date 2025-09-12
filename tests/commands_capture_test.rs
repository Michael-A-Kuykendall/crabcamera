#[cfg(test)]
mod commands_capture_tests {
    use crabcamera::commands::capture::{
        capture_photo_sequence, capture_single_photo, get_capture_stats, get_or_create_camera,
        release_camera, save_frame_compressed, save_frame_to_disk, start_camera_preview,
        stop_camera_preview, CaptureStats, FramePool,
    };
    use crabcamera::tests::{set_mock_camera_mode, MockCaptureMode};
    use crabcamera::types::{CameraFormat, CameraFrame};
    use std::sync::Arc;
    use tokio::sync::Mutex;

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
        let invalid_path = "/invalid/path/that/does/not/exist/test.bin";

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
}
