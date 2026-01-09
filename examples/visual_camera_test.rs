//! Visual Camera Test Example
//!
//! This example captures real frames from your camera and saves them as JPEG images
//! so you can VISUALLY verify that the camera hardware is working.
//!
//! Run this to see actual camera output saved to disk.

use crabcamera::commands::init::{get_available_cameras, initialize_camera_system};
use crabcamera::platform::PlatformCamera;
use crabcamera::types::{CameraFormat, CameraInitParams};
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦀 CrabCamera VISUAL Camera Test");
    println!("================================");
    println!("This will capture REAL frames from your camera and save them as images!");
    println!("You'll see actual video output, not just console spam.\n");

    // Create output directory
    let output_dir = "camera_test_output";
    fs::create_dir_all(output_dir)?;
    println!("📁 Created output directory: {}", output_dir);

    // Initialize camera system
    println!("📷 Initializing camera system...");
    initialize_camera_system().await?;

    // Get available cameras
    let cameras = get_available_cameras().await?;
    if cameras.is_empty() {
        println!("❌ No cameras found!");
        return Ok(());
    }

    println!("📷 Available cameras:");
    for (i, camera) in cameras.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, camera.name, camera.id);
    }

    // Use first camera
    let device_id = cameras[0].id.clone();

    println!("\n📸 Testing DIRECT camera capture...");
    println!("Device ID: {}", device_id);

    // Create camera parameters - let it choose the best format
    let camera_params = CameraInitParams {
        device_id: device_id.clone(),
        format: CameraFormat {
            width: 1280,
            height: 720,
            fps: 30.0,
            format_type: "MJPEG".to_string(), // Request MJPEG
        },
        controls: Default::default(),
    };

    // Initialize camera directly
    println!("🔧 Initializing camera with params: 1280x720 @ 30fps");
    let mut camera = match PlatformCamera::new(camera_params) {
        Ok(cam) => {
            println!("✅ Camera initialized successfully");
            cam
        }
        Err(e) => {
            println!("❌ Failed to initialize camera: {}", e);
            println!("💡 This indicates a real camera hardware/driver issue!");
            return Ok(());
        }
    };

    // Start camera stream
    println!("🚀 Starting camera stream...");
    match camera.start_stream() {
        Ok(_) => println!("✅ Camera stream started successfully"),
        Err(e) => {
            println!("❌ Failed to start camera stream: {}", e);
            println!("💡 Camera hardware problem detected!");
            return Ok(());
        }
    }

    println!("\n📸 CAPTURING REAL CAMERA FRAMES...");
    println!("==================================");

    // Capture several frames and save them
    for frame_num in 1..=5 {
        println!("📸 Capturing frame {}...", frame_num);

        // Capture frame
        match camera.capture_frame() {
            Ok(frame) => {
                println!(
                    "   ✅ Captured frame {}: {}x{} bytes",
                    frame_num, frame.width, frame.height
                );

                // Convert RGB data to JPEG
                let filename = format!("{}/camera_frame_{}.jpg", output_dir, frame_num);
                match save_rgb_as_jpeg(&frame.data, frame.width, frame.height, &filename) {
                    Ok(_) => println!("   💾 Saved camera frame as JPEG: {}", filename),
                    Err(e) => println!("   ❌ Failed to save JPEG: {}", e),
                }
            }
            Err(e) => {
                println!("   ❌ Failed to capture frame {}: {}", frame_num, e);
            }
        }

        // Wait between captures
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    println!("\n🛑 Stopping camera...");
    // Camera will be automatically stopped when dropped

    println!("\n🎉 TEST COMPLETE!");
    println!("=================");
    println!(
        "📁 Check the '{}' directory for actual camera images!",
        output_dir
    );
    println!("🖼️  If you see real camera footage in the JPEG files, your camera hardware works!");
    println!("✅ This proves the camera capture pipeline is functional.");
    println!("\n🔗 Next: Try recording video with 'cargo run --example record_video'");

    Ok(())
}

// Helper function to save RGB data as JPEG
fn save_rgb_as_jpeg(
    rgb_data: &[u8],
    width: u32,
    height: u32,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create RGB image from raw bytes
    let img = image::RgbImage::from_raw(width, height, rgb_data.to_vec())
        .ok_or("Failed to create image from RGB data")?;

    // Convert to DynamicImage and save as JPEG
    let dynamic_img = image::DynamicImage::ImageRgb8(img);
    dynamic_img.save(filename)?;

    Ok(())
}
