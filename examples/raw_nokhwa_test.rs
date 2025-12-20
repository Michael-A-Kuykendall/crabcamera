//! Raw nokhwa test - bypass crabcamera to test camera directly
//!
//! Run with: cargo run --example raw_nokhwa_test

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::{query, Camera};
use std::thread;
use std::time::Duration;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Raw Nokhwa Camera Test");
    println!("═══════════════════════════════════════════════════════════════\n");

    // List cameras
    println!("📋 Listing cameras...");
    match query(ApiBackend::MediaFoundation) {
        Ok(cameras) => {
            for (i, cam) in cameras.iter().enumerate() {
                println!("   [{}] {}", i, cam.human_name());
            }
        }
        Err(e) => {
            println!("   ❌ Failed to query cameras: {}", e);
            return;
        }
    }
    println!();

    // Create camera
    println!("📋 Creating camera...");
    let requested_format =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = match Camera::new(CameraIndex::Index(0), requested_format) {
        Ok(cam) => {
            println!("   ✅ Camera created");
            println!("   Format: {:?}", cam.camera_format());
            cam
        }
        Err(e) => {
            println!("   ❌ Failed to create camera: {}", e);
            return;
        }
    };

    // Open stream
    println!("\n📋 Opening camera stream...");
    match camera.open_stream() {
        Ok(_) => println!("   ✅ Stream opened"),
        Err(e) => {
            println!("   ❌ Failed to open stream: {}", e);
            return;
        }
    }

    println!("   Stream is open: {}", camera.is_stream_open());

    // Warmup - capture frames and check their content
    println!("\n📋 Capturing warmup frames...");
    for i in 0..20 {
        thread::sleep(Duration::from_millis(100));

        match camera.frame() {
            Ok(frame) => {
                let bytes = frame.buffer_bytes();
                let len = bytes.len();

                // Check if frame is all zeros or all same value (gray)
                let first_byte = bytes.first().copied().unwrap_or(0);
                let all_same = bytes.iter().all(|&b| b == first_byte);
                let sum: u64 = bytes.iter().map(|&b| b as u64).sum();
                let avg = sum / len as u64;

                // Sample some pixels
                let sample1 = bytes
                    .get(0..3)
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();
                let sample_mid = bytes
                    .get(len / 2..len / 2 + 3)
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();
                let sample_end = bytes
                    .get(len - 3..)
                    .map(|s| format!("{:?}", s))
                    .unwrap_or_default();

                println!(
                    "   Frame {}: {}x{}, {} bytes, avg={}, all_same={}, samples: {} | {} | {}",
                    i + 1,
                    frame.resolution().width_x,
                    frame.resolution().height_y,
                    len,
                    avg,
                    all_same,
                    sample1,
                    sample_mid,
                    sample_end
                );

                // If we get non-uniform data, save it
                if !all_same && i >= 5 {
                    println!("\n   🎉 Got varied frame data! Saving...");

                    let img = image::RgbImage::from_vec(
                        frame.resolution().width_x,
                        frame.resolution().height_y,
                        bytes.to_vec(),
                    );

                    if let Some(img) = img {
                        let filename = format!("raw_capture_{}.jpg", i);
                        match img.save(&filename) {
                            Ok(_) => println!("   ✅ Saved to {}", filename),
                            Err(e) => println!("   ❌ Save failed: {}", e),
                        }
                    }
                    break;
                }
            }
            Err(e) => {
                println!("   Frame {}: ❌ Error: {}", i + 1, e);
            }
        }
    }

    // Stop stream
    println!("\n📋 Stopping stream...");
    match camera.stop_stream() {
        Ok(_) => println!("   ✅ Stream stopped"),
        Err(e) => println!("   ⚠️  Stop failed: {}", e),
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Test Complete");
    println!("═══════════════════════════════════════════════════════════════");
}
