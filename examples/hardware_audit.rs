//! Comprehensive Hardware Test - Tests ALL CrabCamera commands with real hardware
//! 
//! Run with: cargo run --example hardware_audit
//!
//! This tests every command that interacts with camera hardware.
//! Results are printed to console and can be used to verify functionality.

use crabcamera::commands::init::{
    initialize_camera_system, get_available_cameras, get_system_diagnostics,
    get_platform_info, test_camera_system, get_current_platform,
    check_camera_availability, get_camera_formats, get_recommended_format,
    get_optimal_settings,
};
use crabcamera::commands::capture::{
    capture_single_photo, capture_photo_sequence, capture_with_quality_retry,
    start_camera_preview, stop_camera_preview, release_camera,
    save_frame_to_disk, save_frame_compressed, get_capture_stats,
};
use crabcamera::commands::advanced::{
    get_camera_controls, set_camera_controls, test_camera_capabilities,
    capture_burst_sequence, get_camera_performance,
};
use crabcamera::commands::quality::{
    validate_frame_quality, validate_provided_frame,
    get_quality_config, capture_best_quality_frame,
};
use crabcamera::commands::permissions::{
    request_camera_permission, check_camera_permission_status,
};
use crabcamera::types::{CameraControls, BurstConfig};
use std::time::Duration;
use tokio::time::sleep;

#[allow(dead_code)]
struct TestResult {
    name: &'static str,
    passed: bool,
    message: String,
}

impl TestResult {
    fn pass(name: &'static str) -> Self {
        TestResult { name, passed: true, message: "OK".to_string() }
    }
    
    fn fail(name: &'static str, msg: impl Into<String>) -> Self {
        TestResult { name, passed: false, message: msg.into() }
    }
}

#[tokio::main]
async fn main() {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║          CrabCamera Hardware Audit - v{}                  ║", crabcamera::VERSION);
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let mut results: Vec<TestResult> = Vec::new();
    let mut device_id = String::from("0");
    
    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 1: INITIALIZATION & SYSTEM INFO
    // ═══════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 1: Initialization & System Info");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: initialize_camera_system
    print!("  [1.1] initialize_camera_system ... ");
    match initialize_camera_system().await {
        Ok(msg) => {
            println!("✅ {}", msg);
            results.push(TestResult::pass("initialize_camera_system"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("initialize_camera_system", e));
        }
    }

    // Test: get_current_platform
    print!("  [1.2] get_current_platform ... ");
    match get_current_platform().await {
        Ok(platform) => {
            println!("✅ {}", platform);
            results.push(TestResult::pass("get_current_platform"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_current_platform", e));
        }
    }

    // Test: get_platform_info
    print!("  [1.3] get_platform_info ... ");
    match get_platform_info().await {
        Ok(info) => {
            println!("✅ {:?} / {}", info.platform, info.backend);
            results.push(TestResult::pass("get_platform_info"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_platform_info", e));
        }
    }

    // Test: get_system_diagnostics
    print!("  [1.4] get_system_diagnostics ... ");
    match get_system_diagnostics().await {
        Ok(diag) => {
            println!("✅ {} cameras, permission={}", diag.camera_count, diag.permission_status);
            results.push(TestResult::pass("get_system_diagnostics"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_system_diagnostics", e));
        }
    }

    // Test: test_camera_system
    print!("  [1.5] test_camera_system ... ");
    match test_camera_system().await {
        Ok(result) => {
            let successes = result.test_results.iter()
                .filter(|(_, r)| matches!(r, crabcamera::platform::CameraTestResult::Success))
                .count();
            println!("✅ {} cameras, {} passed", result.cameras_found, successes);
            results.push(TestResult::pass("test_camera_system"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("test_camera_system", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 2: CAMERA DISCOVERY
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 2: Camera Discovery");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: get_available_cameras
    print!("  [2.1] get_available_cameras ... ");
    match get_available_cameras().await {
        Ok(cameras) => {
            if cameras.is_empty() {
                println!("⚠️  No cameras found!");
                results.push(TestResult::fail("get_available_cameras", "No cameras"));
            } else {
                println!("✅ Found {} camera(s):", cameras.len());
                for cam in &cameras {
                    println!("       → {} (ID: {})", cam.name, cam.id);
                    device_id = cam.id.clone();
                }
                results.push(TestResult::pass("get_available_cameras"));
            }
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_available_cameras", e));
        }
    }

    // Test: check_camera_availability
    print!("  [2.2] check_camera_availability({}) ... ", device_id);
    match check_camera_availability(device_id.clone()).await {
        Ok(available) => {
            println!("{}", if available { "✅ Available" } else { "❌ Not available" });
            results.push(if available { 
                TestResult::pass("check_camera_availability") 
            } else { 
                TestResult::fail("check_camera_availability", "Not available") 
            });
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("check_camera_availability", e));
        }
    }

    // Test: get_camera_formats
    print!("  [2.3] get_camera_formats({}) ... ", device_id);
    match get_camera_formats(device_id.clone()).await {
        Ok(formats) => {
            println!("✅ {} format(s)", formats.len());
            for fmt in formats.iter().take(3) {
                println!("       → {}x{} @ {}fps", fmt.width, fmt.height, fmt.fps);
            }
            results.push(TestResult::pass("get_camera_formats"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_camera_formats", e));
        }
    }

    // Test: get_recommended_format
    print!("  [2.4] get_recommended_format ... ");
    match get_recommended_format().await {
        Ok(fmt) => {
            println!("✅ {}x{} @ {}fps", fmt.width, fmt.height, fmt.fps);
            results.push(TestResult::pass("get_recommended_format"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_recommended_format", e));
        }
    }

    // Test: get_optimal_settings
    print!("  [2.5] get_optimal_settings ... ");
    match get_optimal_settings().await {
        Ok(settings) => {
            println!("✅ device={}", settings.device_id);
            results.push(TestResult::pass("get_optimal_settings"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_optimal_settings", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 3: PERMISSIONS
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 3: Permissions");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: check_camera_permission_status
    print!("  [3.1] check_camera_permission_status ... ");
    match check_camera_permission_status().await {
        Ok(info) => {
            println!("✅ {}", info.status);
            results.push(TestResult::pass("check_camera_permission_status"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("check_camera_permission_status", e));
        }
    }

    // Test: request_camera_permission
    print!("  [3.2] request_camera_permission ... ");
    match request_camera_permission().await {
        Ok(info) => {
            println!("✅ {}", info.status);
            results.push(TestResult::pass("request_camera_permission"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("request_camera_permission", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 4: CAMERA CONTROLS & CAPABILITIES
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 4: Camera Controls & Capabilities");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: test_camera_capabilities
    print!("  [4.1] test_camera_capabilities({}) ... ", device_id);
    match test_camera_capabilities(device_id.clone()).await {
        Ok(caps) => {
            println!("✅");
            println!("       Auto Focus:     {}", if caps.supports_auto_focus { "✓" } else { "✗" });
            println!("       Manual Focus:   {}", if caps.supports_manual_focus { "✓" } else { "✗" });
            println!("       Auto Exposure:  {}", if caps.supports_auto_exposure { "✓" } else { "✗" });
            println!("       Manual Exposure:{}", if caps.supports_manual_exposure { "✓" } else { "✗" });
            println!("       White Balance:  {}", if caps.supports_white_balance { "✓" } else { "✗" });
            results.push(TestResult::pass("test_camera_capabilities"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("test_camera_capabilities", e));
        }
    }

    // Test: get_camera_controls
    print!("  [4.2] get_camera_controls({}) ... ", device_id);
    match get_camera_controls(device_id.clone()).await {
        Ok(controls) => {
            println!("✅");
            println!("       Auto Focus:    {:?}", controls.auto_focus);
            println!("       Auto Exposure: {:?}", controls.auto_exposure);
            println!("       Brightness:    {:?}", controls.brightness);
            println!("       Contrast:      {:?}", controls.contrast);
            results.push(TestResult::pass("get_camera_controls"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_camera_controls", e));
        }
    }

    // Test: set_camera_controls (toggle auto-exposure)
    print!("  [4.3] set_camera_controls({}) ... ", device_id);
    let test_controls = CameraControls {
        brightness: Some(0.0),
        ..CameraControls::default()
    };
    match set_camera_controls(device_id.clone(), test_controls).await {
        Ok(msg) => {
            println!("✅ {}", msg);
            results.push(TestResult::pass("set_camera_controls"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("set_camera_controls", e));
        }
    }

    // Test: get_camera_performance
    print!("  [4.4] get_camera_performance({}) ... ", device_id);
    match get_camera_performance(device_id.clone()).await {
        Ok(perf) => {
            println!("✅ {:.1}fps, latency={:.1}ms", perf.fps_actual, perf.capture_latency_ms);
            results.push(TestResult::pass("get_camera_performance"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_camera_performance", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 5: STREAM CONTROL
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 5: Stream Control");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: start_camera_preview
    print!("  [5.1] start_camera_preview({}) ... ", device_id);
    match start_camera_preview(device_id.clone(), None).await {
        Ok(msg) => {
            println!("✅ {}", msg);
            results.push(TestResult::pass("start_camera_preview"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("start_camera_preview", e));
        }
    }

    // Wait for camera to warm up
    println!("       ⏳ Waiting 3 seconds for camera warmup...");
    sleep(Duration::from_secs(3)).await;

    // Test: get_capture_stats
    print!("  [5.2] get_capture_stats({}) ... ", device_id);
    match get_capture_stats(device_id.clone()).await {
        Ok(stats) => {
            println!("✅ device={}, active={}", stats.device_id, stats.is_active);
            results.push(TestResult::pass("get_capture_stats"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_capture_stats", e));
        }
    }

    // Test: stop_camera_preview
    print!("  [5.3] stop_camera_preview({}) ... ", device_id);
    match stop_camera_preview(device_id.clone()).await {
        Ok(msg) => {
            println!("✅ {}", msg);
            results.push(TestResult::pass("stop_camera_preview"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("stop_camera_preview", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 6: CAPTURE OPERATIONS
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 6: Capture Operations (CAMERA WILL ACTIVATE)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: capture_single_photo
    print!("  [6.1] capture_single_photo({}) ... ", device_id);
    let captured_frame = match capture_single_photo(Some(device_id.clone()), None).await {
        Ok(frame) => {
            println!("✅ {}x{}, {} bytes", frame.width, frame.height, frame.size_bytes);
            results.push(TestResult::pass("capture_single_photo"));
            Some(frame)
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("capture_single_photo", e));
            None
        }
    };

    // Test: save_frame_to_disk
    if let Some(ref frame) = captured_frame {
        print!("  [6.2] save_frame_to_disk ... ");
        match save_frame_to_disk(frame.clone(), "audit_raw.png".to_string()).await {
            Ok(msg) => {
                println!("✅ {}", msg);
                results.push(TestResult::pass("save_frame_to_disk"));
            }
            Err(e) => {
                println!("❌ {}", e);
                results.push(TestResult::fail("save_frame_to_disk", e));
            }
        }

        // Test: save_frame_compressed
        print!("  [6.3] save_frame_compressed ... ");
        match save_frame_compressed(frame.clone(), "audit_compressed.jpg".to_string(), Some(85)).await {
            Ok(msg) => {
                println!("✅ {}", msg);
                results.push(TestResult::pass("save_frame_compressed"));
            }
            Err(e) => {
                println!("❌ {}", e);
                results.push(TestResult::fail("save_frame_compressed", e));
            }
        }
    }

    // Test: capture_photo_sequence
    print!("  [6.4] capture_photo_sequence (3 photos) ... ");
    match capture_photo_sequence(device_id.clone(), 3, 200, None).await {
        Ok(frames) => {
            println!("✅ Captured {} frames", frames.len());
            results.push(TestResult::pass("capture_photo_sequence"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("capture_photo_sequence", e));
        }
    }

    // Test: capture_with_quality_retry
    print!("  [6.5] capture_with_quality_retry (min 0.5) ... ");
    match capture_with_quality_retry(Some(device_id.clone()), Some(5), Some(0.5), None).await {
        Ok(frame) => {
            println!("✅ {}x{}", frame.width, frame.height);
            results.push(TestResult::pass("capture_with_quality_retry"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("capture_with_quality_retry", e));
        }
    }

    // Test: capture_burst_sequence
    print!("  [6.6] capture_burst_sequence (5 frames) ... ");
    let burst_config = BurstConfig {
        count: 5,
        interval_ms: 100,
        bracketing: None,
        focus_stacking: false,
        auto_save: false,
        save_directory: None,
    };
    match capture_burst_sequence(device_id.clone(), burst_config).await {
        Ok(frames) => {
            println!("✅ Captured {} frames", frames.len());
            results.push(TestResult::pass("capture_burst_sequence"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("capture_burst_sequence", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 7: QUALITY VALIDATION
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 7: Quality Validation");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: get_quality_config
    print!("  [7.1] get_quality_config ... ");
    match get_quality_config().await {
        Ok(config) => {
            println!("✅ blur_threshold={}", config.blur_threshold);
            results.push(TestResult::pass("get_quality_config"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("get_quality_config", e));
        }
    }

    // Test: validate_frame_quality
    print!("  [7.2] validate_frame_quality({}) ... ", device_id);
    match validate_frame_quality(Some(device_id.clone()), None).await {
        Ok(report) => {
            println!("✅ overall={:.2}, blur={:.2}, exposure={:.2}", 
                report.score.overall, report.score.blur, report.score.exposure);
            results.push(TestResult::pass("validate_frame_quality"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("validate_frame_quality", e));
        }
    }

    // Test: validate_provided_frame
    if let Some(ref frame) = captured_frame {
        print!("  [7.3] validate_provided_frame ... ");
        match validate_provided_frame(frame.clone()).await {
            Ok(report) => {
                println!("✅ overall={:.2}", report.score.overall);
                results.push(TestResult::pass("validate_provided_frame"));
            }
            Err(e) => {
                println!("❌ {}", e);
                results.push(TestResult::fail("validate_provided_frame", e));
            }
        }
    }

    // Test: capture_best_quality_frame
    print!("  [7.4] capture_best_quality_frame (5 attempts) ... ");
    match capture_best_quality_frame(Some(device_id.clone()), None, Some(5)).await {
        Ok(result) => {
            println!("✅ {}x{}, score={:.2}", result.frame.width, result.frame.height, result.quality_report.score.overall);
            results.push(TestResult::pass("capture_best_quality_frame"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("capture_best_quality_frame", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SECTION 8: CLEANUP
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  SECTION 8: Cleanup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Test: release_camera
    print!("  [8.1] release_camera({}) ... ", device_id);
    match release_camera(device_id.clone()).await {
        Ok(msg) => {
            println!("✅ {}", msg);
            results.push(TestResult::pass("release_camera"));
        }
        Err(e) => {
            println!("❌ {}", e);
            results.push(TestResult::fail("release_camera", e));
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════════
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                        AUDIT SUMMARY                             ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let total = results.len();

    println!("  Total Tests: {}", total);
    println!("  ✅ Passed:   {}", passed);
    println!("  ❌ Failed:   {}", failed);
    println!("  Success Rate: {:.1}%\n", (passed as f64 / total as f64) * 100.0);

    if failed > 0 {
        println!("  Failed Tests:");
        for result in results.iter().filter(|r| !r.passed) {
            println!("    ❌ {} - {}", result.name, result.message);
        }
    }

    println!("\n  Output files created:");
    println!("    → audit_raw.png");
    println!("    → audit_compressed.jpg");
    
    println!("\n══════════════════════════════════════════════════════════════════════");
}
