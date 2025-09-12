// CrabCamera Preview Example
// Demonstrates how to use start_camera_preview, stop_camera_preview and get camera frames

use crabcamera::commands::{
    init::{get_available_cameras, initialize_camera_system},
    capture::{start_camera_preview, stop_camera_preview, capture_single_photo, release_camera}
};
use crabcamera::types::CameraFormat;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    
    println!("🦀 CrabCamera Preview Example");
    println!("==============================");
    
    // Step 1: Initialize the camera system
    println!("\n📷 Initializing camera system...");
    match initialize_camera_system().await {
        Ok(message) => println!("✅ {}", message),
        Err(e) => {
            eprintln!("❌ Failed to initialize camera system: {}", e);
            return Err(e.into());
        }
    }
    
    // Step 2: Get available cameras
    println!("\n🔍 Discovering available cameras...");
    let cameras = match get_available_cameras().await {
        Ok(cameras) => cameras,
        Err(e) => {
            eprintln!("❌ Failed to get cameras: {}", e);
            return Err(e.into());
        }
    };
    
    if cameras.is_empty() {
        eprintln!("❌ No cameras found!");
        return Ok(());
    }
    
    for (i, camera) in cameras.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, camera.name, camera.id);
        println!("     Platform: {:?}, Available: {}", camera.platform, camera.is_available);
    }
    
    // Step 3: Use the first available camera
    let camera = &cameras[0];
    let device_id = camera.id.clone();
    println!("\n🎯 Using camera: {} (ID: {})", camera.name, device_id);
    
    // Step 4: Start camera preview
    println!("\n▶️  Starting camera preview...");
    let format = CameraFormat::standard(); // 1280x720 @ 30fps
    match start_camera_preview(device_id.clone(), Some(format)).await {
        Ok(message) => println!("✅ {}", message),
        Err(e) => {
            eprintln!("❌ Failed to start preview: {}", e);
            return Err(e.into());
        }
    }
    
    println!("📹 Preview is now running! Camera stream is active.");
    println!("⏰ Waiting 3 seconds before capturing frames...");
    sleep(Duration::from_secs(3)).await;
    
    // Step 5: Capture some frames while preview is running
    println!("\n📸 Capturing frames from active preview stream...");
    for i in 1..=5 {
        match capture_single_photo(Some(device_id.clone()), None).await {
            Ok(frame) => {
                println!("  Frame {}: {}x{} pixels ({} bytes) at {}", 
                    i, frame.width, frame.height, frame.size_bytes, frame.timestamp.format("%H:%M:%S"));
            }
            Err(e) => {
                eprintln!("  ❌ Failed to capture frame {}: {}", i, e);
            }
        }
        
        // Small delay between frames
        sleep(Duration::from_millis(500)).await;
    }
    
    println!("\n⏰ Preview running for 5 more seconds...");
    sleep(Duration::from_secs(5)).await;
    
    // Step 6: Stop camera preview
    println!("\n⏹️  Stopping camera preview...");
    match stop_camera_preview(device_id.clone()).await {
        Ok(message) => println!("✅ {}", message),
        Err(e) => {
            eprintln!("❌ Failed to stop preview: {}", e);
        }
    }
    
    // Step 7: Release camera resources
    println!("\n🗑️  Releasing camera resources...");
    match release_camera(device_id.clone()).await {
        Ok(message) => println!("✅ {}", message),
        Err(e) => {
            eprintln!("❌ Failed to release camera: {}", e);
        }
    }
    
    println!("\n🎉 Example completed!");
    println!("\n💡 Key Points:");
    println!("   • start_camera_preview() starts the camera stream");
    println!("   • Camera remains active for continuous capture");
    println!("   • capture_single_photo() gets frames from active stream");
    println!("   • stop_camera_preview() stops the stream"); 
    println!("   • release_camera() cleans up all resources");
    
    Ok(())
}

/* 
HOW TO RUN:
===========

1. Add this example to Cargo.toml:
   [[example]]
   name = "camera_preview"
   path = "examples/camera_preview.rs"

2. Run the example:
   cargo run --example camera_preview --all-features

3. Expected output:
   - Camera system initializes
   - Available cameras listed
   - Preview starts (camera LED turns on)
   - 5 frames captured while preview runs
   - Preview stops (camera LED turns off)
   - Resources cleaned up

GETTING CAMERA FRAME STREAM:
============================

The camera frame stream works like this:

1. start_camera_preview() → Activates camera hardware
2. Camera continuously captures frames into internal buffer
3. capture_single_photo() → Gets latest frame from buffer
4. stop_camera_preview() → Deactivates camera hardware

For real-time streaming, call capture_single_photo() in a loop
while preview is active. Each call returns the most recent frame.
*/