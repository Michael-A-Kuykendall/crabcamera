//! Live Audio Test with OBSBOT Camera Microphone
//!
//! This example tests the full audio pipeline:
//! 1. Enumerate audio devices (find OBSBOT mic)
//! 2. Capture PCM audio samples
//! 3. Encode to Opus
//! 4. Verify output is valid
//!
//! Run with: cargo run --example live_audio_test --features "recording,audio"

use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎤 CrabCamera Live Audio Test");
    println!("==============================");
    println!();

    // Test 1: Audio Device Enumeration
    println!("📋 Test 1: Audio Device Enumeration");
    println!("-----------------------------------");

    #[cfg(feature = "audio")]
    {
        use crabcamera::audio::{get_default_audio_device, list_audio_devices};

        let devices = list_audio_devices()?;
        println!("   ✅ Found {} audio input device(s):", devices.len());

        for (i, dev) in devices.iter().enumerate() {
            let default_marker = if dev.is_default { " [DEFAULT]" } else { "" };
            println!("      {}. {}{}", i + 1, dev.name, default_marker);
            println!(
                "         Sample Rate: {} Hz, Channels: {}",
                dev.sample_rate, dev.channels
            );
            println!("         ID: {}", dev.id);
        }

        // Look for OBSBOT
        let obsbot = devices
            .iter()
            .find(|d| d.name.to_lowercase().contains("obsbot"));
        if let Some(obs) = obsbot {
            println!();
            println!("   🎯 Found OBSBOT microphone: {}", obs.name);
        }

        // Get default device
        println!();
        let default = get_default_audio_device()?;
        println!("   📌 Default device: {}", default.name);
        println!(
            "      {} Hz, {} channel(s)",
            default.sample_rate, default.channels
        );
    }

    #[cfg(not(feature = "audio"))]
    {
        println!("   ⚠️  Audio feature not enabled. Run with --features \"audio\"");
        return Ok(());
    }

    // Test 2: Audio Capture with Opus Encoding
    println!();
    println!("🎙️  Test 2: Live Audio Capture + Opus Encoding (3 seconds)");
    println!("----------------------------------------------------------");

    #[cfg(feature = "audio")]
    {
        use crabcamera::audio::{AudioCapture, OpusEncoder, PTSClock};

        // Create a shared clock for timestamping
        let clock = PTSClock::new();

        // Create capture: use default device, 48kHz (Opus requirement), stereo
        let mut capture = AudioCapture::new(None, 48000, 2, clock.clone())?;

        println!("   📊 Capture config:");
        println!("      Sample Rate: {} Hz", capture.sample_rate());
        println!("      Channels: {}", capture.channels());

        // Create Opus encoder: 48kHz, stereo, 128kbps
        let mut encoder = OpusEncoder::new(48000, 2, 128_000)?;
        println!("   📊 Encoder: Opus @ 128 kbps");

        println!("   ▶️  Starting capture...");
        capture.start()?;

        // Clock starts on construction, no explicit start needed

        let start = Instant::now();
        let duration = Duration::from_secs(3);
        let mut total_pcm_frames = 0usize;
        let mut total_pcm_samples = 0usize;
        let mut total_opus_packets = 0usize;
        let mut total_opus_bytes = 0usize;
        let mut min_level: f32 = 1.0;
        let mut max_level: f32 = 0.0;
        let mut first_opus_toc: Option<u8> = None;

        println!("   🎤 Capturing audio (speak into your mic!)...");

        while start.elapsed() < duration {
            // Try to read an audio frame
            if let Some(frame) = capture.try_read() {
                total_pcm_frames += 1;
                total_pcm_samples += frame.samples.len();

                // Calculate audio level
                let level: f32 = frame
                    .samples
                    .iter()
                    .map(|s| s.abs())
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .unwrap_or(0.0);

                min_level = min_level.min(level);
                max_level = max_level.max(level);

                // Encode to Opus
                match encoder.encode(&frame) {
                    Ok(packets) => {
                        for packet in packets {
                            total_opus_packets += 1;
                            total_opus_bytes += packet.data.len();

                            // Save first TOC for analysis
                            if first_opus_toc.is_none() && !packet.data.is_empty() {
                                first_opus_toc = Some(packet.data[0]);
                            }
                        }
                    }
                    Err(e) => {
                        println!("   ⚠️  Encode error: {}", e);
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        capture.stop()?;

        println!("   ⏹️  Capture stopped");
        println!();
        println!("   📈 PCM Results:");
        println!("      Total frames: {}", total_pcm_frames);
        println!("      Total samples: {}", total_pcm_samples);
        println!(
            "      Duration: {:.2}s of audio",
            total_pcm_samples as f64 / 48000.0 / 2.0
        ); // stereo
        println!("      Min level: {:.4}", min_level);
        println!("      Max level: {:.4}", max_level);

        if max_level > 0.01 {
            println!("   ✅ Audio signal detected!");
        } else {
            println!("   ⚠️  Very low/no audio signal - is the mic muted?");
        }

        println!();
        println!("   📈 Opus Results:");
        println!("      Opus packets: {}", total_opus_packets);
        println!("      Total size: {} bytes", total_opus_bytes);

        if total_opus_packets > 0 {
            let avg_packet_size = total_opus_bytes / total_opus_packets;
            let bitrate_actual = (total_opus_bytes * 8) as f64 / 3.0 / 1000.0;
            println!("      Avg packet size: {} bytes", avg_packet_size);
            println!("      Actual bitrate: {:.1} kbps", bitrate_actual);

            // Verify Opus TOC byte
            if let Some(toc) = first_opus_toc {
                let config = (toc >> 3) & 0x1F;
                let stereo = (toc >> 2) & 0x01;
                let frame_count_code = toc & 0x03;

                println!();
                println!("   🔍 Opus TOC analysis (first packet):");
                println!("      TOC byte: 0x{:02X}", toc);
                println!("      Config: {} (bandwidth/framesize)", config);
                println!("      Stereo: {}", if stereo == 1 { "yes" } else { "no" });
                println!("      Frame count code: {}", frame_count_code);
            }

            println!();
            println!("   ✅ Opus encoding working!");
        } else {
            println!("   ❌ No Opus packets produced");
        }
    }

    println!();
    println!("🎉 Live Audio Test Complete!");
    println!();

    Ok(())
}
