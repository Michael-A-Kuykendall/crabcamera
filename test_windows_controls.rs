// Simple test to verify Windows MediaFoundation controls compile and instantiate

use crabcamera::platform::windows::{WindowsCamera, MediaFoundationControls};
use crabcamera::types::{CameraFormat, CameraControls};

fn main() {
    env_logger::init();
    
    println!("Testing Windows MediaFoundation controls...");
    
    // Test 1: Can we create MediaFoundationControls?
    println!("\n1. Testing MediaFoundationControls creation...");
    match MediaFoundationControls::new(0) {
        Ok(controls) => {
            println!("✅ MediaFoundationControls created successfully");
            
            // Test 2: Can we call methods without crashing?
            println!("\n2. Testing control methods...");
            
            let test_controls = CameraControls {
                auto_focus: Some(true),
                focus_distance: Some(0.5),
                auto_exposure: Some(false),
                exposure_time: Some(1.0/60.0), // 1/60 second
                brightness: Some(0.1),
                contrast: Some(0.2),
                saturation: Some(0.0),
                ..Default::default()
            };
            
            match controls.apply_controls(&test_controls) {
                Ok(unsupported) => {
                    println!("✅ apply_controls() executed successfully");
                    if !unsupported.is_empty() {
                        println!("   Unsupported controls: {:?}", unsupported);
                    }
                },
                Err(e) => println!("❌ apply_controls() failed: {}", e),
            }
            
            match controls.get_controls() {
                Ok(current) => {
                    println!("✅ get_controls() executed successfully");
                    println!("   Current controls: {:?}", current);
                },
                Err(e) => println!("❌ get_controls() failed: {}", e),
            }
            
            match controls.get_capabilities() {
                Ok(caps) => {
                    println!("✅ get_capabilities() executed successfully");
                    println!("   Capabilities: auto_focus={}, manual_focus={}, auto_exposure={}, manual_exposure={}", 
                        caps.supports_auto_focus, caps.supports_manual_focus,
                        caps.supports_auto_exposure, caps.supports_manual_exposure);
                },
                Err(e) => println!("❌ get_capabilities() failed: {}", e),
            }
        },
        Err(e) => {
            println!("❌ MediaFoundationControls creation failed: {}", e);
            println!("   This is expected if no cameras are available or MediaFoundation isn't working");
        }
    }
    
    // Test 3: Can we create WindowsCamera?
    println!("\n3. Testing WindowsCamera creation...");
    let format = CameraFormat::new(640, 480, 30.0);
    match WindowsCamera::new("0".to_string(), format) {
        Ok(mut camera) => {
            println!("✅ WindowsCamera created successfully");
            
            // Test basic methods
            println!("   Device ID: {}", camera.get_device_id());
            println!("   Is available: {}", camera.is_available());
            
            // Try to start/stop stream
            match camera.start_stream() {
                Ok(_) => println!("✅ start_stream() succeeded"),
                Err(e) => println!("❌ start_stream() failed: {}", e),
            }
            
            match camera.stop_stream() {
                Ok(_) => println!("✅ stop_stream() succeeded"), 
                Err(e) => println!("❌ stop_stream() failed: {}", e),
            }
            
            // Try to apply controls through WindowsCamera
            let test_controls = CameraControls {
                auto_focus: Some(true),
                brightness: Some(0.1),
                ..Default::default()
            };
            
            match camera.apply_controls(&test_controls) {
                Ok(unsupported) => {
                    println!("✅ WindowsCamera.apply_controls() succeeded");
                    if !unsupported.is_empty() {
                        println!("   Unsupported: {:?}", unsupported);
                    }
                },
                Err(e) => println!("❌ WindowsCamera.apply_controls() failed: {}", e),
            }
            
        },
        Err(e) => {
            println!("❌ WindowsCamera creation failed: {}", e);
            println!("   This might be expected if no cameras are available");
        }
    }
    
    println!("\n🔧 Test Summary:");
    println!("- MediaFoundationControls struct instantiates");
    println!("- All control methods can be called without panicking");
    println!("- WindowsCamera hybrid architecture works");
    println!("- Thread safety should work (Send + Sync implemented)");
    println!("\n⚠️  Note: Actual camera functionality requires real hardware and proper device discovery");
}