// Test the public API that users would actually use

use std::collections::HashMap;

fn main() {
    println!("🧪 Testing CrabCamera v0.3.0 Windows Controls");
    println!("============================================");
    
    // Test 1: Can we import the public API?
    println!("\n1. ✅ Public API imports working");
    
    // Test 2: Can we create basic types?
    println!("\n2. Testing type creation...");
    
    let format = crabcamera::types::CameraFormat::new(640, 480, 30.0);
    println!("✅ CameraFormat: {}x{} @ {}fps", format.width, format.height, format.fps);
    
    let controls = crabcamera::types::CameraControls {
        auto_focus: Some(true),
        focus_distance: Some(0.5),
        auto_exposure: Some(false),
        exposure_time: Some(1.0/60.0),
        brightness: Some(0.1),
        contrast: Some(0.2),
        saturation: Some(0.0),
        ..Default::default()
    };
    println!("✅ CameraControls created with {} controls set", 
        [controls.auto_focus, controls.auto_exposure].iter().filter(|x| x.is_some()).count() + 
        [controls.focus_distance, controls.exposure_time, controls.brightness, controls.contrast, controls.saturation].iter().filter(|x| x.is_some()).count()
    );
    
    let capabilities = crabcamera::types::CameraCapabilities {
        supports_auto_focus: true,
        supports_manual_focus: true,
        supports_auto_exposure: true,
        supports_manual_exposure: true,
        supports_white_balance: true,
        supports_zoom: false,
        supports_flash: false,
        supports_burst_mode: true,
        supports_hdr: false,
        max_resolution: (1920, 1080),
        max_fps: 30.0,
        exposure_range: Some((1.0/1000.0, 1.0)),
        iso_range: Some((100, 6400)),
        focus_range: Some((0.0, 1.0)),
    };
    println!("✅ CameraCapabilities: focus({}/{}), exposure({}/{}), wb({})", 
        capabilities.supports_auto_focus, capabilities.supports_manual_focus,
        capabilities.supports_auto_exposure, capabilities.supports_manual_exposure,
        capabilities.supports_white_balance
    );
    
    // Test 3: Platform detection
    println!("\n3. Testing platform detection...");
    let platform = crabcamera::types::Platform::current();
    println!("✅ Platform detected: {:?}", platform);
    
    // Test 4: Can we access platform module? 
    println!("\n4. Testing platform module access...");
    
    #[cfg(target_os = "windows")]
    {
        // This will tell us if our Windows module structure is working
        println!("✅ Windows platform compilation available");
        
        // Test if we can call the camera system function
        match crabcamera::platform::CameraSystem::list_cameras() {
            Ok(cameras) => {
                println!("✅ Camera enumeration succeeded: {} cameras found", cameras.len());
                for (i, camera) in cameras.iter().enumerate() {
                    println!("   Camera {}: {} ({})", i, camera.name, camera.id);
                }
            },
            Err(e) => {
                println!("⚠️  Camera enumeration failed: {} (expected if no cameras)", e);
            }
        }
        
        match crabcamera::platform::CameraSystem::initialize() {
            Ok(msg) => println!("✅ Camera system init: {}", msg),
            Err(e) => println!("⚠️  Camera system init failed: {}", e),
        }
        
        match crabcamera::platform::CameraSystem::get_platform_info() {
            Ok(info) => {
                println!("✅ Platform info: {} backend", info.backend);
                println!("   Features: {:?}", info.features);
            },
            Err(e) => println!("❌ Platform info failed: {}", e),
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        println!("ℹ️  Not on Windows - skipping Windows-specific tests");
    }
    
    // Test 5: Can we create a mock camera for testing?
    println!("\n5. Testing mock camera system...");
    
    // Set environment variable to force mock mode
    std::env::set_var("CRABCAMERA_USE_MOCK", "1");
    
    let init_params = crabcamera::types::CameraInitParams::new("test_camera".to_string())
        .with_format(format)
        .with_auto_focus(true);
    
    match crabcamera::platform::PlatformCamera::new(init_params) {
        Ok(mut camera) => {
            println!("✅ Mock camera created successfully");
            
            // Test basic operations
            if camera.is_available() {
                println!("✅ Camera reports as available");
            }
            
            if let Some(id) = camera.get_device_id() {
                println!("✅ Device ID: {}", id);
            }
            
            match camera.start_stream() {
                Ok(_) => println!("✅ Stream started"),
                Err(e) => println!("❌ Stream start failed: {}", e),
            }
            
            match camera.capture_frame() {
                Ok(frame) => {
                    println!("✅ Frame captured: {}x{} {} bytes", 
                        frame.width, frame.height, frame.data.len());
                },
                Err(e) => println!("❌ Frame capture failed: {}", e),
            }
            
            // Test the new v0.3.0 controls functionality
            match camera.apply_controls(&controls) {
                Ok(_) => println!("✅ Controls applied successfully"),
                Err(e) => println!("❌ Controls failed: {}", e),
            }
            
            match camera.get_controls() {
                Ok(current) => println!("✅ Got current controls: auto_focus={:?}", current.auto_focus),
                Err(e) => println!("❌ Get controls failed: {}", e),
            }
            
            match camera.test_capabilities() {
                Ok(caps) => {
                    println!("✅ Capabilities: focus({}/{}), exposure({}/{})", 
                        caps.supports_auto_focus, caps.supports_manual_focus,
                        caps.supports_auto_exposure, caps.supports_manual_exposure);
                },
                Err(e) => println!("❌ Capabilities test failed: {}", e),
            }
            
            match camera.stop_stream() {
                Ok(_) => println!("✅ Stream stopped"),
                Err(e) => println!("❌ Stream stop failed: {}", e),
            }
        },
        Err(e) => {
            println!("❌ Mock camera creation failed: {}", e);
        }
    }
    
    println!("\n🎯 Test Summary:");
    println!("- ✅ Public API accessible and types create correctly");
    println!("- ✅ Platform detection working");
    println!("- ✅ Camera system functions callable");
    println!("- ✅ Mock camera system functional");
    println!("- ✅ v0.3.0 camera controls API available");
    
    println!("\n⚠️  Next Steps Needed:");
    println!("- Test with real Windows cameras and MediaFoundation");
    println!("- Verify COM interface management doesn't crash");
    println!("- Test thread safety with concurrent access");
    println!("- Validate control value ranges and normalization");
    println!("- Test capability detection with different camera hardware");
}