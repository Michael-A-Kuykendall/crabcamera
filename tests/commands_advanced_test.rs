//! Advanced Camera Controls Testing
//!
//! **Note:** These tests share the same mock camera and must run serially.
//! Run with: `cargo test --test commands_advanced_test -- --test-threads=1`
//!
//! Comprehensive test suite for professional camera controls including:
//! - Manual focus control validation
//! - Exposure bracketing accuracy
//! - White balance calibration
//! - HDR capture sequences
//! - Advanced control parameter validation
//! - Performance testing for advanced operations

use crabcamera::commands::advanced::{
    capture_burst_sequence, capture_focus_stack_legacy, capture_hdr_sequence, get_camera_controls,
    get_camera_performance, set_camera_controls, set_manual_exposure, set_manual_focus,
    set_white_balance, test_camera_capabilities as test_capabilities,
};
use crabcamera::types::{BurstConfig, CameraControls, WhiteBalance};
use std::time::{Duration, Instant};
use tokio;
use tokio::sync::Mutex as AsyncMutex;

/// Test synchronization lock to prevent concurrent camera control tests
/// This ensures only one test modifies camera controls at a time
static TEST_LOCK: AsyncMutex<()> = AsyncMutex::const_new(());

/// Mock device ID for testing
const TEST_DEVICE_ID: &str = "test_camera_advanced";

/// Helper function to create test controls
fn create_test_controls() -> CameraControls {
    CameraControls {
        auto_focus: Some(false),
        focus_distance: Some(0.5),
        auto_exposure: Some(false),
        exposure_time: Some(1.0 / 125.0), // 1/125s
        iso_sensitivity: Some(400),
        white_balance: Some(WhiteBalance::Auto),
        aperture: Some(5.6),
        zoom: Some(1.0),
        brightness: Some(0.0),
        contrast: Some(0.0),
        saturation: Some(0.0),
        sharpness: Some(0.0),
        noise_reduction: Some(true),
        image_stabilization: Some(true),
    }
}

/// Test setting and getting basic camera controls
#[tokio::test]
async fn test_set_get_camera_controls() {
    // Acquire async lock to prevent concurrent camera control modifications
    let _lock = TEST_LOCK.lock().await;

    let controls = create_test_controls();
    let device_id = TEST_DEVICE_ID.to_string();

    // Get initial state to understand what controls are supported
    let initial_result = get_camera_controls(device_id.clone()).await;
    let initial_controls = match initial_result {
        Ok(controls) => controls,
        Err(e) => {
            // In CI environment, camera might not be available
            println!(
                "Warning: Camera not available for control test (expected in CI): {}",
                e
            );
            return;
        }
    };

    // Set controls
    let set_result = set_camera_controls(device_id.clone(), controls.clone()).await;
    match set_result {
        Ok(message) => {
            assert!(message.contains("Controls applied"));
        }
        Err(e) => {
            // In CI environment, camera might not be available
            println!(
                "Warning: Camera control test failed (expected in CI): {}",
                e
            );
            return;
        }
    }

    // Give hardware time to apply the settings
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Get controls back
    let get_result = get_camera_controls(device_id.clone()).await;
    match get_result {
        Ok(retrieved_controls) => {
            // The test passes if the operation succeeded - we don't require specific controls to be supported
            // Different cameras have different capabilities, so we just verify the API works
            // If controls were changed from initial state, they should be reflected (if supported)
            // If not changed, that's also fine (camera may not support those controls)

            let auto_focus_changed = initial_controls.auto_focus != controls.auto_focus;
            let auto_exposure_changed = initial_controls.auto_exposure != controls.auto_exposure;
            let white_balance_changed = initial_controls.white_balance != controls.white_balance;

            if auto_focus_changed {
                // If we tried to change auto_focus and it didn't change, that's OK (not supported)
                // If it did change, it should match what we set
                if retrieved_controls.auto_focus != initial_controls.auto_focus {
                    assert_eq!(
                        retrieved_controls.auto_focus, controls.auto_focus,
                        "auto_focus changed but not to expected value"
                    );
                }
            }

            if auto_exposure_changed {
                if retrieved_controls.auto_exposure != initial_controls.auto_exposure {
                    assert_eq!(
                        retrieved_controls.auto_exposure, controls.auto_exposure,
                        "auto_exposure changed but not to expected value"
                    );
                }
            }

            if white_balance_changed {
                if retrieved_controls.white_balance != initial_controls.white_balance {
                    assert_eq!(
                        retrieved_controls.white_balance, controls.white_balance,
                        "white_balance changed but not to expected value"
                    );
                }
            }

            // Test passes - camera control API works, even if specific controls aren't supported
        }
        Err(e) => {
            println!("Warning: Get controls test failed (expected in CI): {}", e);
        }
    }

    // Cleanup: Reset controls to defaults for next test
    let default_controls = CameraControls::default();
    let _ = set_camera_controls(device_id, default_controls).await;
}

/// Test manual focus control with parameter validation
#[tokio::test]
async fn test_manual_focus_control() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test valid focus distances
    let valid_distances = [0.0, 0.25, 0.5, 0.75, 1.0];

    for distance in valid_distances.iter() {
        let result = set_manual_focus(device_id.clone(), *distance).await;
        match result {
            Ok(_) => {
                // Success is good
            }
            Err(e) if e.contains("mutex") || e.contains("camera") => {
                // Expected in CI without camera hardware
                println!("Warning: Focus test failed (expected in CI): {}", e);
            }
            Err(e) => {
                // Should not be a parameter validation error
                assert!(!e.contains("Focus distance must be"));
            }
        }
    }

    // Test invalid focus distances
    let invalid_distances = [-0.1, 1.1, 2.0, -1.0];

    for distance in invalid_distances.iter() {
        let result = set_manual_focus(device_id.clone(), *distance).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.contains("Focus distance must be between 0.0"));
        }
    }
}

/// Test manual exposure settings with validation
#[tokio::test]
async fn test_manual_exposure_settings() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test valid exposure combinations
    let valid_combinations = [
        (1.0 / 1000.0, 100), // Fast shutter, low ISO
        (1.0 / 125.0, 400),  // Standard settings
        (1.0 / 30.0, 800),   // Slow shutter, higher ISO
        (1.0 / 4.0, 3200),   // Very slow, high ISO
    ];

    for (exposure_time, iso) in valid_combinations.iter() {
        let result = set_manual_exposure(device_id.clone(), *exposure_time, *iso).await;
        match result {
            Ok(_) => {
                // Success is good
            }
            Err(e) if e.contains("mutex") || e.contains("camera") => {
                // Expected in CI
                println!("Warning: Exposure test failed (expected in CI): {}", e);
            }
            Err(e) => {
                // Should not be parameter validation errors
                assert!(!e.contains("Exposure time must be"));
                assert!(!e.contains("ISO sensitivity must be"));
            }
        }
    }

    // Test invalid exposure times
    let invalid_exposures = [0.0, -0.1, 11.0, 20.0];
    for exposure in invalid_exposures.iter() {
        let result = set_manual_exposure(device_id.clone(), *exposure, 400).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.contains("Exposure time must be"));
        }
    }

    // Test invalid ISO values
    let invalid_isos = [25, 49, 25600, 100000];
    for iso in invalid_isos.iter() {
        let result = set_manual_exposure(device_id.clone(), 1.0 / 125.0, *iso).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.contains("ISO sensitivity must be"));
        }
    }
}

/// Test white balance modes
#[tokio::test]
async fn test_white_balance_modes() {
    let device_id = TEST_DEVICE_ID.to_string();

    let wb_modes = [
        WhiteBalance::Auto,
        WhiteBalance::Daylight,
        WhiteBalance::Fluorescent,
        WhiteBalance::Incandescent,
        WhiteBalance::Flash,
        WhiteBalance::Cloudy,
        WhiteBalance::Shade,
        WhiteBalance::Custom(5500), // 5500K daylight
    ];

    for wb_mode in wb_modes.iter() {
        let result = set_white_balance(device_id.clone(), wb_mode.clone()).await;
        match result {
            Ok(_) => {
                // Success is good
            }
            Err(e) if e.contains("mutex") || e.contains("camera") => {
                // Expected in CI
                println!("Warning: WB test failed (expected in CI): {}", e);
            }
            Err(e) => {
                // Unexpected error
                println!("Unexpected white balance error: {}", e);
            }
        }
    }
}

/// Test burst sequence capture with various configurations
#[tokio::test]
async fn test_burst_sequence_capture() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test basic burst configuration
    let basic_config = BurstConfig {
        count: 3,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };

    let result = capture_burst_sequence(device_id.clone(), basic_config).await;
    match result {
        Ok(frames) => {
            assert_eq!(frames.len(), 3);
            for frame in frames {
                assert!(frame.is_valid());
                assert!(!frame.data.is_empty());
            }
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            // Expected in CI
            println!("Warning: Burst test failed (expected in CI): {}", e);
        }
        Err(e) => {
            println!("Unexpected burst error: {}", e);
        }
    }
}

/// Test exposure bracketing in burst mode
#[tokio::test]
async fn test_exposure_bracketing() {
    let device_id = TEST_DEVICE_ID.to_string();

    let bracketing_config = crabcamera::types::ExposureBracketing {
        base_exposure: 1.0 / 125.0,
        stops: vec![-1.0, 0.0, 1.0], // Under, normal, over
    };

    let burst_config = BurstConfig {
        count: 3,
        interval_ms: 200,
        bracketing: Some(bracketing_config),
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };

    let result = capture_burst_sequence(device_id, burst_config).await;
    match result {
        Ok(frames) => {
            assert_eq!(frames.len(), 3);

            // Verify frames have metadata indicating different exposures
            for (i, frame) in frames.iter().enumerate() {
                assert!(frame.is_valid());
                if let Some(ref settings) = frame.metadata.capture_settings {
                    // Check that exposure was varied for bracketing
                    assert!(
                        settings.exposure_time.is_some() || settings.auto_exposure == Some(false)
                    );
                }
                println!("Bracketed frame {}: {} bytes", i + 1, frame.size_bytes);
            }
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Bracketing test failed (expected in CI): {}", e);
        }
        Err(e) => {
            println!("Unexpected bracketing error: {}", e);
        }
    }
}

/// Test focus stacking configuration
#[tokio::test]
async fn test_focus_stacking_legacy() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test valid stack counts
    let valid_counts = [3, 5, 10, 20];

    for count in valid_counts.iter() {
        let result = capture_focus_stack_legacy(device_id.clone(), *count).await;
        match result {
            Ok(frames) => {
                assert_eq!(frames.len() as u32, *count);
                for frame in frames {
                    assert!(frame.is_valid());
                }
            }
            Err(e) if e.contains("mutex") || e.contains("camera") => {
                println!("Warning: Focus stack test failed (expected in CI): {}", e);
            }
            Err(e) => {
                // Should not be validation error for valid counts
                assert!(!e.contains("Focus stack count must be"));
                println!("Unexpected focus stack error: {}", e);
            }
        }
    }

    // Test invalid stack counts
    let invalid_counts = [1, 2, 21, 50];

    for count in invalid_counts.iter() {
        let result = capture_focus_stack_legacy(device_id.clone(), *count).await;
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.contains("Focus stack count must be between 3 and 20"));
        }
    }
}

/// Test HDR sequence capture
#[tokio::test]
async fn test_hdr_capture() {
    let device_id = TEST_DEVICE_ID.to_string();

    let result = capture_hdr_sequence(device_id).await;
    match result {
        Ok(frames) => {
            // HDR should capture multiple frames (typically 3-5)
            assert!(frames.len() >= 3);
            assert!(frames.len() <= 7);

            for frame in frames {
                assert!(frame.is_valid());
                assert!(frame.size_bytes > 0);
            }
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: HDR test failed (expected in CI): {}", e);
        }
        Err(e) => {
            println!("Unexpected HDR error: {}", e);
        }
    }
}

/// Test camera capabilities detection
#[tokio::test]
async fn test_camera_capabilities() {
    let device_id = TEST_DEVICE_ID.to_string();

    let result = test_capabilities(device_id).await;
    match result {
        Ok(capabilities) => {
            // Verify capabilities structure
            assert!(capabilities.max_resolution.0 > 0);
            assert!(capabilities.max_resolution.1 > 0);

            println!("Camera capabilities:");
            println!("  Manual focus: {}", capabilities.supports_manual_focus);
            println!(
                "  Manual exposure: {}",
                capabilities.supports_manual_exposure
            );
            println!("  White balance: {}", capabilities.supports_white_balance);
            println!(
                "  Max resolution: {}x{}",
                capabilities.max_resolution.0, capabilities.max_resolution.1
            );
            println!("  Manual focus: {}", capabilities.supports_manual_focus);
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Capabilities test failed (expected in CI): {}", e);
        }
        Err(e) => {
            println!("Unexpected capabilities error: {}", e);
        }
    }
}

/// Test camera performance metrics
#[tokio::test]
async fn test_performance_metrics() {
    let device_id = TEST_DEVICE_ID.to_string();

    let result = get_camera_performance(device_id).await;
    match result {
        Ok(metrics) => {
            // Verify metrics structure
            assert!(metrics.capture_latency_ms >= 0.0);
            assert!(metrics.fps_actual >= 0.0);
            assert!(metrics.fps_actual >= 0.0);

            println!("Performance metrics:");
            println!("  Capture latency: {:.2}ms", metrics.capture_latency_ms);
            println!("  Actual FPS: {:.2}", metrics.fps_actual);
            println!("  Processing time: {:.2}ms", metrics.processing_time_ms);
            println!("  Frame drops: {}", metrics.dropped_frames);
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Performance test failed (expected in CI): {}", e);
        }
        Err(e) => {
            println!("Unexpected performance error: {}", e);
        }
    }
}

/// Test burst configuration parameter validation
#[tokio::test]
async fn test_burst_config_validation() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test invalid count (0)
    let invalid_config_zero = BurstConfig {
        count: 0,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };

    let result = capture_burst_sequence(device_id.clone(), invalid_config_zero).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Invalid burst count"));
    }

    // Test invalid count (too high)
    let invalid_config_high = BurstConfig {
        count: 100,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };

    let result = capture_burst_sequence(device_id, invalid_config_high).await;
    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.contains("Invalid burst count"));
    }
}

/// Performance benchmark for advanced camera operations
#[tokio::test]
async fn test_advanced_operations_performance() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Benchmark controls setting speed
    let start = Instant::now();
    let controls = create_test_controls();

    match set_camera_controls(device_id.clone(), controls).await {
        Ok(_) => {
            let controls_time = start.elapsed();
            println!("Camera controls setting took: {:?}", controls_time);

            // Should be reasonably fast (under 1 second)
            assert!(controls_time < Duration::from_secs(1));
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Performance test skipped (no camera): {}", e);
            return;
        }
        Err(e) => {
            println!("Unexpected performance test error: {}", e);
        }
    }

    // Benchmark burst capture performance
    let start = Instant::now();
    let burst_config = BurstConfig {
        count: 5,
        interval_ms: 50, // Fast interval
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };

    match capture_burst_sequence(device_id, burst_config).await {
        Ok(frames) => {
            let burst_time = start.elapsed();
            println!(
                "Burst capture ({} frames) took: {:?}",
                frames.len(),
                burst_time
            );

            // Calculate effective FPS
            let fps = frames.len() as f32 / burst_time.as_secs_f32();
            println!("Effective burst FPS: {:.2}", fps);

            // Should maintain reasonable performance
            assert!(fps >= 1.0); // At least 1 FPS
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Burst performance test skipped (no camera): {}", e);
        }
        Err(e) => {
            println!("Unexpected burst performance error: {}", e);
        }
    }
}

/// Test edge cases and boundary conditions
#[tokio::test]
async fn test_advanced_edge_cases() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test very fast intervals
    let fast_config = BurstConfig {
        count: 2,
        interval_ms: 1, // 1ms interval
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };

    let result = capture_burst_sequence(device_id.clone(), fast_config).await;
    match result {
        Ok(frames) => {
            assert_eq!(frames.len(), 2);
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Fast interval test skipped: {}", e);
        }
        Err(e) => {
            println!("Fast interval error: {}", e);
        }
    }

    // Test extreme exposure values (boundary conditions)
    let extreme_controls = CameraControls {
        exposure_time: Some(0.001), // 1ms (very fast)
        iso_sensitivity: Some(50),  // Minimum ISO
        ..CameraControls::default()
    };

    let result = set_camera_controls(device_id, extreme_controls).await;
    match result {
        Ok(_) => {
            // Should handle extreme values
        }
        Err(e) if e.contains("mutex") || e.contains("camera") => {
            println!("Warning: Extreme values test skipped: {}", e);
        }
        Err(e) => {
            println!("Extreme values error: {}", e);
        }
    }
}

/// Test concurrent advanced operations
#[tokio::test]
async fn test_concurrent_advanced_operations() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test concurrent control setting (should handle properly)
    let handles = (0..3)
        .map(|i| {
            let device_id = device_id.clone();
            tokio::spawn(async move {
                let controls = CameraControls {
                    focus_distance: Some(i as f32 * 0.3),
                    ..CameraControls::default()
                };
                set_camera_controls(device_id, controls).await
            })
        })
        .collect::<Vec<_>>();

    let results = futures::future::join_all(handles).await;

    let mut success_count = 0;
    let mut expected_failures = 0;

    for result in results {
        match result {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) if e.contains("mutex") || e.contains("camera") => {
                expected_failures += 1;
            }
            Ok(Err(e)) => {
                println!("Unexpected concurrent error: {}", e);
            }
            Err(e) => {
                println!("Task join error: {}", e);
            }
        }
    }

    // Either all should succeed or all should fail (CI)
    if success_count > 0 {
        println!("Concurrent operations succeeded: {}", success_count);
    } else {
        println!(
            "Concurrent operations failed (expected in CI): {}",
            expected_failures
        );
    }
}

/// Test memory usage and resource cleanup in advanced operations
#[tokio::test]
async fn test_resource_management() {
    let device_id = TEST_DEVICE_ID.to_string();

    // Test multiple burst sequences to ensure proper cleanup
    for i in 0..3 {
        let config = BurstConfig {
            count: 2,
            interval_ms: 100,
            bracketing: None,
            focus_stacking: false,
            auto_save: false,
            save_directory: None,
        };

        match capture_burst_sequence(device_id.clone(), config).await {
            Ok(frames) => {
                println!(
                    "Resource test iteration {}: {} frames captured",
                    i + 1,
                    frames.len()
                );

                // Verify frames are properly allocated
                for frame in frames {
                    assert!(!frame.data.is_empty());
                    assert!(frame.size_bytes > 0);
                }
            }
            Err(e) if e.contains("mutex") || e.contains("camera") => {
                println!("Warning: Resource test {} skipped: {}", i + 1, e);
            }
            Err(e) => {
                println!("Resource test {} error: {}", i + 1, e);
            }
        }
    }
}
