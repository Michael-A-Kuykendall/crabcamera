// CrabCamera Headless Capture Example
// Demonstrates headless camera capture using the new headless API

use crabcamera::headless::{HeadlessSession, list_devices, list_formats, CaptureConfig, BufferPolicy, AudioMode};
use crabcamera::types::CameraFormat;
use crabcamera::audio::list_audio_devices;
use std::time::Duration;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    println!("🦀 CrabCamera Headless Capture Example");
    println!("=====================================");

    // Step 1: List available devices
    println!("\n🔍 Discovering available cameras...");
    let devices = list_devices()?;
    if devices.is_empty() {
        eprintln!("❌ No cameras found!");
        return Ok(());
    }

    for (i, device) in devices.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, device.name, device.id);
    }

    // Use the first device
    let device = &devices[0];
    println!("\n📷 Using camera: {} ({})", device.name, device.id);

    // Step 2: List audio devices
    println!("\n🎤 Available audio devices...");
    match list_audio_devices() {
        Ok(devices) => {
            if devices.is_empty() {
                println!("  ❌ No audio devices found!");
            } else {
                for (i, dev) in devices.iter().enumerate() {
                    println!("  {}. {} ({})", i + 1, dev.name, dev.id);
                }
            }
        }
        Err(e) => {
            println!("  ❌ Error listing audio devices: {e}");
        }
    }

    // Step 3: List formats
    println!("\n📋 Available formats:");
    let formats = list_formats(&device.id)?;
    if formats.is_empty() {
        eprintln!("❌ No formats found!");
        return Ok(());
    }

    for (i, format) in formats.iter().enumerate() {
        println!("  {}. {}x{}@{} {}", i + 1, format.width, format.height, format.fps, format.format_type);
    }

    // Use the first format
    let format = &formats[0];
    println!("\n🎥 Using format: {}x{}@{} {}", format.width, format.height, format.fps, format.format_type);

    // Step 3: Create capture config
    let config = CaptureConfig {
        device_id: device.id.clone(),
        format: CameraFormat {
            width: format.width,
            height: format.height,
            fps: format.fps,
            format_type: format.format_type.clone(),
        },
        buffer_policy: BufferPolicy::DropOldest { capacity: 10 },
        audio_mode: AudioMode::Enabled,
        audio_device_id: Some("audio_0_87a48c3e".to_string()),
    };

    // Step 4: Open session
    println!("\n🔓 Opening headless session...");
    let session = HeadlessSession::open(config)?;

    // Step 5: Start capture
    println!("▶️  Starting capture...");
    session.start()?;

    // Step 6: Capture some frames
    println!("\n📸 Capturing frames...");
    let mut frame_count = 0;
    let start_time = std::time::Instant::now();

    let mut audio_count = 0;
    let mut frame_saved = false;

    while frame_count < 10 && start_time.elapsed() < Duration::from_secs(10) {
        match session.get_frame(Duration::from_millis(1000)) {
            Ok(Some(frame)) => {
                frame_count += 1;
                println!("  Frame {}: {}x{} {} seq:{} size:{} bytes",
                    frame_count, frame.width, frame.height, frame.format,
                    frame.sequence, frame.data.len());
                
                if !frame_saved {
                    fs::write("captured_frame.raw", &frame.data)?;
                    println!("    💾 Saved frame to captured_frame.raw");
                    frame_saved = true;
                }
            }
            Ok(None) => {
                println!("  Timeout waiting for frame");
            }
            Err(e) => {
                eprintln!("  Error getting frame: {e}");
                break;
            }
        }

        // Try to get audio packet
        match session.get_audio_packet(Duration::from_millis(100)) {
            Ok(Some(packet)) => {
                audio_count += 1;
                println!("  Audio {}: {} samples seq:{} size:{} bytes channels:{}",
                    audio_count, packet.data.len() / 4, packet.sequence, packet.data.len(), packet.channels);
                
use std::fs::OpenOptions;
use std::io::Write;
                let mut file = OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open("captured_audio.raw")?;
                file.write_all(&packet.data)?;
                println!("    🎵 Appended audio to captured_audio.raw");
            }
            Ok(None) => {
                // No audio packet available, continue
            }
            Err(e) => {
                eprintln!("  Error getting audio: {e}");
            }
        }
    }

    let dropped = session.dropped_frames()?;
    println!("\n📊 Captured {frame_count} frames, {audio_count} audio packets, {dropped} dropped");

    // Step 7: Stop capture
    println!("⏹️  Stopping capture...");
    match session.stop(Duration::from_millis(10000)) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Warning: Stop timed out, but session may still be stopping: {e:?}");
        }
    }

    // Step 8: Close session
    println!("🔒 Closing session...");
    session.close(Duration::from_millis(5000))?;

    println!("\n✅ Headless capture example completed successfully!");
    Ok(())
}