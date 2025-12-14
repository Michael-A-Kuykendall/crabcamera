//! Debug camera reuse behavior

use nokhwa::{Camera, pixel_format::RgbFormat, utils::{RequestedFormat, RequestedFormatType, CameraIndex}};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=============================================================");
    println!("  Camera Reuse Debug");
    println!("=============================================================\n");

    // Create first camera
    println!("[1] Creating camera (first time)...");
    let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(CameraIndex::Index(0), requested)?;
    println!("    Format: {:?}", camera.camera_format());
    
    // Open stream
    println!("[2] Opening stream...");
    camera.open_stream()?;
    
    // Capture frame
    let frame = camera.frame()?;
    println!("    Frame 1: {}x{}, {} bytes", 
        frame.resolution().width_x, frame.resolution().height_y, frame.buffer().len());
    
    // Close stream 
    println!("[3] Closing stream...");
    camera.stop_stream()?;
    
    // Sleep a bit
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Reopen stream (simulating registry reuse)
    println!("[4] Reopening stream on SAME camera object...");
    camera.open_stream()?;
    
    // Capture another frame
    let frame = camera.frame()?;
    println!("    Frame 2: {}x{}, {} bytes",
        frame.resolution().width_x, frame.resolution().height_y, frame.buffer().len());
    
    // Check MJPEG header
    let buffer = frame.buffer();
    if buffer.len() >= 3 {
        println!("    Has MJPEG header: {}", buffer[0] == 0xFF && buffer[1] == 0xD8);
    }
    
    camera.stop_stream()?;
    
    // Now try creating a SECOND camera object for same device
    println!("\n[5] Creating NEW camera object for same device...");
    let requested2 = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let camera2_result = Camera::new(CameraIndex::Index(0), requested2);
    
    match camera2_result {
        Ok(mut camera2) => {
            println!("    Second camera created! Format: {:?}", camera2.camera_format());
            camera2.open_stream()?;
            let frame = camera2.frame()?;
            println!("    Frame 3: {}x{}, {} bytes",
                frame.resolution().width_x, frame.resolution().height_y, frame.buffer().len());
            camera2.stop_stream()?;
        }
        Err(e) => {
            println!("    ERROR creating second camera: {}", e);
        }
    }
    
    println!("\n=============================================================\n");
    Ok(())
}
