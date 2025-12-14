//! Quick Camera Test - Run this to verify CrabCamera works with your hardware
//! 
//! Run with: cargo run --example quick_test
//!
//! This will:
//! 1. List all cameras on your system
//! 2. Start the camera stream (warm-up)
//! 3. Take a photo with the first camera
//! 4. Save it to disk
//! 5. Show system diagnostics

use crabcamera::commands::init::{
    get_available_cameras, initialize_camera_system, get_system_diagnostics
};
use crabcamera::commands::capture::{
    capture_single_photo, save_frame_compressed, 
    start_camera_preview, stop_camera_preview
};
use crabcamera::commands::advanced::{get_camera_controls, test_camera_capabilities};
use crabcamera::types::CameraFormat;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  🦀 CrabCamera Quick Test - v{}", crabcamera::VERSION);
    println!("═══════════════════════════════════════════════════════════════\n");

    // Initialize
    println!("📋 STEP 1: Initialize Camera System");
    println!("─────────────────────────────────────");
    match initialize_camera_system().await {
        Ok(msg) => println!("   ✅ {}\n", msg),
        Err(e) => {
            println!("   ❌ Failed: {}\n", e);
            return;
        }
    }

    // System Diagnostics
    println!("📋 STEP 2: System Diagnostics");
    println!("─────────────────────────────────────");
    match get_system_diagnostics().await {
        Ok(diag) => {
            println!("   Version:    {}", diag.crate_version);
            println!("   Platform:   {}", diag.platform);
            println!("   Backend:    {}", diag.backend);
            println!("   Cameras:    {}", diag.camera_count);
            println!("   Permission: {}", diag.permission_status);
            println!("   Features:   {:?}\n", diag.features_enabled);
        }
        Err(e) => println!("   ⚠️  Could not get diagnostics: {}\n", e),
    }

    // List cameras
    println!("📋 STEP 3: Discover Cameras");
    println!("─────────────────────────────────────");
    let cameras = match get_available_cameras().await {
        Ok(cams) => {
            if cams.is_empty() {
                println!("   ❌ No cameras found! Is a webcam connected?\n");
                return;
            }
            for (i, cam) in cams.iter().enumerate() {
                println!("   [{}] {} (ID: {})", i, cam.name, cam.id);
                println!("       Available: {}", cam.is_available);
                println!("       Formats: {} supported", cam.supports_formats.len());
                if let Some(best) = cam.supports_formats.first() {
                    println!("       Best: {}x{} @ {}fps", best.width, best.height, best.fps);
                }
            }
            println!();
            cams
        }
        Err(e) => {
            println!("   ❌ Failed to list cameras: {}\n", e);
            return;
        }
    };

    let camera = &cameras[0];
    let device_id = camera.id.clone();

    // Test camera capabilities
    println!("📋 STEP 4: Test Camera Capabilities");
    println!("─────────────────────────────────────");
    match test_camera_capabilities(device_id.clone()).await {
        Ok(caps) => {
            println!("   Auto Focus:     {}", if caps.supports_auto_focus { "✅" } else { "❌" });
            println!("   Manual Focus:   {}", if caps.supports_manual_focus { "✅" } else { "❌" });
            println!("   Auto Exposure:  {}", if caps.supports_auto_exposure { "✅" } else { "❌" });
            println!("   Manual Exposure:{}", if caps.supports_manual_exposure { "✅" } else { "❌" });
            println!("   White Balance:  {}", if caps.supports_white_balance { "✅" } else { "❌" });
            println!();
        }
        Err(e) => println!("   ⚠️  Could not test capabilities: {}\n", e),
    }

    // Get current controls
    println!("📋 STEP 5: Current Camera Controls");
    println!("─────────────────────────────────────");
    match get_camera_controls(device_id.clone()).await {
        Ok(controls) => {
            println!("   Auto Focus:    {:?}", controls.auto_focus);
            println!("   Auto Exposure: {:?}", controls.auto_exposure);
            println!("   Brightness:    {:?}", controls.brightness);
            println!("   Contrast:      {:?}", controls.contrast);
            println!();
        }
        Err(e) => println!("   ⚠️  Could not get controls: {}\n", e),
    }

    // Start camera stream to warm it up
    println!("📋 STEP 6: Start Camera Stream (warm-up)");
    println!("─────────────────────────────────────");
    let format = CameraFormat::standard();
    println!("   Using format: {}x{} @ {}fps", format.width, format.height, format.fps);
    
    match start_camera_preview(device_id.clone(), Some(format.clone())).await {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(e) => {
            println!("   ❌ Failed to start stream: {}", e);
            return;
        }
    }
    
    println!("   ⏳ Warming up camera (2 seconds)...");
    sleep(Duration::from_secs(2)).await;
    println!("   ✅ Camera warmed up!\n");

    // Capture a photo
    println!("📋 STEP 7: Capture Test Photo");
    println!("─────────────────────────────────────");
    
    match capture_single_photo(Some(device_id.clone()), Some(format)).await {
        Ok(frame) => {
            println!("   ✅ Captured frame!");
            println!("      Size: {}x{} pixels", frame.width, frame.height);
            println!("      Data: {} bytes", frame.size_bytes);
            println!("      Time: {}", frame.timestamp);
            
            // Save to disk as proper JPEG
            let filename = format!("test_capture_{}.jpg", 
                chrono::Utc::now().format("%Y%m%d_%H%M%S"));
            println!("\n   💾 Saving to {}...", filename);
            
            match save_frame_compressed(frame, filename.clone(), Some(90)).await {
                Ok(_) => println!("   ✅ Saved! Check the current directory for {}", filename),
                Err(e) => println!("   ⚠️  Could not save: {}", e),
            }
        }
        Err(e) => println!("   ❌ Capture failed: {}", e),
    }

    // Stop the camera stream
    println!("\n📋 STEP 8: Stop Camera Stream");
    println!("─────────────────────────────────────");
    match stop_camera_preview(device_id.clone()).await {
        Ok(msg) => println!("   ✅ {}", msg),
        Err(e) => println!("   ⚠️  {}", e),
    }

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  🦀 Test Complete!");
    println!("═══════════════════════════════════════════════════════════════");
}
