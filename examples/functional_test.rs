//! CrabCamera Functional Test Suite
//! 
//! This test properly warms up the camera before testing all functionality.
//! Run with: cargo run --example functional_test --features recording --release
//!
//! The OB Spot and similar cameras need:
//! 1. Device enumeration (may wake from sleep)
//! 2. Camera initialization with format
//! 3. Stream start (warmup period)
//! 4. Test frame capture before real operations

use std::time::{Duration, Instant};
use std::thread;
use nokhwa::{Camera, query};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       CrabCamera Functional Test Suite                           ║");
    println!("║       Testing with OB Spot / USB Cameras                         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let mut results = TestResults::new();

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 1: RAW NOKHWA - Direct camera access (bypasses crabcamera)
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  PHASE 1: Raw Nokhwa Camera Access (Direct Hardware Test)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Step 1.1: Query cameras with MediaFoundation
    print!("  [1.1] Query cameras (MediaFoundation)... ");
    let cameras = match query(ApiBackend::MediaFoundation) {
        Ok(cams) => {
            println!("✅ Found {} camera(s)", cams.len());
            for (i, cam) in cams.iter().enumerate() {
                println!("        [{i}] {}", cam.human_name());
            }
            results.pass("nokhwa_query_mf");
            cams
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("nokhwa_query_mf", &e.to_string());
            
            // Try Auto backend as fallback
            print!("  [1.1b] Query cameras (Auto fallback)... ");
            match query(ApiBackend::Auto) {
                Ok(cams) => {
                    println!("✅ Found {} camera(s)", cams.len());
                    results.pass("nokhwa_query_auto");
                    cams
                }
                Err(e2) => {
                    println!("❌ {e2}");
                    results.fail("nokhwa_query_auto", &e2.to_string());
                    println!("\n  ⚠️  No cameras detected. Is the OB Spot connected and awake?");
                    println!("      Try unplugging and replugging the camera.\n");
                    results.print_summary();
                    return Ok(());
                }
            }
        }
    };

    if cameras.is_empty() {
        println!("\n  ⚠️  Camera list is empty. Hardware may be sleeping.");
        results.print_summary();
        return Ok(());
    }

    // Step 1.2: Create camera instance
    print!("\n  [1.2] Create camera instance... ");
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = match Camera::new(CameraIndex::Index(0), requested) {
        Ok(cam) => {
            let fmt = cam.camera_format();
            println!("✅ {}x{} @ {}fps", fmt.resolution().width_x, fmt.resolution().height_y, fmt.frame_rate());
            results.pass("nokhwa_create_camera");
            cam
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("nokhwa_create_camera", &e.to_string());
            println!("\n  ⚠️  Camera creation failed. The device may need to be reset.");
            results.print_summary();
            return Ok(());
        }
    };

    // Step 1.3: Open stream (CRITICAL - this wakes up the camera)
    print!("  [1.3] Open camera stream (warmup)... ");
    match camera.open_stream() {
        Ok(_) => {
            println!("✅ Stream opened");
            results.pass("nokhwa_open_stream");
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("nokhwa_open_stream", &e.to_string());
            results.print_summary();
            return Ok(());
        }
    }

    // Step 1.4: Warmup period - capture several frames to stabilize
    print!("  [1.4] Camera warmup (5 frames)... ");
    let warmup_start = Instant::now();
    let mut warmup_success = 0;
    for i in 0..5 {
        thread::sleep(Duration::from_millis(100));
        match camera.frame() {
            Ok(_) => warmup_success += 1,
            Err(e) => {
                if i == 0 {
                    println!("❌ First frame failed: {e}");
                }
            }
        }
    }
    if warmup_success >= 3 {
        println!("✅ {warmup_success}/5 frames in {:?}", warmup_start.elapsed());
        results.pass("nokhwa_warmup");
    } else {
        println!("⚠️  Only {warmup_success}/5 warmup frames succeeded");
        results.fail("nokhwa_warmup", &format!("Only {warmup_success}/5 frames"));
    }

    // Step 1.5: Capture test frame and verify data
    print!("  [1.5] Capture and verify frame data... ");
    match camera.frame() {
        Ok(frame) => {
            let bytes = frame.buffer_bytes();
            let res = frame.resolution();
            let is_jpeg = bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8;
            let data_type = if is_jpeg { "MJPEG" } else { "RAW" };
            println!("✅ {}x{}, {} bytes ({})", res.width_x, res.height_y, bytes.len(), data_type);
            results.pass("nokhwa_capture_frame");
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("nokhwa_capture_frame", &e.to_string());
        }
    }

    // Close stream before CrabCamera tests
    let _ = camera.stop_stream();
    drop(camera);
    thread::sleep(Duration::from_millis(500)); // Let camera reset

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 2: CRABCAMERA PLATFORM LAYER
    // ═══════════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  PHASE 2: CrabCamera Platform Layer");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    use crabcamera::platform::CameraSystem;
    use crabcamera::types::{CameraFormat, CameraInitParams};
    use crabcamera::PlatformCamera;

    // Step 2.1: CameraSystem::list_cameras
    print!("  [2.1] CameraSystem::list_cameras()... ");
    let crab_cameras = match CameraSystem::list_cameras() {
        Ok(cams) => {
            println!("✅ Found {} camera(s)", cams.len());
            for cam in &cams {
                println!("        {} (id: {})", cam.name, cam.id);
            }
            results.pass("crabcamera_list");
            cams
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("crabcamera_list", &e.to_string());
            results.print_summary();
            return Ok(());
        }
    };

    let camera_id = crab_cameras[0].id.clone();
    let camera_name = crab_cameras[0].name.clone();

    // Step 2.2: Create PlatformCamera
    print!("  [2.2] PlatformCamera::new()... ");
    let format = CameraFormat::new(1920, 1080, 30.0);
    let params = CameraInitParams::new(camera_id.clone()).with_format(format.clone());
    let mut platform_cam = match PlatformCamera::new(params) {
        Ok(cam) => {
            println!("✅ Created for '{camera_name}'");
            results.pass("crabcamera_create");
            cam
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("crabcamera_create", &e.to_string());
            results.print_summary();
            return Ok(());
        }
    };

    // Step 2.3: Start stream
    print!("  [2.3] PlatformCamera::start_stream()... ");
    match platform_cam.start_stream() {
        Ok(_) => {
            println!("✅ Stream started");
            results.pass("crabcamera_start_stream");
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("crabcamera_start_stream", &e.to_string());
        }
    }

    // Step 2.4: Warmup
    print!("  [2.4] Platform warmup (3 frames)... ");
    for _ in 0..3 {
        thread::sleep(Duration::from_millis(100));
        let _ = platform_cam.capture_frame();
    }
    println!("✅ Done");
    results.pass("crabcamera_warmup");

    // Step 2.5: Capture and validate frame
    print!("  [2.5] PlatformCamera::capture_frame()... ");
    match platform_cam.capture_frame() {
        Ok(frame) => {
            let valid = frame.width > 0 && frame.height > 0 && !frame.data.is_empty();
            if valid {
                println!("✅ {}x{}, {} bytes", frame.width, frame.height, frame.data.len());
                results.pass("crabcamera_capture");
            } else {
                println!("❌ Invalid frame data");
                results.fail("crabcamera_capture", "Invalid frame dimensions or empty data");
            }
        }
        Err(e) => {
            println!("❌ {e}");
            results.fail("crabcamera_capture", &e.to_string());
        }
    }

    // Cleanup
    let _ = platform_cam.stop_stream();
    drop(platform_cam);
    thread::sleep(Duration::from_millis(500));

    // ═══════════════════════════════════════════════════════════════════════════
    // PHASE 3: RECORDING MODULE (if feature enabled)
    // ═══════════════════════════════════════════════════════════════════════════
    #[cfg(feature = "recording")]
    {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("  PHASE 3: Recording Module (openh264 + muxide)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        use crabcamera::recording::{Recorder, RecordingConfig, H264Encoder};

        // Step 3.1: Create H264Encoder
        print!("  [3.1] H264Encoder::new(640x480)... ");
        match H264Encoder::new(640, 480, 30.0, 2_000_000) {
            Ok(_encoder) => {
                println!("✅ Encoder created");
                results.pass("encoder_create");
            }
            Err(e) => {
                println!("❌ {e}");
                results.fail("encoder_create", &e.to_string());
            }
        }

        // Step 3.2: Encode test frame
        print!("  [3.2] Encode synthetic frame... ");
        match H264Encoder::new(320, 240, 30.0, 1_000_000) {
            Ok(mut encoder) => {
                let test_rgb = vec![128u8; 320 * 240 * 3]; // Gray frame
                match encoder.encode_rgb(&test_rgb) {
                    Ok(encoded) => {
                        let is_annexb = encoded.data.starts_with(&[0, 0, 0, 1]) || 
                                        encoded.data.starts_with(&[0, 0, 1]);
                        if is_annexb && !encoded.data.is_empty() {
                            println!("✅ {} bytes, keyframe={}", encoded.data.len(), encoded.is_keyframe);
                            results.pass("encoder_encode");
                        } else {
                            println!("❌ Invalid Annex B output");
                            results.fail("encoder_encode", "Not valid Annex B");
                        }
                    }
                    Err(e) => {
                        println!("❌ {e}");
                        results.fail("encoder_encode", &e.to_string());
                    }
                }
            }
            Err(e) => {
                println!("❌ Encoder creation failed: {e}");
                results.fail("encoder_encode", &e.to_string());
            }
        }

        // Step 3.3: Create Recorder and write frames
        print!("  [3.3] Recorder: Create + write 10 frames... ");
        let output_path = std::path::PathBuf::from("functional_test_output.mp4");
        let config = RecordingConfig::new(320, 240, 15.0)
            .with_title("Functional Test");
        
        match Recorder::new(&output_path, config) {
            Ok(mut recorder) => {
                let mut success = true;
                for i in 0..10 {
                    let gray = ((i * 20) % 256) as u8;
                    let rgb = vec![gray; 320 * 240 * 3];
                    if let Err(e) = recorder.write_rgb_frame(&rgb, 320, 240) {
                        println!("❌ Frame {i} failed: {e}");
                        success = false;
                        break;
                    }
                }
                if success {
                    match recorder.finish() {
                        Ok(stats) => {
                            println!("✅ {} frames, {} bytes", stats.video_frames, stats.bytes_written);
                            results.pass("recorder_write");
                            // Cleanup
                            let _ = std::fs::remove_file(&output_path);
                        }
                        Err(e) => {
                            println!("❌ Finish failed: {e}");
                            results.fail("recorder_write", &e.to_string());
                        }
                    }
                } else {
                    results.fail("recorder_write", "Frame write failed");
                }
            }
            Err(e) => {
                println!("❌ {e}");
                results.fail("recorder_write", &e.to_string());
            }
        }

        // Step 3.4: Record from real camera
        print!("  [3.4] Record 2 seconds from camera... ");
        
        // Re-init camera
        let params = CameraInitParams::new(camera_id.clone()).with_format(CameraFormat::new(1920, 1080, 30.0));
        match PlatformCamera::new(params) {
            Ok(mut cam) => {
                if cam.start_stream().is_ok() {
                    // Warmup
                    for _ in 0..5 {
                        thread::sleep(Duration::from_millis(50));
                        let _ = cam.capture_frame();
                    }

                    // Get actual dimensions
                    if let Ok(test_frame) = cam.capture_frame() {
                        let (w, h) = (test_frame.width, test_frame.height);
                        
                        // Use 720p if camera is 4K (faster encoding)
                        let (rec_w, rec_h) = if w > 1920 { (1280, 720) } else { (w, h) };
                        
                        let output = std::path::PathBuf::from("functional_test_camera.mp4");
                        let config = RecordingConfig::new(rec_w, rec_h, 30.0)
                            .with_title("Camera Recording Test");
                        
                        match Recorder::new(&output, config) {
                            Ok(mut recorder) => {
                                let start = Instant::now();
                                let mut frame_count = 0u32;
                                
                                while start.elapsed() < Duration::from_secs(2) {
                                    if let Ok(frame) = cam.capture_frame() {
                                        // Downscale if needed
                                        let data = if w != rec_w {
                                            downscale_rgb(&frame.data, w as usize, h as usize, rec_w as usize, rec_h as usize)
                                        } else {
                                            frame.data.clone()
                                        };
                                        
                                        if recorder.write_rgb_frame(&data, rec_w, rec_h).is_ok() {
                                            frame_count += 1;
                                        }
                                    }
                                    thread::sleep(Duration::from_millis(33)); // ~30fps
                                }
                                
                                let _ = cam.stop_stream();
                                let _ = frame_count; // Silence unused warning
                                
                                match recorder.finish() {
                                    Ok(stats) => {
                                        println!("✅ {} frames, {:.2}s, {} KB", 
                                            stats.video_frames, 
                                            stats.duration_secs,
                                            stats.bytes_written / 1024);
                                        results.pass("recorder_camera");
                                        let _ = std::fs::remove_file(&output);
                                    }
                                    Err(e) => {
                                        println!("❌ {e}");
                                        results.fail("recorder_camera", &e.to_string());
                                    }
                                }
                            }
                            Err(e) => {
                                println!("❌ {e}");
                                results.fail("recorder_camera", &e.to_string());
                            }
                        }
                    } else {
                        println!("❌ Could not get test frame");
                        results.fail("recorder_camera", "No test frame");
                    }
                } else {
                    println!("❌ Stream start failed");
                    results.fail("recorder_camera", "Stream start failed");
                }
            }
            Err(e) => {
                println!("❌ {e}");
                results.fail("recorder_camera", &e.to_string());
            }
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════════════
    println!();
    results.print_summary();

    Ok(())
}

/// Simple nearest-neighbor downscale
fn downscale_rgb(src: &[u8], src_w: usize, src_h: usize, dst_w: usize, dst_h: usize) -> Vec<u8> {
    let mut dst = Vec::with_capacity(dst_w * dst_h * 3);
    for dy in 0..dst_h {
        let sy = (dy * src_h) / dst_h;
        for dx in 0..dst_w {
            let sx = (dx * src_w) / dst_w;
            let i = (sy * src_w + sx) * 3;
            dst.push(src[i]);
            dst.push(src[i + 1]);
            dst.push(src[i + 2]);
        }
    }
    dst
}

struct TestResults {
    passed: Vec<String>,
    failed: Vec<(String, String)>,
}

impl TestResults {
    fn new() -> Self {
        Self { passed: Vec::new(), failed: Vec::new() }
    }
    
    fn pass(&mut self, name: &str) {
        self.passed.push(name.to_string());
    }
    
    fn fail(&mut self, name: &str, reason: &str) {
        self.failed.push((name.to_string(), reason.to_string()));
    }
    
    fn print_summary(&self) {
        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                     FUNCTIONAL TEST SUMMARY                      ║");
        println!("╚══════════════════════════════════════════════════════════════════╝\n");
        
        let total = self.passed.len() + self.failed.len();
        let pct = if total > 0 { (self.passed.len() * 100) / total } else { 0 };
        
        println!("  Total: {}  |  ✅ Passed: {}  |  ❌ Failed: {}  |  Rate: {}%\n",
            total, self.passed.len(), self.failed.len(), pct);
        
        if !self.failed.is_empty() {
            println!("  Failed Tests:");
            for (name, reason) in &self.failed {
                println!("    ❌ {name}: {reason}");
            }
            println!();
        }
        
        println!("  Passed Tests:");
        for name in &self.passed {
            println!("    ✅ {name}");
        }
    }
}
