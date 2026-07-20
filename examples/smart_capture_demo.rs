//! Smart Capture Demo
//! 
//! Demonstrates the "Smart Trigger" feature that automatically captures
//! when image quality is optimal and stable.

use crabcamera::quality::{SmartTrigger, TriggerConfig, TriggerStatus};
use crabcamera::types::CameraFrame;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🦀 CrabCamera Smart Capture Demo");
    println!("--------------------------------");

    // 1. Configure the Smart Trigger
    let config = TriggerConfig {
        min_quality_score: 0.8,
        min_stability_duration: Duration::from_millis(100),
        timeout: Some(Duration::from_secs(2)),
        required_consecutive_good_frames: 3,
        lock_after_ready: true,
    };

    let mut trigger = SmartTrigger::new(config);
    println!("Waiting for perfect shot...");

    // 2. Simulate a camera stream
    // In a real app, this would be your camera callback loop
    for i in 0..20 {
        // Simulate varying quality (ramping up)
        let quality_sim = (i as f32 / 15.0).min(0.9); 
        
        let frame = create_dummy_frame(quality_sim);
        
        // 3. Process frame through Invariant-backed Smart Trigger
        let (status, report) = trigger.process_frame(&frame);
        
        println!(
            "Frame {}: Quality={:.2} Status={:?} | Blur={:.2} Exposure={:.2}", 
            i, 
            report.score.overall, 
            status,
            report.score.blur,
            report.score.exposure
        );

        match status {
            TriggerStatus::Ready => {
                println!("\n📸 CAPTURE TRIGGERED! Taking the shot.");
                if let Some(best) = trigger.get_best_frame() {
                    println!("Saved frame with {} bytes", best.data.len());
                }
                break;
            }
            TriggerStatus::Timeout => {
                println!("\n⏰ TIMEOUT! Capturing best available.");
                if let Some(best) = trigger.get_best_frame() {
                    println!("Saved backup frame with {} bytes", best.data.len());
                }
                break;
            }
            _ => thread::sleep(Duration::from_millis(50)),
        }
    }
}

// Helper to create dummy frames with predictable "quality"
// (In reality, QualityValidator analyzes pixel data, here we exploit the validator's logic
// or just rely on the fact that solid gray frames have low noise/high stability scores maybe?
// Actually, for the demo to work without real images, we need to know what the validator likes.
// The validator likes reasonable exposure and low noise.
fn create_dummy_frame(target_quality: f32) -> CameraFrame {
    // Determine brightness based on target_quality to trick exposure analyzer
    // 128 is perfect mid-gray (good exposure)
    let brightness = if target_quality > 0.5 { 128 } else { 10 }; 
    
    // Create a generic frame
    let width = 640;
    let height = 480;
    let data = vec![brightness; (width * height * 3) as usize];
    
    CameraFrame::new(data, width, height, "simulated".into())
}
