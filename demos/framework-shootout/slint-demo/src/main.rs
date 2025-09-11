slint::include_modules!();

use std::time::{Duration, Instant};

fn main() -> Result<(), slint::PlatformError> {
    let ui = CameraControls::new()?;
    
    // Setup callbacks
    let ui_weak = ui.as_weak();
    ui.on_focus_changed(move |value| {
        println!("Focus changed to: {:.1}%", value);
    });
    
    let ui_weak2 = ui.as_weak();
    ui.on_iso_changed(move |value| {
        println!("ISO changed to: {}", value);
    });
    
    let ui_weak3 = ui.as_weak();
    ui.on_exposure_changed(move |value| {
        println!("Exposure changed to: 1/{:.0}s", 1.0 / value);
    });
    
    let ui_weak4 = ui.as_weak();
    ui.on_white_balance_changed(move |value| {
        println!("White balance changed to: {}", value);
    });
    
    let ui_weak5 = ui.as_weak();
    ui.on_capture_photo(move || {
        if let Some(ui) = ui_weak5.upgrade() {
            let count = ui.get_photos_captured() + 1;
            ui.set_photos_captured(count);
            println!("Photo captured! Total: {}", count);
        }
    });
    
    // FPS counter update
    let ui_weak_fps = ui.as_weak();
    let timer = slint::Timer::default();
    let mut last_frame = Instant::now();
    
    timer.start(slint::TimerMode::Repeated, Duration::from_millis(16), move || {
        if let Some(ui) = ui_weak_fps.upgrade() {
            let now = Instant::now();
            let elapsed = now.duration_since(last_frame);
            let fps = 1.0 / elapsed.as_secs_f32();
            ui.set_fps(fps);
            last_frame = now;
        }
    });
    
    ui.run()
}