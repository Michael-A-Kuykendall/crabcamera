//! Camera Warmup Analysis
//!
//! This example analyzes the OBSBOT camera's warm-up behavior to determine
//! the minimum time needed before we get valid frames.
//!
//! Run with: cargo run --example camera_warmup_analysis --release

use std::thread;
use std::time::{Duration, Instant};

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::{query, Camera};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════════╗");
    println!("║       Camera Warmup Analysis                                     ║");
    println!("║       Testing OBSBOT initialization behavior                     ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    // Step 1: Query cameras
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Step 1: Camera Discovery");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let cameras = query(ApiBackend::MediaFoundation)?;

    if cameras.is_empty() {
        println!("  ❌ No cameras found! Is the OBSBOT connected?");
        return Ok(());
    }

    for (i, cam) in cameras.iter().enumerate() {
        println!("  Camera {}: {}", i, cam.human_name());
        println!("    Index: {:?}", cam.index());
        println!("    Description: {}", cam.description());
    }

    // Step 2: Open camera and measure first frame time
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Step 2: Camera Open + Stream Start");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let open_start = Instant::now();

    // Use highest resolution to match what the camera actually provides
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

    let mut camera = Camera::new(CameraIndex::Index(0), requested)?;
    let open_time = open_start.elapsed();
    println!("  Camera::new() took: {:?}", open_time);

    let fmt = camera.camera_format();
    println!(
        "  Format: {}x{} @ {}fps",
        fmt.resolution().width_x,
        fmt.resolution().height_y,
        fmt.frame_rate()
    );

    let stream_start = Instant::now();
    camera.open_stream()?;
    let stream_time = stream_start.elapsed();
    println!("  open_stream() took: {:?}", stream_time);

    // Step 3: Measure time to first frame
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Step 3: First Frame Timing");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let first_frame_start = Instant::now();
    let first_frame = camera.poll_frame()?;
    let first_frame_time = first_frame_start.elapsed();

    println!("  First frame() call took: {:?}", first_frame_time);
    println!("  Frame size: {} bytes", first_frame.buffer_bytes().len());

    // Check if first frame looks valid (non-zero, proper JPEG header if MJPEG)
    let bytes = first_frame.buffer_bytes();
    let is_jpeg = bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8;
    let non_zero_count = bytes.iter().filter(|&&b| b != 0).count();

    println!("  Is JPEG: {}", is_jpeg);
    println!(
        "  Non-zero bytes: {} ({:.1}%)",
        non_zero_count,
        (non_zero_count as f64 / bytes.len() as f64) * 100.0
    );

    // Step 4: Measure frame-by-frame warmup
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Step 4: Frame-by-Frame Analysis (30 frames)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("  Frame | Time (ms) | Size (bytes) | Valid | Notes");
    println!("  ──────┼───────────┼──────────────┼───────┼────────────────────");

    let mut stable_start: Option<usize> = None;
    let mut prev_size = 0usize;
    let stream_opened = Instant::now();

    for i in 0..30 {
        let frame_start = Instant::now();
        match camera.poll_frame() {
            Ok(frame) => {
                let elapsed = frame_start.elapsed();
                let bytes = frame.buffer_bytes();
                let size = bytes.len();
                let is_valid = bytes.len() > 1000 && (bytes[0] == 0xFF && bytes[1] == 0xD8);

                // Detect stability (similar size frames)
                let size_stable = if prev_size > 0 {
                    let diff = (size as i64 - prev_size as i64).abs();
                    diff < 50000 // Within 50KB
                } else {
                    false
                };

                let notes = if size < 1000 {
                    "TOO SMALL - invalid"
                } else if !is_valid {
                    "Not JPEG - may be blank"
                } else if !size_stable && prev_size > 0 {
                    "Size varying - stabilizing"
                } else if stable_start.is_none() && size_stable {
                    stable_start = Some(i);
                    "←← STABLE START"
                } else {
                    ""
                };

                println!(
                    "  {:5} | {:9.1} | {:12} | {:5} | {}",
                    i + 1,
                    elapsed.as_secs_f64() * 1000.0,
                    size,
                    if is_valid { "✓" } else { "✗" },
                    notes
                );

                prev_size = size;
            }
            Err(e) => {
                println!("  {:5} | ERROR: {}", i + 1, e);
            }
        }

        // Small delay between frames to let camera process
        thread::sleep(Duration::from_millis(33)); // ~30fps
    }

    let total_warmup_time = stream_opened.elapsed();

    // Step 5: Summary
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("  Step 5: Summary & Recommendations");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!(
        "  Total time from stream open to frame 30: {:?}",
        total_warmup_time
    );
    println!(
        "  Camera::new() + open_stream(): {:?}",
        open_time + stream_time
    );

    if let Some(stable_frame) = stable_start {
        let stable_time = Duration::from_millis((stable_frame as u64 + 1) * 33);
        println!(
            "  Frames until stable: {} (approx {:?})",
            stable_frame + 1,
            stable_time
        );
        println!("\n  ✅ RECOMMENDATION:");
        println!(
            "     Warmup: {} frames with 33ms delay = {:?}",
            stable_frame + 5, // Add buffer
            Duration::from_millis((stable_frame as u64 + 5) * 33)
        );
    } else {
        println!("  ⚠️  Could not detect stable point");
        println!("\n  RECOMMENDATION:");
        println!("     Use at least 15 frames warmup (~500ms)");
    }

    // Cleanup
    println!("\n  Stopping camera...");
    camera.stop_stream()?;
    println!("  Done!");

    Ok(())
}
