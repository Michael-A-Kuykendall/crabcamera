//! Minimal test - what turns on the OBSBOT camera?
//!
//! Run with: cargo run --example camera_on_test

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::{query, Camera};
use std::thread;
use std::time::Duration;

fn main() {
    println!("Testing what turns on the OBSBOT camera...\n");

    // Step 1: Query - does this turn it on?
    println!("STEP 1: query() - checking cameras...");
    let _ = query(ApiBackend::MediaFoundation);
    println!("   Camera LED on? (wait 2 sec)");
    thread::sleep(Duration::from_secs(2));

    // Step 2: Camera::new() - does this turn it on?
    println!("\nSTEP 2: Camera::new() - creating camera object...");
    let requested_format =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(CameraIndex::Index(0), requested_format).unwrap();
    println!("   Camera LED on? (wait 2 sec)");
    thread::sleep(Duration::from_secs(2));

    // Step 3: open_stream() - does this turn it on?
    println!("\nSTEP 3: open_stream() - opening stream...");
    camera.open_stream().unwrap();
    println!("   Camera LED on? (wait 2 sec)");
    thread::sleep(Duration::from_secs(2));

    // Step 4: frame() - capture
    println!("\nSTEP 4: frame() - capturing...");
    let frame = camera.frame().unwrap();
    println!("   Got frame: {} bytes", frame.buffer_bytes().len());
    println!(
        "   Resolution: {}x{}",
        frame.resolution().width_x,
        frame.resolution().height_y
    );

    // Check the data
    let bytes = frame.buffer_bytes();
    let first_3: Vec<u8> = bytes.iter().take(3).copied().collect();
    println!("   First 3 bytes: {:?}", first_3);

    // Is it JPEG (FFD8FF) or RGB data?
    if first_3 == vec![255, 216, 255] {
        println!("   ⚠️  Data is MJPEG (not decoded to RGB!)");

        // Try to decode it ourselves
        println!("\n   Attempting manual JPEG decode...");
        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let rgb = img.to_rgb8();
                println!("   ✅ Decoded to RGB: {}x{}", rgb.width(), rgb.height());

                // Save it
                rgb.save("camera_on_test.jpg").unwrap();
                println!("   ✅ Saved to camera_on_test.jpg");
            }
            Err(e) => println!("   ❌ Decode failed: {}", e),
        }
    } else {
        println!("   Data appears to be RGB (first byte not 0xFF)");

        // Save as RGB
        if let Some(img) = image::RgbImage::from_vec(
            frame.resolution().width_x,
            frame.resolution().height_y,
            bytes.to_vec(),
        ) {
            img.save("camera_on_test.jpg").unwrap();
            println!("   ✅ Saved to camera_on_test.jpg");
        }
    }

    // Cleanup
    println!("\nSTEP 5: stop_stream()...");
    let _ = camera.stop_stream();
    println!("   Done!");
}
