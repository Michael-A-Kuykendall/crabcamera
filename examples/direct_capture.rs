//! Direct capture test - captures at native resolution with MJPEG decode
//! 
//! Run with: cargo run --example direct_capture

use nokhwa::{Camera, query};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType};
use std::time::Duration;
use std::thread;

fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Direct Camera Capture Test");
    println!("═══════════════════════════════════════════════════════════════\n");

    // List cameras
    println!("📋 Finding cameras...");
    match query(ApiBackend::MediaFoundation) {
        Ok(cameras) => {
            for (i, cam) in cameras.iter().enumerate() {
                println!("   [{}] {}", i, cam.human_name());
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
            return;
        }
    }

    // Create camera at highest resolution (which will give us MJPEG)
    println!("\n📋 Creating camera (native resolution)...");
    let requested_format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = match Camera::new(CameraIndex::Index(0), requested_format) {
        Ok(cam) => {
            println!("   ✅ Format: {:?}", cam.camera_format());
            cam
        }
        Err(e) => {
            println!("   ❌ Failed: {}", e);
            return;
        }
    };

    // Open stream
    println!("\n📋 Opening stream (camera LED should turn on)...");
    if let Err(e) = camera.open_stream() {
        println!("   ❌ Failed: {}", e);
        return;
    }
    println!("   ✅ Stream open");

    // Wait for camera to stabilize
    println!("\n📋 Warming up camera (3 seconds)...");
    thread::sleep(Duration::from_secs(3));

    // Capture a frame
    println!("\n📋 Capturing frame...");
    match camera.frame() {
        Ok(frame) => {
            let bytes = frame.buffer_bytes();
            let width = frame.resolution().width_x;
            let height = frame.resolution().height_y;
            
            println!("   Raw: {}x{}, {} bytes", width, height, bytes.len());
            
            // Check if MJPEG
            if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
                println!("   Format: MJPEG (will decode)");
                
                match image::load_from_memory(&bytes) {
                    Ok(img) => {
                        let rgb = img.to_rgb8();
                        println!("   Decoded: {}x{} RGB", rgb.width(), rgb.height());
                        
                        // Save
                        match rgb.save("direct_capture.jpg") {
                            Ok(_) => println!("   ✅ Saved to direct_capture.jpg"),
                            Err(e) => println!("   ❌ Save failed: {}", e),
                        }
                    }
                    Err(e) => println!("   ❌ Decode failed: {}", e),
                }
            } else {
                println!("   Format: Raw RGB");
                
                // Check if valid
                let nonzero = bytes.iter().filter(|&&b| b != 0).count();
                let pct = (nonzero as f64 / bytes.len() as f64) * 100.0;
                println!("   Non-zero pixels: {:.1}%", pct);
                
                if let Some(img) = image::RgbImage::from_vec(width, height, bytes.to_vec()) {
                    match img.save("direct_capture.jpg") {
                        Ok(_) => println!("   ✅ Saved to direct_capture.jpg"),
                        Err(e) => println!("   ❌ Save failed: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("   ❌ Capture failed: {}", e),
    }

    // Stop
    println!("\n📋 Stopping stream...");
    let _ = camera.stop_stream();
    println!("   ✅ Done!");
    
    println!("\n═══════════════════════════════════════════════════════════════");
}
