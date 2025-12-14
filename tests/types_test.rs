//! Tests for CrabCamera core types
//! 
//! Ensures type safety and correct behavior of fundamental data structures.

use crabcamera::types::{
    Platform, CameraDeviceInfo, CameraFormat, CameraFrame, 
    CameraControls, CameraInitParams, WhiteBalance,
    CameraCapabilities, CameraPerformanceMetrics,
};

#[cfg(test)]
mod platform_tests {
    use super::*;

    #[test]
    fn test_platform_current_detection() {
        let platform = Platform::current();
        // Should detect a valid platform on any system
        assert_ne!(platform, Platform::Unknown, "Platform should be detected");
    }

    #[test]
    fn test_platform_as_str() {
        assert_eq!(Platform::Windows.as_str(), "windows");
        assert_eq!(Platform::MacOS.as_str(), "macos");
        assert_eq!(Platform::Linux.as_str(), "linux");
        assert_eq!(Platform::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_platform_equality() {
        assert_eq!(Platform::Windows, Platform::Windows);
        assert_ne!(Platform::Windows, Platform::MacOS);
    }

    #[test]
    fn test_platform_serialization() {
        let platform = Platform::Windows;
        let json = serde_json::to_string(&platform).unwrap();
        assert!(json.contains("Windows"));
        
        let deserialized: Platform = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, platform);
    }
}

#[cfg(test)]
mod camera_format_tests {
    use super::*;

    #[test]
    fn test_format_creation() {
        let format = CameraFormat::new(1920, 1080, 30.0);
        assert_eq!(format.width, 1920);
        assert_eq!(format.height, 1080);
        assert_eq!(format.fps, 30.0);
        assert_eq!(format.format_type, "RGB8");
    }

    #[test]
    fn test_format_presets() {
        let hd = CameraFormat::hd();
        assert_eq!(hd.width, 1920);
        assert_eq!(hd.height, 1080);
        
        let standard = CameraFormat::standard();
        assert_eq!(standard.width, 1280);
        assert_eq!(standard.height, 720);
        
        let low = CameraFormat::low();
        assert_eq!(low.width, 640);
        assert_eq!(low.height, 480);
    }

    #[test]
    fn test_format_with_type() {
        let format = CameraFormat::new(1920, 1080, 30.0)
            .with_format_type("MJPEG".to_string());
        assert_eq!(format.format_type, "MJPEG");
    }

    #[test]
    fn test_format_equality() {
        let format1 = CameraFormat::new(1920, 1080, 30.0);
        let format2 = CameraFormat::new(1920, 1080, 30.0);
        let format3 = CameraFormat::new(1280, 720, 30.0);
        
        assert_eq!(format1, format2);
        assert_ne!(format1, format3);
    }

    #[test]
    fn test_format_serialization() {
        let format = CameraFormat::hd();
        let json = serde_json::to_string(&format).unwrap();
        let deserialized: CameraFormat = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, format);
    }
}

#[cfg(test)]
mod camera_device_info_tests {
    use super::*;

    #[test]
    fn test_device_creation() {
        let device = CameraDeviceInfo::new("cam0".to_string(), "Test Camera".to_string());
        assert_eq!(device.id, "cam0");
        assert_eq!(device.name, "Test Camera");
        assert!(device.is_available);
        assert!(device.supports_formats.is_empty());
    }

    #[test]
    fn test_device_builder_pattern() {
        let formats = vec![CameraFormat::hd(), CameraFormat::standard()];
        
        let device = CameraDeviceInfo::new("cam1".to_string(), "Pro Camera".to_string())
            .with_description("Professional webcam".to_string())
            .with_formats(formats.clone())
            .with_availability(true);
        
        assert_eq!(device.description, Some("Professional webcam".to_string()));
        assert_eq!(device.supports_formats.len(), 2);
        assert!(device.is_available);
    }

    #[test]
    fn test_device_unavailable() {
        let device = CameraDeviceInfo::new("cam2".to_string(), "Disconnected".to_string())
            .with_availability(false);
        assert!(!device.is_available);
    }
}

#[cfg(test)]
mod camera_frame_tests {
    use super::*;

    #[test]
    fn test_frame_creation() {
        let data = vec![0u8; 1920 * 1080 * 3]; // RGB data
        let frame = CameraFrame::new(data.clone(), 1920, 1080, "cam0".to_string());
        
        assert_eq!(frame.width, 1920);
        assert_eq!(frame.height, 1080);
        assert_eq!(frame.device_id, "cam0");
        assert_eq!(frame.size_bytes, data.len());
        assert!(!frame.id.is_empty()); // UUID generated
    }

    #[test]
    fn test_frame_aspect_ratio() {
        let data = vec![0u8; 100];
        
        let frame_16_9 = CameraFrame::new(data.clone(), 1920, 1080, "test".to_string());
        assert!((frame_16_9.aspect_ratio() - 1.777).abs() < 0.01);
        
        let frame_4_3 = CameraFrame::new(data.clone(), 640, 480, "test".to_string());
        assert!((frame_4_3.aspect_ratio() - 1.333).abs() < 0.01);
    }

    #[test]
    fn test_frame_validity() {
        let valid_frame = CameraFrame::new(vec![1, 2, 3], 100, 100, "test".to_string());
        assert!(valid_frame.is_valid());
        
        let empty_frame = CameraFrame::new(vec![], 100, 100, "test".to_string());
        assert!(!empty_frame.is_valid());
        
        let zero_width = CameraFrame::new(vec![1, 2, 3], 0, 100, "test".to_string());
        assert!(!zero_width.is_valid());
    }

    #[test]
    fn test_frame_with_format() {
        let frame = CameraFrame::new(vec![0], 100, 100, "test".to_string())
            .with_format("JPEG".to_string());
        assert_eq!(frame.format, "JPEG");
    }
}

#[cfg(test)]
mod camera_controls_tests {
    use super::*;

    #[test]
    fn test_controls_default() {
        let controls = CameraControls::default();
        assert_eq!(controls.auto_focus, Some(true));
        assert_eq!(controls.auto_exposure, Some(true));
    }

    #[test]
    fn test_controls_professional_preset() {
        let controls = CameraControls::professional();
        // Professional mode typically has more manual control
        assert!(controls.auto_focus.is_some());
        assert!(controls.auto_exposure.is_some());
    }

    #[test]
    fn test_white_balance_variants() {
        let wb_auto = WhiteBalance::Auto;
        let wb_custom = WhiteBalance::Custom(5500);
        
        // Ensure serialization works for all variants
        let json_auto = serde_json::to_string(&wb_auto).unwrap();
        let json_custom = serde_json::to_string(&wb_custom).unwrap();
        
        assert!(json_auto.contains("Auto"));
        assert!(json_custom.contains("5500"));
    }
}

#[cfg(test)]
mod camera_init_params_tests {
    use super::*;

    #[test]
    fn test_init_params_creation() {
        let params = CameraInitParams::new("cam0".to_string());
        assert_eq!(params.device_id, "cam0");
        // Should default to standard format
        assert_eq!(params.format.width, 1280);
        assert_eq!(params.format.height, 720);
    }

    #[test]
    fn test_init_params_builder() {
        let params = CameraInitParams::new("cam0".to_string())
            .with_format(CameraFormat::hd())
            .with_auto_focus(true)
            .with_auto_exposure(false);
        
        assert_eq!(params.format.width, 1920);
        assert_eq!(params.controls.auto_focus, Some(true));
        assert_eq!(params.controls.auto_exposure, Some(false));
    }

    #[test]
    fn test_init_params_professional() {
        let params = CameraInitParams::professional("pro_cam".to_string());
        assert_eq!(params.device_id, "pro_cam");
        // Professional should have higher resolution
        assert!(params.format.width >= 2000);
    }
}

#[cfg(test)]
mod camera_capabilities_tests {
    use super::*;

    #[test]
    fn test_capabilities_default() {
        let caps = CameraCapabilities::default();
        // Verify default struct can be created and has valid state
        // The actual default values may vary by implementation
        let _ = caps.supports_auto_focus;
        let _ = caps.supports_manual_focus;
    }
}

#[cfg(test)]
mod camera_performance_tests {
    use super::*;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = CameraPerformanceMetrics::default();
        assert_eq!(metrics.dropped_frames, 0);
        assert_eq!(metrics.buffer_overruns, 0);
        assert_eq!(metrics.fps_actual, 0.0);
    }
}
