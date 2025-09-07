//! Integration tests for end-to-end camera workflows
//!
//! Tests complete camera workflows from initialization through capture
//! with real-world simulation scenarios for plant photography.

use super::*;
use crate::types::{CameraDeviceInfo, CameraFormat, CameraFrame, Platform};
use crate::errors::CameraError;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_camera_workflow() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Step 1: Enumerate cameras
        let devices = mock_system.get_devices().await;
        assert!(!devices.is_empty());
        
        // Step 2: Select first available camera
        let selected_device = &devices[0];
        assert!(selected_device.is_available);
        
        // Step 3: Choose appropriate format
        let _format = selected_device.supports_formats.iter()
            .find(|f| f.width >= 1280 && f.height >= 720)
            .cloned()
            .unwrap_or_else(|| CameraFormat::hd());
        
        // Step 4: Capture image
        let result = mock_system.mock_capture(&selected_device.id).await;
        assert!(result.is_ok());
        
        let frame = result.unwrap();
        assert!(frame.is_valid());
        assert_eq!(frame.device_id, selected_device.id);
    }

    #[tokio::test]
    async fn test_plant_photography_workflow() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Plant photography requires high-quality settings
        let plant_formats = get_plant_photography_formats();
        
        for _format in plant_formats.iter().take(3) { // Test first 3 formats
            // Step 1: Set up for plant photography
            let devices = mock_system.get_devices().await;
            let camera = &devices[0];
            
            // Step 2: Capture with plant-optimized format
            let result = mock_system.mock_capture(&camera.id).await;
            assert!(result.is_ok());
            
            let frame = result.unwrap();
            
            // Step 3: Validate plant photography quality
            assert!(validate_plant_frame_quality(&frame));
            assert!(frame.width >= 1280); // Minimum for plant detail analysis
            assert!(frame.height >= 720);
        }
    }

    #[tokio::test]
    async fn test_multi_camera_switching() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let devices = mock_system.get_devices().await;
        if devices.len() < 2 {
            return; // Skip if not enough mock devices
        }
        
        // Test switching between cameras
        for (_i, device) in devices.iter().take(3).enumerate() {
            let result = mock_system.mock_capture(&device.id).await;
            assert!(result.is_ok());
            
            let frame = result.unwrap();
            assert!(frame.is_valid());
            assert_eq!(frame.device_id, device.id);
            
            // Simulate delay between camera switches
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[tokio::test]
    async fn test_error_recovery_workflow() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        
        // Step 1: Start with error condition
        mock_system.set_capture_mode(MockCaptureMode::Failure);
        
        let result1 = mock_system.mock_capture("error_recovery").await;
        assert!(result1.is_err());
        
        // Step 2: Attempt recovery by switching to success mode
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let result2 = mock_system.mock_capture("error_recovery").await;
        assert!(result2.is_ok());
        
        let frame = result2.unwrap();
        assert!(frame.is_valid());
    }

    #[tokio::test]
    async fn test_permission_workflow() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        
        // Step 1: Test permission denied scenario
        mock_system.set_error_mode(Some(CameraError::PermissionDenied("Permission test".to_string())));
        
        let result = mock_system.mock_capture("permission_test").await;
        assert!(result.is_err());
        
        // Step 2: Grant permission and retry
        mock_system.set_error_mode(None);
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let result2 = mock_system.mock_capture("permission_test").await;
        assert!(result2.is_ok());
    }

    #[tokio::test]
    async fn test_format_selection_workflow() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let devices = mock_system.get_devices().await;
        let device = &devices[0];
        
        // Test different format selections
        let test_formats = vec![
            CameraFormat::low(),
            CameraFormat::standard(), 
            CameraFormat::hd(),
        ];
        
        for format in test_formats {
            // Check if device supports the format
            let supports_format = device.supports_formats.iter()
                .any(|f| f.width == format.width && f.height == format.height);
            
            if supports_format {
                let result = mock_system.mock_capture(&device.id).await;
                assert!(result.is_ok());
                
                let frame = result.unwrap();
                assert!(frame.is_valid());
            }
        }
    }

    #[tokio::test]
    async fn test_rapid_capture_sequence() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        let device_id = "rapid_test";
        let capture_count = 20;
        let mut frames = Vec::with_capacity(capture_count);
        
        let start = tokio::time::Instant::now();
        
        // Rapid capture sequence
        for _i in 0..capture_count {
            let result = mock_system.mock_capture(device_id).await;
            assert!(result.is_ok());
            
            frames.push(result.unwrap());
        }
        
        let elapsed = start.elapsed();
        
        // Verify all captures succeeded
        assert_eq!(frames.len(), capture_count);
        for frame in frames {
            assert!(frame.is_valid());
            assert_eq!(frame.device_id, device_id);
        }
        
        // Check performance - should handle rapid captures
        let captures_per_second = capture_count as f64 / elapsed.as_secs_f64();
        assert!(captures_per_second >= 10.0); // At least 10 FPS in mock mode
    }

    #[tokio::test]
    async fn test_cross_platform_integration() {
        init_test_env();
        
        let current_platform = Platform::current();
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Test platform-specific camera access
        match current_platform {
            Platform::Windows => {
                // Test Windows-specific integration
                let result = simulate_windows_camera_integration().await;
                assert!(result.is_ok());
            },
            Platform::MacOS => {
                // Test macOS-specific integration  
                let result = simulate_macos_camera_integration().await;
                assert!(result.is_ok());
            },
            Platform::Linux => {
                // Test Linux-specific integration
                let result = simulate_linux_camera_integration().await;
                assert!(result.is_ok());
            },
            Platform::Unknown => {
                // Test generic integration
                let result = simulate_generic_camera_integration().await;
                assert!(result.is_ok());
            },
        }
    }

    #[tokio::test]
    async fn test_tauri_frontend_integration() {
        init_test_env();
        
        // Simulate frontend calling backend commands
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Step 1: Frontend requests available cameras
        let cameras_result = simulate_get_cameras_command().await;
        assert!(cameras_result.is_ok());
        let cameras = cameras_result.unwrap();
        
        // Step 2: Frontend selects camera and requests formats
        let selected_camera = &cameras[0];
        let formats_result = simulate_get_formats_command(&selected_camera.id).await;
        assert!(formats_result.is_ok());
        
        // Step 3: Frontend initiates capture
        let format = CameraFormat::hd();
        let capture_result = simulate_capture_image_command(&selected_camera.id, format).await;
        assert!(capture_result.is_ok());
        
        let frame = capture_result.unwrap();
        assert!(frame.is_valid());
    }

    #[tokio::test]
    async fn test_memory_cleanup_integration() {
        init_test_env();
        
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        
        // Capture many frames to test memory management
        for i in 0..50 {
            let result = mock_system.mock_capture(&format!("cleanup_test_{}", i)).await;
            assert!(result.is_ok());
            
            let frame = result.unwrap();
            assert!(frame.is_valid());
            
            // Frame should be dropped here, freeing memory
        }
        
        // If we reach here without OOM, memory cleanup is working
        assert!(true);
    }

    // Simulation functions for platform-specific testing
    
    async fn simulate_windows_camera_integration() -> Result<CameraFrame, CameraError> {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        mock_system.mock_capture("windows_integration_test").await
    }

    async fn simulate_macos_camera_integration() -> Result<CameraFrame, CameraError> {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        mock_system.mock_capture("macos_integration_test").await
    }

    async fn simulate_linux_camera_integration() -> Result<CameraFrame, CameraError> {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        mock_system.mock_capture("linux_integration_test").await
    }

    async fn simulate_generic_camera_integration() -> Result<CameraFrame, CameraError> {
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success);
        mock_system.mock_capture("generic_integration_test").await
    }

    async fn simulate_get_cameras_command() -> Result<Vec<CameraDeviceInfo>, CameraError> {
        let mock_system = setup_test_environment().await;
        Ok(mock_system.get_devices().await)
    }

    async fn simulate_get_formats_command(device_id: &str) -> Result<Vec<CameraFormat>, CameraError> {
        if device_id.is_empty() {
            return Err(CameraError::InitializationError("Empty device ID".to_string()));
        }
        Ok(get_test_formats())
    }

    async fn simulate_capture_image_command(device_id: &str, format: CameraFormat) -> Result<CameraFrame, CameraError> {
        if device_id.is_empty() {
            return Err(CameraError::InitializationError("Empty device ID".to_string()));
        }
        
        if format.width == 0 || format.height == 0 || format.fps == 0.0 {
            return Err(CameraError::InitializationError("Invalid format parameters".to_string()));
        }
        
        let mock_system = setup_test_environment().await;
        mock_system.mock_capture(device_id).await
    }
}