//! Tests for CrabCamera core types
//!
//! Ensures type safety and correct behavior of fundamental data structures.

use crabcamera::types::{
    BurstConfig, CameraCapabilities, CameraControls, CameraDeviceInfo, CameraFormat, CameraFrame,
    CameraInitParams, CameraPerformanceMetrics, ExposureBracketing, FrameMetadata, Platform,
    WhiteBalance,
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
        let format = CameraFormat::new(1920, 1080, 30.0).with_format_type("MJPEG".to_string());
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
        let frame =
            CameraFrame::new(vec![0], 100, 100, "test".to_string()).with_format("JPEG".to_string());
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

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = CameraPerformanceMetrics {
            capture_latency_ms: 16.67,
            processing_time_ms: 5.5,
            memory_usage_mb: 128.5,
            fps_actual: 59.94,
            dropped_frames: 3,
            buffer_overruns: 1,
            quality_score: 0.95,
        };

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: CameraPerformanceMetrics = serde_json::from_str(&json).unwrap();

        assert!(
            (deserialized.capture_latency_ms - metrics.capture_latency_ms).abs() < f32::EPSILON
        );
        assert!(
            (deserialized.processing_time_ms - metrics.processing_time_ms).abs() < f32::EPSILON
        );
        assert!((deserialized.memory_usage_mb - metrics.memory_usage_mb).abs() < f32::EPSILON);
        assert!((deserialized.fps_actual - metrics.fps_actual).abs() < f32::EPSILON);
        assert_eq!(deserialized.dropped_frames, metrics.dropped_frames);
        assert_eq!(deserialized.buffer_overruns, metrics.buffer_overruns);
        assert!((deserialized.quality_score - metrics.quality_score).abs() < f32::EPSILON);
    }
}

#[cfg(test)]
mod burst_config_tests {
    use super::*;

    #[test]
    fn test_burst_config_hdr_preset() {
        let hdr_burst = BurstConfig::hdr_burst();

        assert_eq!(hdr_burst.count, 3);
        assert_eq!(hdr_burst.interval_ms, 200);
        assert!(hdr_burst.bracketing.is_some());
        assert!(!hdr_burst.focus_stacking);
        assert!(hdr_burst.auto_save);
        assert_eq!(hdr_burst.save_directory, Some("hdr_captures".to_string()));

        let bracketing = hdr_burst.bracketing.unwrap();
        assert_eq!(bracketing.stops, vec![-1.0, 0.0, 1.0]);
        assert!((bracketing.base_exposure - 1.0 / 125.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_burst_config_custom() {
        let custom_burst = BurstConfig {
            count: 10,
            interval_ms: 100,
            bracketing: None,
            focus_stacking: true,
            auto_save: false,
            save_directory: Some("/custom/path".to_string()),
        };

        assert_eq!(custom_burst.count, 10);
        assert_eq!(custom_burst.interval_ms, 100);
        assert!(custom_burst.bracketing.is_none());
        assert!(custom_burst.focus_stacking);
        assert!(!custom_burst.auto_save);
    }

    #[test]
    fn test_burst_config_serialization() {
        let burst = BurstConfig::hdr_burst();

        let json = serde_json::to_string(&burst).unwrap();
        let deserialized: BurstConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.count, burst.count);
        assert_eq!(deserialized.interval_ms, burst.interval_ms);
        assert_eq!(deserialized.focus_stacking, burst.focus_stacking);
        assert_eq!(deserialized.auto_save, burst.auto_save);
        assert_eq!(deserialized.save_directory, burst.save_directory);

        // Compare bracketing if present
        if let (Some(orig), Some(deser)) = (&burst.bracketing, &deserialized.bracketing) {
            assert_eq!(orig.stops, deser.stops);
            assert!((orig.base_exposure - deser.base_exposure).abs() < f32::EPSILON);
        }
    }

    #[test]
    fn test_exposure_bracketing_custom() {
        let custom_bracketing = ExposureBracketing {
            stops: vec![-2.0, -1.0, 0.0, 1.0, 2.0], // 5-shot HDR
            base_exposure: 1.0 / 60.0,
        };

        assert_eq!(custom_bracketing.stops.len(), 5);
        assert_eq!(custom_bracketing.stops[0], -2.0);
        assert_eq!(custom_bracketing.stops[4], 2.0);
        assert!((custom_bracketing.base_exposure - 1.0 / 60.0).abs() < f32::EPSILON);

        // Test serialization
        let json = serde_json::to_string(&custom_bracketing).unwrap();
        let deserialized: ExposureBracketing = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.stops, custom_bracketing.stops);
    }

    #[test]
    fn test_burst_config_edge_cases() {
        // Single shot "burst"
        let single_burst = BurstConfig {
            count: 1,
            interval_ms: 0,
            bracketing: None,
            focus_stacking: false,
            auto_save: true,
            save_directory: None,
        };

        assert_eq!(single_burst.count, 1);
        assert_eq!(single_burst.interval_ms, 0);

        // Very long burst
        let long_burst = BurstConfig {
            count: u32::MAX,
            interval_ms: u32::MAX,
            bracketing: Some(ExposureBracketing {
                stops: vec![f32::MIN, 0.0, f32::MAX],
                base_exposure: f32::EPSILON,
            }),
            focus_stacking: true,
            auto_save: true,
            save_directory: Some("x".repeat(1000)),
        };

        assert_eq!(long_burst.count, u32::MAX);
        assert_eq!(long_burst.save_directory.as_ref().unwrap().len(), 1000);
    }
}

#[cfg(test)]
mod frame_metadata_tests {
    use super::*;

    #[test]
    fn test_frame_metadata_default() {
        let metadata = FrameMetadata::default();

        assert!(metadata.exposure_time.is_none());
        assert!(metadata.iso_sensitivity.is_none());
        assert!(metadata.white_balance.is_none());
        assert!(metadata.focus_distance.is_none());
        assert!(metadata.aperture.is_none());
        assert!(metadata.flash_fired.is_none());
        assert!(metadata.scene_mode.is_none());
        assert!(metadata.capture_settings.is_none());
    }

    #[test]
    fn test_frame_metadata_complete() {
        let metadata = FrameMetadata {
            exposure_time: Some(1.0 / 125.0),
            iso_sensitivity: Some(800),
            white_balance: Some(WhiteBalance::Daylight),
            focus_distance: Some(0.5),
            aperture: Some(5.6),
            flash_fired: Some(true),
            scene_mode: Some("Portrait".to_string()),
            capture_settings: Some(CameraControls::professional()),
        };

        assert!(metadata.exposure_time.is_some());
        assert_eq!(metadata.iso_sensitivity, Some(800));
        assert_eq!(metadata.white_balance, Some(WhiteBalance::Daylight));
        assert_eq!(metadata.focus_distance, Some(0.5));
        assert_eq!(metadata.aperture, Some(5.6));
        assert_eq!(metadata.flash_fired, Some(true));
        assert_eq!(metadata.scene_mode, Some("Portrait".to_string()));
        assert!(metadata.capture_settings.is_some());
    }

    #[test]
    fn test_frame_metadata_serialization() {
        let metadata = FrameMetadata {
            exposure_time: Some(0.004), // 1/250s
            iso_sensitivity: Some(1600),
            white_balance: Some(WhiteBalance::Custom(5200)),
            focus_distance: Some(0.75),
            aperture: Some(2.8),
            flash_fired: Some(false),
            scene_mode: Some("Night".to_string()),
            capture_settings: Some(CameraControls::default()),
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: FrameMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.exposure_time, metadata.exposure_time);
        assert_eq!(deserialized.iso_sensitivity, metadata.iso_sensitivity);
        assert_eq!(deserialized.white_balance, metadata.white_balance);
        assert_eq!(deserialized.focus_distance, metadata.focus_distance);
        assert_eq!(deserialized.aperture, metadata.aperture);
        assert_eq!(deserialized.flash_fired, metadata.flash_fired);
        assert_eq!(deserialized.scene_mode, metadata.scene_mode);
    }

    #[test]
    fn test_frame_metadata_debug_clone() {
        let metadata = FrameMetadata {
            exposure_time: Some(1.0 / 60.0),
            iso_sensitivity: Some(400),
            white_balance: Some(WhiteBalance::Auto),
            focus_distance: None,
            aperture: None,
            flash_fired: Some(false),
            scene_mode: Some("Auto".to_string()),
            capture_settings: None,
        };

        let cloned = metadata.clone();
        let debug_str = format!("{:?}", metadata);

        assert_eq!(cloned.exposure_time, metadata.exposure_time);
        assert_eq!(cloned.iso_sensitivity, metadata.iso_sensitivity);
        assert!(debug_str.contains("FrameMetadata"));
    }
}

#[cfg(test)]
mod thread_safety_tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_types_send_sync() {
        // Verify that our types implement Send + Sync for thread safety
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<Platform>();
        assert_send_sync::<CameraFormat>();
        assert_send_sync::<CameraDeviceInfo>();
        assert_send_sync::<CameraFrame>();
        assert_send_sync::<CameraControls>();
        assert_send_sync::<WhiteBalance>();
        assert_send_sync::<CameraCapabilities>();
        assert_send_sync::<CameraPerformanceMetrics>();
        assert_send_sync::<BurstConfig>();
        assert_send_sync::<ExposureBracketing>();
        assert_send_sync::<FrameMetadata>();
        assert_send_sync::<CameraInitParams>();
    }

    #[test]
    fn test_concurrent_serialization() {
        let format = Arc::new(CameraFormat::hd());
        let mut handles = vec![];

        // Spawn multiple threads to serialize the same format concurrently
        for i in 0..10 {
            let format_clone = format.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let json = serde_json::to_string(&*format_clone).unwrap();
                    let _deserialized: CameraFormat = serde_json::from_str(&json).unwrap();
                }
                i
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_frame_creation() {
        let mut handles = vec![];

        // Create frames concurrently to test UUID uniqueness under load
        for i in 0..5 {
            let handle = thread::spawn(move || {
                let mut ids = std::collections::HashSet::new();
                for j in 0..200 {
                    let frame = CameraFrame::new(
                        vec![i as u8, j as u8],
                        10,
                        10,
                        format!("thread_{}_frame_{}", i, j),
                    );
                    ids.insert(frame.id);
                }
                ids
            });
            handles.push(handle);
        }

        // Collect all IDs and verify uniqueness
        let mut all_ids = std::collections::HashSet::new();
        for handle in handles {
            let ids = handle.join().unwrap();
            for id in ids {
                assert!(
                    all_ids.insert(id.clone()),
                    "UUID collision detected: {}",
                    id
                );
            }
        }

        // Should have 5 threads * 200 frames = 1000 unique IDs
        assert_eq!(all_ids.len(), 1000);
    }
}
