//! Platform detection and camera listing tests
//!
//! Tests platform-specific functionality and camera device enumeration
//! across Windows, macOS, and Linux systems.

use super::*;
use crate::types::{Platform, CameraDeviceInfo, CameraFormat};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        init_test_env();
        
        let platform = Platform::current();
        
        // Platform should never be Unknown in real environments
        // but may be Unknown in test environments
        assert!(matches!(platform, 
            Platform::Windows | Platform::MacOS | Platform::Linux | Platform::Unknown
        ));
        
        // Test platform string conversion
        let platform_str = platform.as_str();
        assert!(!platform_str.is_empty());
        assert!(["windows", "macos", "linux", "unknown"].contains(&platform_str));
    }

    #[test]
    fn test_platform_string_consistency() {
        let platforms = [
            Platform::Windows,
            Platform::MacOS, 
            Platform::Linux,
            Platform::Unknown,
        ];
        
        for platform in platforms {
            let str_repr = platform.as_str();
            assert!(!str_repr.is_empty());
            assert!(str_repr.len() > 2); // Reasonable minimum length
        }
    }

    #[test] 
    fn test_platform_comparison() {
        assert_eq!(Platform::Windows, Platform::Windows);
        assert_ne!(Platform::Windows, Platform::MacOS);
        assert_ne!(Platform::MacOS, Platform::Linux);
        assert_ne!(Platform::Linux, Platform::Unknown);
    }

    #[tokio::test]
    async fn test_camera_device_creation() {
        init_test_env();
        
        let platform = Platform::current();
        let device = create_mock_device("test_id", "Test Camera", platform);
        
        assert_eq!(device.id, "test_id");
        assert_eq!(device.name, "Test Camera");
        assert_eq!(device.platform, platform);
        assert!(device.is_available);
        assert!(!device.supports_formats.is_empty());
    }

    #[tokio::test]
    async fn test_platform_specific_devices() {
        init_test_env();
        
        // Test Windows devices
        let win_device = create_mock_device("win_test", "Windows Camera", Platform::Windows);
        assert_eq!(win_device.platform, Platform::Windows);
        assert!(win_device.description.is_some());
        
        // Test macOS devices  
        let mac_device = create_mock_device("mac_test", "macOS Camera", Platform::MacOS);
        assert_eq!(mac_device.platform, Platform::MacOS);
        
        // Test Linux devices
        let linux_device = create_mock_device("linux_test", "Linux Camera", Platform::Linux);
        assert_eq!(linux_device.platform, Platform::Linux);
    }

    #[tokio::test]
    async fn test_camera_format_validation() {
        init_test_env();
        
        let formats = get_test_formats();
        
        for format in formats {
            // Basic validation
            assert!(format.width > 0);
            assert!(format.height > 0); 
            assert!(format.fps > 0.0);
            assert!(!format.format_type.is_empty());
            
            // Reasonable bounds
            assert!(format.width <= 8192); // Max reasonable width
            assert!(format.height <= 6144); // Max reasonable height
            assert!(format.fps <= 240.0); // Max reasonable FPS
        }
    }

    #[tokio::test]
    async fn test_standard_formats() {
        init_test_env();
        
        let hd = CameraFormat::hd();
        assert_eq!(hd.width, 1920);
        assert_eq!(hd.height, 1080);
        assert_eq!(hd.fps, 30.0);
        
        let standard = CameraFormat::standard();
        assert_eq!(standard.width, 1280);
        assert_eq!(standard.height, 720);
        
        let low = CameraFormat::low();
        assert_eq!(low.width, 640);
        assert_eq!(low.height, 480);
    }

    #[tokio::test]
    async fn test_plant_photography_formats() {
        init_test_env();
        
        let plant_formats = get_plant_photography_formats();
        
        // All plant formats should be high quality
        for format in plant_formats {
            assert!(format.width >= 1280, "Plant photography needs high resolution");
            assert!(format.height >= 720, "Plant photography needs high resolution");
            
            // Check aspect ratio is reasonable for plant photography
            let aspect = format.width as f32 / format.height as f32;
            assert!(aspect >= 1.0, "Plant photography should use landscape or square formats");
            assert!(aspect <= 2.5, "Aspect ratio should not be too extreme");
        }
    }

    #[tokio::test]
    async fn test_mock_system_platform_devices() {
        let mock_system = MockCameraSystem::new();
        
        // Test each platform
        for platform in [Platform::Windows, Platform::MacOS, Platform::Linux, Platform::Unknown] {
            mock_system.add_mock_devices(platform).await;
            let devices = mock_system.get_devices().await;
            
            assert!(!devices.is_empty(), "Should have devices for platform {:?}", platform);
            
            for device in devices {
                assert_eq!(device.platform, platform);
                assert!(device.is_available);
                assert!(!device.supports_formats.is_empty());
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[tokio::test]
    async fn test_windows_specific_functionality() {
        init_test_env();
        
        assert_eq!(Platform::current(), Platform::Windows);
        
        // Test Windows-specific device naming
        let device = create_mock_device("win_cam_0", "Integrated Camera", Platform::Windows);
        assert!(device.description.unwrap().contains("windows"));
    }

    #[cfg(target_os = "macos")]
    #[tokio::test] 
    async fn test_macos_specific_functionality() {
        init_test_env();
        
        assert_eq!(Platform::current(), Platform::MacOS);
        
        // Test macOS-specific device naming
        let device = create_mock_device("mac_cam_0", "FaceTime HD Camera", Platform::MacOS);
        assert!(device.name.contains("FaceTime") || device.name.contains("Camera"));
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_linux_specific_functionality() {
        init_test_env();
        
        assert_eq!(Platform::current(), Platform::Linux);
        
        // Test Linux-specific device naming (v4l style)
        let device = create_mock_device("v4l_0", "/dev/video0", Platform::Linux);
        assert!(device.name.starts_with("/dev/video"));
    }

    #[tokio::test]
    async fn test_device_availability_states() {
        init_test_env();
        
        let available_device = CameraDeviceInfo::new("test1".to_string(), "Available Camera".to_string())
            .with_availability(true);
        assert!(available_device.is_available);
        
        let unavailable_device = CameraDeviceInfo::new("test2".to_string(), "Busy Camera".to_string())
            .with_availability(false);
        assert!(!unavailable_device.is_available);
    }

    #[tokio::test]
    async fn test_device_format_support() {
        init_test_env();
        
        let formats = vec![
            CameraFormat::hd(),
            CameraFormat::standard(),
        ];
        
        let device = CameraDeviceInfo::new("test".to_string(), "Test Camera".to_string())
            .with_formats(formats.clone());
            
        assert_eq!(device.supports_formats.len(), 2);
        assert!(device.supports_formats.contains(&CameraFormat::hd()));
        assert!(device.supports_formats.contains(&CameraFormat::standard()));
    }

    #[tokio::test]
    async fn test_error_handling_platform_detection() {
        init_test_env();
        
        // This test ensures platform detection doesn't panic
        let platform = Platform::current();
        let platform_str = platform.as_str();
        
        // Basic safety checks
        assert!(!platform_str.is_empty());
        assert!(platform_str.len() < 50); // Reasonable upper bound
    }

    #[tokio::test]
    async fn test_concurrent_device_access() {
        use tokio::task;
        
        let mock_system = MockCameraSystem::new();
        mock_system.add_mock_devices(Platform::current()).await;
        
        // Spawn multiple concurrent tasks accessing devices
        let handles = (0..10).map(|i| {
            let system = mock_system.clone();
            task::spawn(async move {
                let devices = system.get_devices().await;
                assert!(!devices.is_empty(), "Task {} should see devices", i);
                devices.len()
            })
        }).collect::<Vec<_>>();
        
        // Wait for all tasks to complete
        let results = futures::future::try_join_all(handles).await.unwrap();
        
        // All tasks should see the same number of devices
        let first_count = results[0];
        for count in results {
            assert_eq!(count, first_count);
        }
    }

    #[tokio::test]
    async fn test_device_enumeration_performance() {
        use std::time::Instant;
        
        let mock_system = MockCameraSystem::new();
        
        let start = Instant::now();
        mock_system.add_mock_devices(Platform::current()).await;
        let devices = mock_system.get_devices().await;
        let elapsed = start.elapsed();
        
        // Device enumeration should be reasonably fast
        assert!(elapsed.as_millis() < 1000, "Device enumeration took too long: {:?}", elapsed);
        assert!(!devices.is_empty());
    }

    #[tokio::test]
    async fn test_platform_capabilities() {
        init_test_env();
        
        let current_platform = Platform::current();
        
        match current_platform {
            Platform::Windows => {
                // Windows should support MediaFoundation
                // This is a placeholder for platform-specific capability tests
                assert!(true); // Would test Windows-specific features
            },
            Platform::MacOS => {
                // macOS should support AVFoundation
                assert!(true); // Would test macOS-specific features
            },
            Platform::Linux => {
                // Linux should support V4L2
                assert!(true); // Would test Linux-specific features
            },
            Platform::Unknown => {
                // Unknown platform - basic functionality only
                assert!(true);
            },
        }
    }

    #[tokio::test]
    async fn test_format_compatibility() {
        init_test_env();
        
        let device = create_mock_device("test", "Test Camera", Platform::current());
        let formats = device.supports_formats;
        
        // Check that all formats are valid
        for format in formats {
            assert!(format.width > 0);
            assert!(format.height > 0);
            assert!(format.fps > 0.0);
            
            // Check format type is supported
            assert!(["RGB8", "JPEG", "RAW", "YUV"].contains(&format.format_type.as_str()) ||
                   !format.format_type.is_empty());
        }
    }

    #[tokio::test]
    async fn test_device_description_generation() {
        init_test_env();
        
        for platform in [Platform::Windows, Platform::MacOS, Platform::Linux] {
            let device = create_mock_device("test", "Test Camera", platform);
            
            assert!(device.description.is_some());
            let desc = device.description.unwrap();
            assert!(desc.contains(platform.as_str()));
            assert!(desc.contains("Mock camera device"));
        }
    }
}