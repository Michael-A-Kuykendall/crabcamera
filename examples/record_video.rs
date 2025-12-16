//! Real camera recording test
//! 
//! This example records 5 seconds of video from your default camera
//! to test the CrabCamera + Muxide integration.
//!
//! Usage: cargo run --example record_video --features recording

use std::path::PathBuf;
use std::time::{Duration, Instant};
use crabcamera::platform::CameraSystem;
use crabcamera::types::{CameraFormat, CameraFrame, CameraInitParams};
use crabcamera::PlatformCamera;

/// Simple nearest-neighbor downscaling for RGB frames
fn downscale_frame(frame: &CameraFrame, target_width: u32, target_height: u32) -> CameraFrame {
    let src_w = frame.width as usize;
    let src_h = frame.height as usize;
    let dst_w = target_width as usize;
    let dst_h = target_height as usize;
    
    let mut data = Vec::with_capacity(dst_w * dst_h * 3);
    
    for dy in 0..dst_h {
        let sy = (dy * src_h) / dst_h;
        for dx in 0..dst_w {
            let sx = (dx * src_w) / dst_w;
            let src_idx = (sy * src_w + sx) * 3;
            data.push(frame.data[src_idx]);
            data.push(frame.data[src_idx + 1]);
            data.push(frame.data[src_idx + 2]);
        }
    }
    
    CameraFrame::new(data, target_width, target_height, frame.device_id.clone())
        .with_format(frame.format.clone())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();

    println!("🦀 CrabCamera Video Recording Test");
    println!("===================================");

    // List available cameras
    let cameras = CameraSystem::list_cameras()?;
    println!("\n📷 Available cameras:");
    for (i, cam) in cameras.iter().enumerate() {
        println!("  [{}] {} ({})", i, cam.name, cam.id);
    }

    if cameras.is_empty() {
        println!("❌ No cameras found!");
        return Ok(());
    }

    // Use first camera
    let camera_info = &cameras[0];
    println!("\n🎬 Using camera: {}", camera_info.name);

    // Get formats from camera info
    let formats = &camera_info.supports_formats;
    println!("📐 Supported formats: {} total", formats.len());
    
    // Find a good format - prefer 720p or 1080p
    let format = formats.iter()
        .find(|f| f.width == 1280 && f.height == 720)
        .or_else(|| formats.iter().find(|f| f.width == 1920 && f.height == 1080))
        .or_else(|| formats.iter().find(|f| f.width >= 640 && f.height >= 480))
        .cloned()
        .unwrap_or_else(|| CameraFormat::new(640, 480, 30.0));
    
    println!("🎯 Selected format: {}x{} @ {}fps", format.width, format.height, format.fps);

    // Configure recording
    let output_path = PathBuf::from("test_recording.mp4");

    println!("\n⏺️  Starting recording to: {}", output_path.display());
    println!("   Duration: 5 seconds");
    println!("   Press Ctrl+C to stop early\n");

    // Initialize camera using PlatformCamera
    let init_params = CameraInitParams::new(camera_info.id.clone())
        .with_format(format.clone());
    let mut cam = PlatformCamera::new(init_params)?;
    cam.start_stream()?;

    // Capture one test frame to check actual dimensions
    println!("   Checking actual camera output...");
    let test_frame = cam.capture_frame()?;
    let actual_width = test_frame.width;
    let actual_height = test_frame.height;
    
    if actual_width != format.width || actual_height != format.height {
        println!("   ⚠️  Camera outputs {}x{} (requested {}x{})", 
            actual_width, actual_height, format.width, format.height);
    }
    
    // For 4K cameras, downscale to 720p to improve encoding speed
    // openh264 is software-only and 4K encoding is very slow
    let (record_width, record_height) = if actual_width > 1920 {
        println!("   📐 Downscaling to 720p for faster encoding");
        (1280u32, 720u32)
    } else {
        (actual_width, actual_height)
    };
    
    // Use actual frame dimensions for recording config
    let config = crabcamera::recording::RecordingConfig::new(
        record_width,
        record_height,
        format.fps as f64,
    ).with_title("CrabCamera Test Recording");

    println!("   🎬 Recording at {}x{}p @ {}fps", record_width, record_height, format.fps);

    // Create recorder
    let mut recorder = crabcamera::recording::Recorder::new(&output_path, config)?;

    // For the test frame, we need to downscale if needed
    let test_frame_to_record = if record_width != actual_width {
        downscale_frame(&test_frame, record_width, record_height)
    } else {
        test_frame.clone()
    };

    // Write the test frame we already captured
    recorder.write_rgb_frame(&test_frame_to_record.data, record_width, record_height)?;
    let mut frame_count = 1u64;

    // Record for 5 seconds
    let target_duration = Duration::from_secs(5);
    let start = Instant::now();
    let target_frame_duration = Duration::from_secs_f64(1.0 / format.fps as f64);
    let mut last_print = Instant::now();
    let needs_downscale = record_width != actual_width;

    while start.elapsed() < target_duration {
        let frame_start = Instant::now();

        // Capture frame
        match cam.capture_frame() {
            Ok(frame) => {
                // Downscale if needed
                let frame_to_record = if needs_downscale {
                    downscale_frame(&frame, record_width, record_height)
                } else {
                    frame
                };

                // Write to recorder
                if let Err(e) = recorder.write_rgb_frame(&frame_to_record.data, record_width, record_height) {
                    log::error!("Failed to write frame: {}", e);
                    continue;
                }
                frame_count += 1;
            }
            Err(e) => {
                log::warn!("Frame capture failed: {}", e);
            }
        }

        // Print progress every second
        if last_print.elapsed() >= Duration::from_secs(1) {
            let elapsed = start.elapsed().as_secs_f64();
            let actual_fps = frame_count as f64 / elapsed;
            print!("\r   📊 {:.1}s - {} frames ({:.1} fps)   ", elapsed, frame_count, actual_fps);
            use std::io::Write;
            std::io::stdout().flush()?;
            last_print = Instant::now();
        }

        // Frame rate limiting
        let frame_time = frame_start.elapsed();
        if frame_time < target_frame_duration {
            std::thread::sleep(target_frame_duration - frame_time);
        }
    }

    println!("\n\n⏹️  Stopping recording...");

    // Stop camera
    cam.stop_stream()?;

    // Finalize recording
    let stats = recorder.finish()?;

    println!("\n✅ Recording complete!");
    println!("   📊 Statistics:");
    println!("      - Video frames: {}", stats.video_frames);
    println!("      - Duration: {:.2}s", stats.duration_secs);
    println!("      - File size: {:.2} MB", stats.bytes_written as f64 / 1_048_576.0);
    println!("      - Average FPS: {:.1}", stats.actual_fps);
    println!("      - Dropped frames: {}", stats.dropped_frames);
    println!("   📁 Output: {}", stats.output_path);

    // Verify file is playable
    let metadata = std::fs::metadata(&output_path)?;
    println!("\n🎬 File created: {} bytes", metadata.len());
    println!("   Try playing it with: vlc {}", output_path.display());

    Ok(())
}
