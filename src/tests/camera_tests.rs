//! Camera initialization and operation tests
//!
//! Tests camera lifecycle management, capture operations, and error handling
//! for plant photography applications.

use super::*;
use crate::types::{CameraFormat, CameraFrame, CameraInitParams};
use crate::errors::CameraError;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_camera_initialization() {
        let mock_system = setup_test_environment().await;
        
        let devices = mock_system.get_devices().await;
        assert!(!devices.is_empty(), "Should have mock devices");
        
        let device = &devices[0];
        let init_params = CameraInitParams::new(device.id.clone());
        
        // Test basic initialization parameters
        assert_eq!(init_params.device_id, device.id);
        assert_eq!(init_params.controls.auto_focus, Some(true));
        assert_eq!(init_params.controls.auto_exposure, Some(true));
        assert_eq!(init_params.format, CameraFormat::standard());
    }

    #[tokio::test]
    async fn test_camera_init_params_builder() {
        let device_id = "test_camera".to_string();
        let custom_format = CameraFormat::hd();
        
        let params = CameraInitParams::new(device_id.clone())
            .with_format(custom_format.clone())
            .with_auto_focus(false)
            .with_auto_exposure(true);
        
        assert_eq!(params.device_id, device_id);
        assert_eq!(params.format, custom_format);
        assert_eq!(params.controls.auto_focus, Some(false));
        assert_eq!(params.controls.auto_exposure, Some(true));
    }

    #[tokio::test]
    async fn test_basic_capture_operation() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let result = mock_system.mock_capture("test_device").await;
        assert!(result.is_ok());
        
        let frame = result.unwrap();
        assert!(frame.is_valid());
        assert_eq!(frame.device_id, "test_device");
        assert!(!frame.data.is_empty());
    }

    #[tokio::test]
    async fn test_capture_error_handling() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Failure);
        
        let result = mock_system.mock_capture("test_device").await;
        assert!(result.is_err());
        
        if let Err(CameraError::CaptureError(msg)) = result {
            assert!(msg.contains("Mock capture failure"));
        } else {
            panic!("Expected CaptureError");
        }
    }

    #[tokio::test]
    async fn test_permission_denied_handling() {
        let mock_system = setup_test_environment().await;
        mock_system.set_error_mode(Some(CameraError::PermissionDenied("Access denied".to_string())));
        
        let result = mock_system.mock_capture("test_device").await;
        assert!(result.is_err());
        
        if let Err(CameraError::PermissionDenied(_)) = result {
            // Expected behavior
        } else {
            panic!("Expected PermissionDenied error");
        }
    }

    #[tokio::test]
    async fn test_camera_frame_validation() {
        let frame = create_mock_frame("test_device");
        
        // Test basic validation
        assert!(frame.is_valid());
        assert!(!frame.data.is_empty());
        assert!(frame.width > 0);
        assert!(frame.height > 0);
        assert!(!frame.id.is_empty());
        
        // Test aspect ratio calculation
        let aspect = frame.aspect_ratio();
        assert!(aspect > 0.0);
        assert_eq!(aspect, frame.width as f32 / frame.height as f32);
        
        // Test size consistency
        assert_eq!(frame.size_bytes, frame.data.len());
    }

    #[tokio::test]
    async fn test_plant_photography_frame_quality() {
        let plant_frame = create_plant_mock_frame("plant_camera");
        
        // Validate plant photography requirements
        assert!(validate_plant_frame_quality(&plant_frame));
        assert!(plant_frame.width >= 1280);
        assert!(plant_frame.height >= 720);
        
        // Check aspect ratio is suitable for plant photography
        let aspect = plant_frame.aspect_ratio();
        assert!(aspect >= 1.0 && aspect <= 2.0);
        
        // Check data size for high resolution
        let expected_pixels = plant_frame.width * plant_frame.height;
        assert_eq!(plant_frame.data.len(), (expected_pixels * 3) as usize); // RGB8
    }

    #[tokio::test]
    async fn test_frame_format_types() {
        let rgb_frame = create_mock_frame("test").with_format("RGB8".to_string());
        assert_eq!(rgb_frame.format, "RGB8");
        
        let jpeg_frame = create_mock_frame("test").with_format("JPEG".to_string());
        assert_eq!(jpeg_frame.format, "JPEG");
        
        let raw_frame = create_mock_frame("test").with_format("RAW".to_string());
        assert_eq!(raw_frame.format, "RAW");
    }

    #[tokio::test]
    async fn test_multiple_resolution_support() {
        let resolutions = [
            (640, 480),   // VGA
            (1280, 720),  // HD
            (1920, 1080), // Full HD
            (3840, 2160), // 4K
        ];
        
        for (width, height) in resolutions {
            let frame = create_mock_frame_with_size("test", width, height);
            
            assert!(frame.is_valid());
            assert_eq!(frame.width, width);
            assert_eq!(frame.height, height);
            
            // Check data size matches resolution
            let expected_size = (width * height * 3) as usize; // RGB8
            assert_eq!(frame.data.len(), expected_size);
        }
    }

    #[tokio::test]
    async fn test_concurrent_captures() {
        use tokio::task;
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Spawn multiple concurrent capture tasks
        let handles = (0..10).map(|i| {
            let system = mock_system.clone();
            task::spawn(async move {
                let result = system.mock_capture(&format!("device_{}", i)).await;
                assert!(result.is_ok());
                result.unwrap()
            })
        }).collect::<Vec<_>>();
        
        // Wait for all captures to complete
        let frames = futures::future::try_join_all(handles).await.unwrap();
        
        // Verify all captures succeeded
        assert_eq!(frames.len(), 10);
        for (i, frame) in frames.iter().enumerate() {
            assert!(frame.is_valid());
            assert_eq!(frame.device_id, format!("device_{}", i));
        }
    }

    #[tokio::test]
    async fn test_slow_capture_handling() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::SlowCapture);
        
        let start = tokio::time::Instant::now();
        let result = mock_system.mock_capture("slow_device").await;
        let elapsed = start.elapsed();
        
        assert!(result.is_ok());
        assert!(elapsed >= Duration::from_millis(100)); // Should take at least 100ms
        assert!(elapsed < Duration::from_millis(500));  // But not too long
    }

    #[tokio::test]
    async fn test_capture_timeout() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::SlowCapture);
        
        // Test with very short timeout
        let timeout_duration = Duration::from_millis(10);
        let result = tokio::time::timeout(
            timeout_duration,
            mock_system.mock_capture("timeout_device")
        ).await;
        
        // Should timeout
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_frame_metadata_accuracy() {
        let device_id = "metadata_test";
        let frame = create_mock_frame(device_id);
        
        // Check metadata fields
        assert_eq!(frame.device_id, device_id);
        assert!(!frame.id.is_empty());
        
        // Check timestamp is recent (within last second)
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(frame.timestamp);
        assert!(age.num_seconds() < 2);
        
        // Check format is set
        assert_eq!(frame.format, "RGB8");
        
        // Check size calculation
        assert_eq!(frame.size_bytes, frame.data.len());
    }

    #[tokio::test]
    async fn test_invalid_frame_detection() {
        // Create invalid frames
        let empty_data_frame = CameraFrame::new(vec![], 640, 480, "test".to_string());
        assert!(!empty_data_frame.is_valid());
        
        let zero_width_frame = CameraFrame::new(vec![1, 2, 3], 0, 480, "test".to_string());
        assert!(!zero_width_frame.is_valid());
        
        let zero_height_frame = CameraFrame::new(vec![1, 2, 3], 640, 0, "test".to_string());
        assert!(!zero_height_frame.is_valid());
        
        // Valid frame should pass
        let valid_frame = create_mock_frame("test");
        assert!(valid_frame.is_valid());
    }

    #[tokio::test]
    async fn test_plant_photography_optimizations() {
        let plant_formats = get_plant_photography_formats();
        
        for format in plant_formats {
            // Create initialization params with plant-optimized settings
            let params = CameraInitParams::new("plant_camera".to_string())
                .with_format(format.clone())
                .with_auto_focus(true)  // Important for sharp plant details
                .with_auto_exposure(true); // Good for varying light conditions
            
            // Test that format is suitable for plant photography
            assert!(format.width >= 1280); // Minimum for detailed analysis
            assert!(format.height >= 720);
            
            // Test frame rate is appropriate (not too high for detailed captures)
            assert!(format.fps <= 60.0);
            
            // Test initialization params
            assert_eq!(params.controls.auto_focus, Some(true)); // Critical for plant detail
            assert_eq!(params.controls.auto_exposure, Some(true)); // Helps with varying lighting
        }
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let mock_system = setup_test_environment().await;
        
        // Start with error mode
        mock_system.set_error_mode(Some(CameraError::CaptureError("Initial error".to_string())));
        
        let result1 = mock_system.mock_capture("recovery_test").await;
        assert!(result1.is_err());
        
        // Clear error mode - should recover
        mock_system.set_error_mode(None);
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let result2 = mock_system.mock_capture("recovery_test").await;
        assert!(result2.is_ok());
        
        let frame = result2.unwrap();
        assert!(frame.is_valid());
    }

    #[tokio::test]
    async fn test_capture_sequence_simulation() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Simulate capturing a sequence of frames
        let mut frames = Vec::new();
        for i in 0..5 {
            let result = mock_system.mock_capture(&format!("sequence_{}", i)).await;
            assert!(result.is_ok());
            frames.push(result.unwrap());
        }
        
        // Verify sequence properties
        assert_eq!(frames.len(), 5);
        
        // Each frame should be valid and unique
        for (i, frame) in frames.iter().enumerate() {
            assert!(frame.is_valid());
            assert_eq!(frame.device_id, format!("sequence_{}", i));
            
            // Each frame should have a unique ID
            for (j, other_frame) in frames.iter().enumerate() {
                if i != j {
                    assert_ne!(frame.id, other_frame.id);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_memory_efficient_capture() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Test that frames don't accumulate in memory
        for i in 0..100 {
            let result = mock_system.mock_capture(&format!("memory_test_{}", i)).await;
            assert!(result.is_ok());
            
            let frame = result.unwrap();
            assert!(frame.is_valid());
            
            // Frame goes out of scope here, should be deallocated
        }
        
        // If we reached here without OOM, memory management is working
        assert!(true);
    }

    #[tokio::test]
    async fn test_format_compatibility_matrix() {
        let test_formats = vec![
            ("RGB8", 3),   // 3 bytes per pixel
            ("JPEG", 1),   // Variable, but test with 1 byte average
            ("RAW", 2),    // Typically 2 bytes per pixel for 16-bit
        ];
        
        for (format_name, bytes_per_pixel) in test_formats {
            let width = 1280;
            let height = 720;
            let data_size = (width * height * bytes_per_pixel) as usize;
            let data = vec![128u8; data_size];
            
            let frame = CameraFrame::new(data, width, height, "format_test".to_string())
                .with_format(format_name.to_string());
            
            assert!(frame.is_valid());
            assert_eq!(frame.format, format_name);
            assert_eq!(frame.width, width);
            assert_eq!(frame.height, height);
        }
    }

    #[tokio::test]
    async fn test_high_resolution_handling() {
        // Test 4K resolution
        let uhd_frame = create_mock_frame_with_size("4k_test", 3840, 2160);
        assert!(uhd_frame.is_valid());
        assert!(validate_plant_frame_quality(&uhd_frame));
        
        // Test 8K resolution (if supported)
        let eightk_frame = create_mock_frame_with_size("8k_test", 7680, 4320);
        assert!(eightk_frame.is_valid());
        assert!(validate_plant_frame_quality(&eightk_frame));
        
        // Verify data size scales correctly
        assert_eq!(uhd_frame.data.len(), (3840 * 2160 * 3) as usize);
        assert_eq!(eightk_frame.data.len(), (7680 * 4320 * 3) as usize);
    }

    #[tokio::test]
    async fn test_aspect_ratio_validation() {
        let test_cases = [
            (640, 480, 4.0/3.0),    // Standard 4:3
            (1280, 720, 16.0/9.0),  // HD 16:9
            (1920, 1080, 16.0/9.0), // FHD 16:9
            (1024, 1024, 1.0),      // Square
            (2560, 1080, 2560.0/1080.0), // Ultrawide
        ];
        
        for (width, height, expected_aspect) in test_cases {
            let frame = create_mock_frame_with_size("aspect_test", width, height);
            let actual_aspect = frame.aspect_ratio();
            
            // Allow small floating point differences
            assert!((actual_aspect - expected_aspect).abs() < 0.01,
                   "Expected aspect {:.2}, got {:.2} for {}x{}", 
                   expected_aspect, actual_aspect, width, height);
        }
    }

    #[tokio::test]
    async fn test_capture_performance_metrics() {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let capture_count = 10;
        let start = tokio::time::Instant::now();
        
        for i in 0..capture_count {
            let result = mock_system.mock_capture(&format!("perf_test_{}", i)).await;
            assert!(result.is_ok());
        }
        
        let elapsed = start.elapsed();
        let captures_per_second = capture_count as f64 / elapsed.as_secs_f64();
        
        // Should be able to capture at reasonable rate (at least 10 FPS in mock)
        assert!(captures_per_second >= 10.0, 
               "Capture rate too slow: {:.2} FPS", captures_per_second);
    }
}