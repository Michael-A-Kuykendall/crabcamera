use crate::tests::{setup_test_environment, init_test_env, MockCameraSystem, MockCaptureMode};
use crate::types::{CameraControls, BurstConfig, ExposureBracketing, WhiteBalance, CameraFormat};
use crate::commands::advanced::*;

#[tokio::test]
async fn test_camera_controls_basic() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    let controls = CameraControls::default();
    
    // Test set camera controls
    let result = set_camera_controls("test_device".to_string(), controls).await;
    assert!(result.is_ok(), "Should be able to set basic camera controls");
}

#[tokio::test]
async fn test_plant_photography_controls() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    let controls = CameraControls::plant_photography();
    
    // Verify plant photography optimizations
    assert_eq!(controls.auto_exposure, Some(false));
    assert_eq!(controls.iso_sensitivity, Some(100));
    assert_eq!(controls.white_balance, Some(WhiteBalance::Daylight));
    assert_eq!(controls.aperture, Some(8.0));
    assert_eq!(controls.contrast, Some(0.3));
    assert_eq!(controls.saturation, Some(0.4));
    assert_eq!(controls.sharpness, Some(0.5));
    
    let result = set_camera_controls("test_device".to_string(), controls).await;
    assert!(result.is_ok(), "Should be able to set plant photography controls");
}

#[tokio::test]
async fn test_manual_focus_controls() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    // Test valid focus distances
    let result = set_manual_focus("test_device".to_string(), 0.5).await;
    assert!(result.is_ok(), "Should accept focus distance 0.5");
    
    let result = set_manual_focus("test_device".to_string(), 0.0).await;
    assert!(result.is_ok(), "Should accept focus distance 0.0 (infinity)");
    
    let result = set_manual_focus("test_device".to_string(), 1.0).await;
    assert!(result.is_ok(), "Should accept focus distance 1.0 (closest)");
    
    // Test invalid focus distances
    let result = set_manual_focus("test_device".to_string(), -0.1).await;
    assert!(result.is_err(), "Should reject negative focus distance");
    
    let result = set_manual_focus("test_device".to_string(), 1.1).await;
    assert!(result.is_err(), "Should reject focus distance > 1.0");
}

#[tokio::test]
async fn test_manual_exposure_controls() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    // Test valid exposure settings
    let result = set_manual_exposure("test_device".to_string(), 1.0/125.0, 400).await;
    assert!(result.is_ok(), "Should accept standard exposure settings");
    
    let result = set_manual_exposure("test_device".to_string(), 1.0/4000.0, 50).await;
    assert!(result.is_ok(), "Should accept fast shutter and low ISO");
    
    let result = set_manual_exposure("test_device".to_string(), 1.0, 3200).await;
    assert!(result.is_ok(), "Should accept slow shutter and high ISO");
    
    // Test invalid exposure settings
    let result = set_manual_exposure("test_device".to_string(), 0.0, 400).await;
    assert!(result.is_err(), "Should reject zero exposure time");
    
    let result = set_manual_exposure("test_device".to_string(), 11.0, 400).await;
    assert!(result.is_err(), "Should reject exposure time > 10 seconds");
    
    let result = set_manual_exposure("test_device".to_string(), 1.0/125.0, 25).await;
    assert!(result.is_err(), "Should reject ISO < 50");
    
    let result = set_manual_exposure("test_device".to_string(), 1.0/125.0, 25600).await;
    assert!(result.is_err(), "Should reject ISO > 12800");
}

#[tokio::test]
async fn test_white_balance_controls() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    // Test different white balance modes
    let wb_modes = vec![
        WhiteBalance::Auto,
        WhiteBalance::Daylight,
        WhiteBalance::Fluorescent,
        WhiteBalance::Incandescent,
        WhiteBalance::Flash,
        WhiteBalance::Cloudy,
        WhiteBalance::Shade,
        WhiteBalance::Custom(5600), // Daylight temperature
    ];
    
    for wb_mode in wb_modes {
        let result = set_white_balance("test_device".to_string(), wb_mode.clone()).await;
        assert!(result.is_ok(), "Should accept white balance mode: {:?}", wb_mode);
    }
}

#[tokio::test]
async fn test_burst_sequence_basic() {
    init_test_env();
    let mock_system = setup_test_environment().await;
    mock_system.set_capture_mode(MockCaptureMode::Success).await;
    
    let config = BurstConfig {
        count: 3,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };
    
    let result = capture_burst_sequence("test_device".to_string(), config).await;
    assert!(result.is_ok(), "Should capture burst sequence");
    
    let frames = result.unwrap();
    assert_eq!(frames.len(), 3, "Should capture 3 frames");
}

#[tokio::test]
async fn test_burst_sequence_invalid_count() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    // Test invalid burst counts
    let config = BurstConfig {
        count: 0,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };
    
    let result = capture_burst_sequence("test_device".to_string(), config).await;
    assert!(result.is_err(), "Should reject burst count of 0");
    
    let config = BurstConfig {
        count: 51,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };
    
    let result = capture_burst_sequence("test_device".to_string(), config).await;
    assert!(result.is_err(), "Should reject burst count > 50");
}

#[tokio::test]
async fn test_hdr_burst_sequence() {
    init_test_env();
    let mock_system = setup_test_environment().await;
    mock_system.set_capture_mode(MockCaptureMode::Success).await;
    
    let result = capture_hdr_sequence("test_device".to_string()).await;
    assert!(result.is_ok(), "Should capture HDR sequence");
    
    let frames = result.unwrap();
    assert_eq!(frames.len(), 3, "HDR should capture 3 frames");
}

#[tokio::test]
async fn test_focus_stack_sequence() {
    init_test_env();
    let mock_system = setup_test_environment().await;
    mock_system.set_capture_mode(MockCaptureMode::Success).await;
    
    let result = capture_focus_stack("test_device".to_string(), 5).await;
    assert!(result.is_ok(), "Should capture focus stack");
    
    let frames = result.unwrap();
    assert_eq!(frames.len(), 5, "Focus stack should capture 5 frames");
}

#[tokio::test]
async fn test_focus_stack_invalid_count() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    // Test invalid focus stack counts
    let result = capture_focus_stack("test_device".to_string(), 2).await;
    assert!(result.is_err(), "Should reject focus stack count < 3");
    
    let result = capture_focus_stack("test_device".to_string(), 21).await;
    assert!(result.is_err(), "Should reject focus stack count > 20");
}

#[tokio::test]
async fn test_exposure_bracketing() {
    init_test_env();
    let mock_system = setup_test_environment().await;
    mock_system.set_capture_mode(MockCaptureMode::Success).await;
    
    let config = BurstConfig {
        count: 3,
        interval_ms: 200,
        bracketing: Some(ExposureBracketing {
            stops: vec![-1.0, 0.0, 1.0],
            base_exposure: 1.0/125.0,
        }),
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };
    
    let result = capture_burst_sequence("test_device".to_string(), config).await;
    assert!(result.is_ok(), "Should capture exposure bracketed sequence");
    
    let frames = result.unwrap();
    assert_eq!(frames.len(), 3, "Should capture 3 bracketed frames");
}

#[tokio::test]
async fn test_plant_optimization() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    let result = optimize_for_plants("test_device".to_string()).await;
    assert!(result.is_ok(), "Should optimize camera for plant photography");
}

#[tokio::test]
async fn test_camera_capabilities() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    let result = test_camera_capabilities("test_device".to_string()).await;
    assert!(result.is_ok(), "Should test camera capabilities");
    
    let capabilities = result.unwrap();
    assert!(capabilities.supports_auto_focus, "Should support auto focus");
    assert!(capabilities.max_resolution.0 > 0, "Should have valid max resolution width");
    assert!(capabilities.max_resolution.1 > 0, "Should have valid max resolution height");
    assert!(capabilities.max_fps > 0.0, "Should have valid max FPS");
}

#[tokio::test]
async fn test_performance_metrics() {
    init_test_env();
    let _mock_system = setup_test_environment().await;
    
    let result = get_camera_performance("test_device".to_string()).await;
    assert!(result.is_ok(), "Should get camera performance metrics");
    
    let metrics = result.unwrap();
    assert!(metrics.capture_latency_ms >= 0.0, "Latency should be non-negative");
    assert!(metrics.fps_actual >= 0.0, "FPS should be non-negative");
    assert!(metrics.quality_score >= 0.0, "Quality score should be non-negative");
}

#[tokio::test]
async fn test_controls_validation() {
    init_test_env();
    
    let controls = CameraControls {
        auto_focus: Some(true),
        focus_distance: Some(0.5),
        auto_exposure: Some(false),
        exposure_time: Some(1.0/125.0),
        iso_sensitivity: Some(400),
        white_balance: Some(WhiteBalance::Daylight),
        aperture: Some(5.6),
        zoom: Some(1.0),
        brightness: Some(0.1),
        contrast: Some(0.2),
        saturation: Some(0.1),
        sharpness: Some(0.3),
        noise_reduction: Some(true),
        image_stabilization: Some(true),
    };
    
    // Verify all control values are within expected ranges
    assert!(controls.focus_distance.unwrap() >= 0.0 && controls.focus_distance.unwrap() <= 1.0);
    assert!(controls.exposure_time.unwrap() > 0.0);
    assert!(controls.iso_sensitivity.unwrap() >= 50);
    assert!(controls.aperture.unwrap() > 0.0);
    assert!(controls.zoom.unwrap() > 0.0);
    assert!(controls.brightness.unwrap() >= -1.0 && controls.brightness.unwrap() <= 1.0);
    assert!(controls.contrast.unwrap() >= -1.0 && controls.contrast.unwrap() <= 1.0);
    assert!(controls.saturation.unwrap() >= -1.0 && controls.saturation.unwrap() <= 1.0);
    assert!(controls.sharpness.unwrap() >= -1.0 && controls.sharpness.unwrap() <= 1.0);
}

#[tokio::test]
async fn test_burst_config_validation() {
    init_test_env();
    
    // Test HDR burst configuration
    let hdr_config = BurstConfig::hdr_burst();
    assert_eq!(hdr_config.count, 3);
    assert!(hdr_config.bracketing.is_some());
    assert!(!hdr_config.focus_stacking);
    assert!(hdr_config.auto_save);
    
    let bracketing = hdr_config.bracketing.unwrap();
    assert_eq!(bracketing.stops.len(), 3);
    assert_eq!(bracketing.stops, vec![-1.0, 0.0, 1.0]);
    assert!(bracketing.base_exposure > 0.0);
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::Instant;
    
    #[tokio::test]
    async fn test_burst_capture_performance() {
        init_test_env();
        let mock_system = setup_test_environment().await;
        mock_system.set_capture_mode(MockCaptureMode::Success).await;
        
        let config = BurstConfig {
            count: 10,
            interval_ms: 50, // Fast burst
            bracketing: None,
            focus_stacking: false,
            auto_save: false,
            save_directory: None,
        };
        
        let start_time = Instant::now();
        let result = capture_burst_sequence("test_device".to_string(), config).await;
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Should capture burst sequence");
        
        let frames = result.unwrap();
        assert_eq!(frames.len(), 10, "Should capture 10 frames");
        
        // Performance expectation: should complete within reasonable time
        // 10 frames * 50ms intervals + processing overhead should be < 2 seconds
        assert!(elapsed.as_millis() < 2000, "Burst capture should complete within 2 seconds");
        
        log::info!("Burst capture performance: {} frames in {:?} ({:.2} fps)", 
            frames.len(), elapsed, frames.len() as f32 / elapsed.as_secs_f32());
    }
    
    #[tokio::test]
    async fn test_controls_application_speed() {
        init_test_env();
        let _mock_system = setup_test_environment().await;
        
        let controls = CameraControls::plant_photography();
        
        let start_time = Instant::now();
        let result = set_camera_controls("test_device".to_string(), controls).await;
        let elapsed = start_time.elapsed();
        
        assert!(result.is_ok(), "Should apply camera controls");
        
        // Performance expectation: controls should apply quickly
        assert!(elapsed.as_millis() < 100, "Controls should apply within 100ms");
        
        log::info!("Controls application time: {:?}", elapsed);
    }
}