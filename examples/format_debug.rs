//! Debug camera format negotiation

use nokhwa::{
    pixel_format::RgbFormat,
    query,
    utils::{ApiBackend, CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=============================================================");
    println!("  Camera Format Debug");
    println!("=============================================================\n");

    // List cameras first
    let cameras = query(ApiBackend::MediaFoundation)?;
    println!("Found {} cameras:", cameras.len());
    for cam in &cameras {
        println!("  [{}] {}", cam.index(), cam.human_name());
    }

    // Try to create camera with AbsoluteHighestResolution
    println!("\n[1] Creating camera with AbsoluteHighestResolution...");
    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
    let mut camera = Camera::new(CameraIndex::Index(0), requested)?;

    // Check what format was negotiated BEFORE opening stream
    let format = camera.camera_format();
    println!("    Negotiated format (before stream): {:?}", format);

    // Open stream
    println!("\n[2] Opening stream...");
    camera.open_stream()?;

    // Check format AFTER opening stream
    let format = camera.camera_format();
    println!("    Format (after stream open): {:?}", format);

    // Capture a frame
    println!("\n[3] Capturing frame...");
    let frame = camera.frame()?;
    println!(
        "    Frame resolution: {}x{}",
        frame.resolution().width_x,
        frame.resolution().height_y
    );
    println!("    Frame bytes: {}", frame.buffer().len());
    println!("    Frame source format: {:?}", frame.source_frame_format());

    // Check if MJPEG header present
    let buffer = frame.buffer();
    if buffer.len() >= 3 {
        let is_mjpeg = buffer[0] == 0xFF && buffer[1] == 0xD8 && buffer[2] == 0xFF;
        println!("    Has MJPEG header: {}", is_mjpeg);
    }

    // Stop stream
    camera.stop_stream()?;

    println!("\n=============================================================\n");
    Ok(())
}
