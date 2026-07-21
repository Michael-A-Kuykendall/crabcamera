//! Full Live A/V Recording Test with OBSBOT Camera
//!
//! This example captures real video from the OBSBOT camera and real audio
//! from its microphone, muxes them into an MP4 file, and validates the output.
//!
//! Run with: cargo run --example live_av_recording --features "recording,audio"

use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 CrabCamera Live A/V Recording Test");
    println!("=====================================");
    println!();

    #[cfg(not(all(feature = "recording", feature = "audio")))]
    {
        println!("⚠️  Requires --features \"recording,audio\"");
        return Ok(());
    }

    #[cfg(all(feature = "recording", feature = "audio"))]
    {
        use crabcamera::audio::list_audio_devices;
        use crabcamera::commands::capture::{
            capture_single_photo, release_camera, start_camera_preview, stop_camera_preview,
        };
        use crabcamera::commands::init::{get_available_cameras, initialize_camera_system};
        use crabcamera::recording::{AudioConfig, Recorder, RecordingConfig};
        use crabcamera::types::CameraFormat;

        // Step 1: Initialize camera system
        println!("📷 Step 1: Initialize Camera System");
        println!("-----------------------------------");
        initialize_camera_system().await?;
        println!("   ✅ Camera system initialized");

        // Step 2: Find camera
        println!();
        println!("🔍 Step 2: Camera Discovery");
        println!("---------------------------");
        let cameras = get_available_cameras().await?;
        if cameras.is_empty() {
            println!("   ❌ No cameras found!");
            return Err("No cameras found".into());
        }

        let camera = &cameras[0];
        let device_id = camera.id.clone();
        println!("   ✅ Found: {} (ID: {})", camera.name, device_id);

        // Step 3: Find audio device
        println!();
        println!("🎤 Step 3: Audio Device Discovery");
        println!("---------------------------------");
        let audio_devices = list_audio_devices()?;
        if audio_devices.is_empty() {
            println!("   ⚠️  No audio devices - recording video only");
        } else {
            for dev in &audio_devices {
                let marker = if dev.is_default { " [DEFAULT]" } else { "" };
                println!("   Found: {}{}", dev.name, marker);
            }
        }

        let use_audio = !audio_devices.is_empty();
        let audio_device = audio_devices
            .iter()
            .find(|d| d.is_default)
            .or(audio_devices.first());

        // Step 4: Start camera preview at 1280x720
        println!();
        println!("📹 Step 4: Start Camera");
        println!("-----------------------");

        let format = CameraFormat::standard(); // 1280x720 @ 30fps
        start_camera_preview(device_id.clone(), Some(format.clone())).await?;
        println!(
            "   ✅ Camera preview started at {}x{}",
            format.width, format.height
        );

        // Let camera warm up
        sleep(tokio::time::Duration::from_millis(500)).await;

        // Get a test frame to confirm it works
        let test_frame = capture_single_photo(Some(device_id.clone()), None).await?;
        println!(
            "   ✅ Test frame captured: {}x{}",
            test_frame.width, test_frame.height
        );

        // Step 5: Setup recording
        println!();
        println!("🎬 Step 5: Setup Recording");
        println!("--------------------------");

        let output_path = std::path::PathBuf::from("live_av_recording.mp4");

        let mut config = RecordingConfig::new(test_frame.width, test_frame.height, 30.0);

        if use_audio {
            if let Some(device) = audio_device {
                println!(
                    "   Audio: {} @ {} Hz, {} ch",
                    device.name, device.sample_rate, device.channels
                );
                config = config.with_audio(AudioConfig {
                    device_id: Some(device.id.clone()),
                    sample_rate: device.sample_rate,
                    channels: device.channels,
                    bitrate: 128_000,
                });
            }
        } else {
            println!("   Audio: disabled (no devices)");
        }

        println!("   Output: {:?}", output_path);

        let mut recorder = Recorder::new(&output_path, config)?;
        println!("   ✅ Recorder initialized");

        // Step 6: Record for 5 seconds
        println!();
        println!("🔴 Step 6: Recording (5 seconds)");
        println!("--------------------------------");
        println!("   🎤 Talk into your mic! 📷 Wave at the camera!");
        println!();

        let duration = Duration::from_secs(3); // Shorter for 4K
        let start = Instant::now();
        let mut frame_count = 0u64;

        while start.elapsed() < duration {
            // Capture frame from camera (no sleep - grab as fast as possible)
            match capture_single_photo(Some(device_id.clone()), None).await {
                Ok(frame) => {
                    // Write to recorder
                    recorder.write_frame(&frame)?;
                    frame_count += 1;

                    if frame_count.is_multiple_of(15) {
                        let elapsed = start.elapsed().as_secs_f64();
                        let fps = frame_count as f64 / elapsed;
                        print!(
                            "\r   🔴 Recording: {:.1}s | {} frames | {:.1} fps    ",
                            elapsed, frame_count, fps
                        );
                        std::io::Write::flush(&mut std::io::stdout())?;
                    }
                }
                Err(e) => {
                    println!("\n   ⚠️  Frame error: {}", e);
                }
            }
        }

        println!("\n   ⏹️  Recording stopped");

        // Step 7: Finalize
        println!();
        println!("📦 Step 7: Finalize Recording");
        println!("-----------------------------");

        let stats = recorder.finish()?;
        println!("   Video frames: {}", stats.video_frames);
        println!(
            "   Bytes written: {} ({:.1} KB)",
            stats.bytes_written,
            stats.bytes_written as f64 / 1024.0
        );

        // Step 8: Validate file
        println!();
        println!("✅ Step 8: Validate Output");
        println!("--------------------------");

        let file_data = std::fs::read(&output_path)?;
        let file_size = file_data.len();
        println!(
            "   File size: {} bytes ({:.1} KB)",
            file_size,
            file_size as f64 / 1024.0
        );

        // Check MP4 signature (ftyp box)
        if file_data.len() >= 8 && &file_data[4..8] == b"ftyp" {
            println!("   ✅ Valid MP4 header (ftyp box)");
        } else {
            println!("   ❌ Invalid MP4 header");
        }

        // Check for moov box (metadata)
        let moov_found = file_data.windows(4).any(|w| w == b"moov");
        if moov_found {
            println!("   ✅ Has moov box (metadata)");
        }

        // Check for mdat box (media data)
        let mdat_found = file_data.windows(4).any(|w| w == b"mdat");
        if mdat_found {
            println!("   ✅ Has mdat box (media data)");
        }

        // Check for video track (avc1 = H.264)
        let h264_found = file_data.windows(4).any(|w| w == b"avc1");
        if h264_found {
            println!("   ✅ Has H.264 video track");
        }

        // Check for audio track
        let aac_found = file_data.windows(4).any(|w| w == b"mp4a");
        let opus_found = file_data.windows(4).any(|w| w == b"Opus");
        if aac_found || opus_found {
            println!(
                "   ✅ Has audio track ({})",
                if aac_found { "AAC" } else { "Opus" }
            );
        } else if use_audio {
            println!("   ⚠️  Expected audio track but none found");
        }

        // Step 9: Cleanup
        println!();
        println!("🗑️  Step 9: Cleanup");
        println!("------------------");
        stop_camera_preview(device_id.clone()).await?;
        release_camera(device_id).await?;
        println!("   ✅ Camera released");

        println!();
        println!("🎉 Live A/V Recording Test Complete!");
        println!();
        println!("   📁 Output file: {:?}", output_path.canonicalize()?);
        println!("   ▶️  Play it: vlc {:?}", output_path);
        println!();
    }

    Ok(())
}
