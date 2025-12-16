//! Save test outputs for physical verification
//! 
//! Run with: cargo run --example save_test_output --features recording --release
//!
//! Creates:
//!   - test_outputs/capture_raw.jpg      (single frame from camera)
//!   - test_outputs/capture_crabcamera.jpg (frame via CrabCamera)
//!   - test_outputs/recording_3sec.mp4   (3 second video recording)

use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};
use std::thread;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn")
    ).init();

    let output_dir = Path::new("test_outputs");
    fs::create_dir_all(output_dir)?;

    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       CrabCamera Output Verification                             ║");
    println!("║       Saving files to test_outputs/                              ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // ═══════════════════════════════════════════════════════════════════════════
    // 1. RAW NOKHWA CAPTURE (MJPEG)
    // ═══════════════════════════════════════════════════════════════════════════
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  1. Raw Nokhwa MJPEG Capture");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    use nokhwa::Camera;
    use nokhwa::pixel_format::RgbFormat;
    use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};

    // NOTE: Don't use query() - it fails when camera is asleep.
    // Camera::new() directly wakes the camera via MediaFoundation.
    print!("  Opening camera (this wakes it up)... ");
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(CameraIndex::Index(0), requested)?;
    let fmt = camera.camera_format();
    println!("{}x{} @ {}fps", fmt.resolution().width_x, fmt.resolution().height_y, fmt.frame_rate());

    camera.open_stream()?;
    
    // Warmup
    print!("  Warming up (5 frames)... ");
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(100));
        let _ = camera.frame();
    }
    println!("done");

    // Capture raw MJPEG frame
    print!("  Capturing MJPEG frame... ");
    let frame = camera.frame()?;
    let raw_bytes = frame.buffer_bytes();
    
    // Check if it's JPEG
    let is_jpeg = raw_bytes.len() >= 3 && raw_bytes[0] == 0xFF && raw_bytes[1] == 0xD8;
    
    let raw_path = output_dir.join("capture_raw.jpg");
    let raw_len = raw_bytes.len();
    if is_jpeg {
        fs::write(&raw_path, &raw_bytes)?;
        println!("✅ Saved {} bytes to {}", raw_len, raw_path.display());
    } else {
        // Convert to JPEG using image crate
        let res = frame.resolution();
        let rgb = frame.decode_image::<RgbFormat>()?;
        let img = image::RgbImage::from_raw(res.width_x, res.height_y, rgb.to_vec())
            .ok_or("Failed to create image")?;
        img.save(&raw_path)?;
        println!("✅ Converted and saved to {}", raw_path.display());
    }

    camera.stop_stream()?;
    drop(camera);
    thread::sleep(Duration::from_millis(500));

    // ═══════════════════════════════════════════════════════════════════════════
    // 2. CRABCAMERA RGB CAPTURE
    // ═══════════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  2. CrabCamera RGB Capture");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    use crabcamera::platform::CameraSystem;
    use crabcamera::types::{CameraFormat, CameraInitParams};
    use crabcamera::PlatformCamera;

    let crab_cameras = CameraSystem::list_cameras()?;
    let camera_id = crab_cameras[0].id.clone();

    print!("  Creating PlatformCamera... ");
    let format = CameraFormat::new(1920, 1080, 30.0);
    let params = CameraInitParams::new(camera_id.clone()).with_format(format);
    let mut platform_cam = PlatformCamera::new(params)?;
    println!("done");

    platform_cam.start_stream()?;

    // Warmup
    for _ in 0..3 {
        thread::sleep(Duration::from_millis(100));
        let _ = platform_cam.capture_frame();
    }

    print!("  Capturing RGB frame... ");
    let frame = platform_cam.capture_frame()?;
    
    // Save as JPEG
    let crab_path = output_dir.join("capture_crabcamera.jpg");
    let img = image::RgbImage::from_raw(frame.width, frame.height, frame.data.clone())
        .ok_or("Failed to create image from CrabCamera frame")?;
    img.save(&crab_path)?;
    println!("✅ Saved {}x{} ({} bytes RGB) to {}", 
        frame.width, frame.height, frame.data.len(), crab_path.display());

    platform_cam.stop_stream()?;
    drop(platform_cam);
    thread::sleep(Duration::from_millis(500));

    // ═══════════════════════════════════════════════════════════════════════════
    // 3. VIDEO RECORDING (3 seconds)
    // ═══════════════════════════════════════════════════════════════════════════
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  3. Video Recording (3 seconds)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    use crabcamera::recording::{Recorder, RecordingConfig};

    // Re-init camera
    let params = CameraInitParams::new(camera_id).with_format(CameraFormat::new(1920, 1080, 30.0));
    let mut cam = PlatformCamera::new(params)?;
    cam.start_stream()?;

    // Warmup
    for _ in 0..5 {
        thread::sleep(Duration::from_millis(50));
        let _ = cam.capture_frame();
    }

    // Get actual frame dimensions
    let test_frame = cam.capture_frame()?;
    let (cam_w, cam_h) = (test_frame.width, test_frame.height);
    println!("  Camera resolution: {}x{}", cam_w, cam_h);

    // Use 720p for encoding (faster)
    let (rec_w, rec_h) = if cam_w > 1920 { (1280u32, 720u32) } else { (cam_w, cam_h) };
    println!("  Recording resolution: {}x{}", rec_w, rec_h);

    let video_path = output_dir.join("recording_3sec.mp4");
    let config = RecordingConfig::new(rec_w, rec_h, 30.0)
        .with_title("CrabCamera Test Recording");

    let mut recorder = Recorder::new(&video_path, config)?;

    print!("  Recording");
    let start = Instant::now();
    let target_duration = Duration::from_secs(3);
    let frame_interval = Duration::from_millis(33); // ~30fps
    let mut frame_count = 0u32;

    while start.elapsed() < target_duration {
        let frame_start = Instant::now();
        
        if let Ok(frame) = cam.capture_frame() {
            // Downscale if needed
            let data = if cam_w != rec_w {
                downscale_rgb(&frame.data, cam_w as usize, cam_h as usize, rec_w as usize, rec_h as usize)
            } else {
                frame.data.clone()
            };
            
            if recorder.write_rgb_frame(&data, rec_w, rec_h).is_ok() {
                frame_count += 1;
                if frame_count % 10 == 0 {
                    print!(".");
                }
            }
        }
        
        // Frame rate limiting
        let elapsed = frame_start.elapsed();
        if elapsed < frame_interval {
            thread::sleep(frame_interval - elapsed);
        }
    }
    println!(" done!");

    cam.stop_stream()?;

    let stats = recorder.finish()?;
    
    println!("\n  ✅ Recording complete!");
    println!("     File: {}", video_path.display());
    println!("     Frames: {}", stats.video_frames);
    println!("     Duration: {:.2}s", stats.duration_secs);
    println!("     Size: {} KB", stats.bytes_written / 1024);
    println!("     Bitrate: {:.1} kbps", (stats.bytes_written as f64 * 8.0) / stats.duration_secs / 1000.0);

    // ═══════════════════════════════════════════════════════════════════════════
    // SUMMARY
    // ═══════════════════════════════════════════════════════════════════════════
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║                     OUTPUT FILES CREATED                         ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let files = [
        ("capture_raw.jpg", "Raw MJPEG from nokhwa"),
        ("capture_crabcamera.jpg", "RGB frame via CrabCamera"),
        ("recording_3sec.mp4", "3-second H.264 video"),
    ];

    for (name, desc) in &files {
        let path = output_dir.join(name);
        if path.exists() {
            let meta = fs::metadata(&path)?;
            println!("  ✅ {} ({} bytes)", name, meta.len());
            println!("     └─ {}", desc);
        } else {
            println!("  ❌ {} - not created", name);
        }
    }

    println!("\n  To view:");
    println!("    explorer test_outputs");
    println!("    # or");
    println!("    start test_outputs\\recording_3sec.mp4");

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
