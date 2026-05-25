
use crabcamera::constants::{DEMO_DEFAULT_FPS, DEMO_DEFAULT_HEIGHT, DEMO_DEFAULT_WIDTH};
use crabcamera::platform::{CameraSystem, PlatformCamera};
use crabcamera::types::{CameraInitParams, CameraFormat};
use slint::{Image, SharedPixelBuffer, Rgb8Pixel, ComponentHandle};
use std::sync::{Arc, Mutex};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ui = CameraControls::new()?;

    // Initialize camera system
    let cameras = CameraSystem::list_cameras().unwrap_or_default();
    println!("Found {} cameras", cameras.len());
    
    // Pick first camera if available
    let camera_arc: Arc<Mutex<Option<PlatformCamera>>> = Arc::new(Mutex::new(None));
    
    if let Some(info) = cameras.first() {
        println!("Initializing camera: {}", info.name);
        
        let params = CameraInitParams::new(info.id.clone())
            .with_format(CameraFormat::new(DEMO_DEFAULT_WIDTH, DEMO_DEFAULT_HEIGHT, DEMO_DEFAULT_FPS));

        match PlatformCamera::new(params) {
            Ok(mut cam) => {
                 let ui_frame_handle = ui.as_weak();
                 
                 // process frame callback
                 cam.frame_callback(move |frame| {
                     let width = frame.width;
                     let height = frame.height;
                     let data = frame.data;
                     
                     if data.len() as u32 != width * height * 3 {
                         return; // Skip invalid frames
                     }

                     let buffer = SharedPixelBuffer::<Rgb8Pixel>::clone_from_slice(
                         &data,
                         width, height
                     );
                     
                     let _ = ui_frame_handle.upgrade_in_event_loop(move |ui| {
                        let image = Image::from_rgb8(buffer);
                        ui.set_frame(image);
                        ui.set_camera_connected(true);
                     });
                 }).expect("Failed to set callback");

                 cam.start_stream().expect("Failed to start stream");
                 *camera_arc.lock().unwrap() = Some(cam);
                 ui.set_camera_connected(true);
            },
            Err(e) => {
                println!("Failed to open camera: {}", e);
                ui.set_camera_connected(false);
            }
        }
    } else {
        println!("No cameras found!");
        ui.set_camera_connected(false);
    }

    ui.run()?;
    
    // Cleanup
    if let Some(mut cam) = camera_arc.lock().unwrap().take() {
        let _ = cam.stop_stream();
    }
    
    Ok(())
}
