//! Debug exactly what hardware_audit does

use crabcamera::commands::capture::{
    start_camera_preview, stop_camera_preview, capture_single_photo,
    save_frame_to_disk, save_frame_compressed,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging to see debug output
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    println!("\n=============================================================");
    println!("  Simulating Hardware Audit Flow");
    println!("=============================================================\n");

    let device_id = "0".to_string();
    
    // Step 1: Start preview (this creates camera in registry)
    println!("[1] start_camera_preview({}, None)...", device_id);
    match start_camera_preview(device_id.clone(), None).await {
        Ok(msg) => println!("    OK: {}", msg),
        Err(e) => println!("    ERROR: {}", e),
    }
    
    // Wait for warmup
    println!("\n[2] Waiting 3 seconds for warmup...");
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    // Stop preview
    println!("\n[3] stop_camera_preview({})...", device_id);
    match stop_camera_preview(device_id.clone()).await {
        Ok(msg) => println!("    OK: {}", msg),
        Err(e) => println!("    ERROR: {}", e),
    }
    
    // Now capture - this reuses the registry camera
    println!("\n[4] capture_single_photo({}, None)...", device_id);
    match capture_single_photo(Some(device_id.clone()), None).await {
        Ok(frame) => {
            println!("    OK: {}x{}, {} bytes", frame.width, frame.height, frame.size_bytes);
            
            // Check first few bytes
            if frame.data.len() >= 3 {
                println!("    First 3 bytes: {:02X} {:02X} {:02X}", 
                    frame.data[0], frame.data[1], frame.data[2]);
            }
            
            // Save raw 
            println!("\n[5] Saving raw frame...");
            match save_frame_to_disk(frame.clone(), "debug_raw.png".to_string()).await {
                Ok(msg) => println!("    OK: {}", msg),
                Err(e) => println!("    ERROR: {}", e),
            }
            
            // Save compressed
            println!("\n[6] Saving compressed frame...");
            match save_frame_compressed(frame.clone(), "debug_compressed.jpg".to_string(), Some(85)).await {
                Ok(msg) => println!("    OK: {}", msg),
                Err(e) => println!("    ERROR: {}", e),
            }
        }
        Err(e) => println!("    ERROR: {}", e),
    }
    
    println!("\n=============================================================\n");
    Ok(())
}
