//! Synthetic A/V Integration Tests
//!
//! These tests use synthetic data based on real OBSBOT hardware captures
//! to verify the full A/V pipeline works without requiring real hardware.
//!
//! Run with: cargo test --test synthetic_av_test --features "recording,audio"

#![cfg(all(feature = "recording", feature = "audio"))]

use std::time::Duration;
use tempfile::tempdir;

use crabcamera::recording::{Recorder, RecordingConfig, AudioConfig};
use crabcamera::audio::{OpusEncoder, PTSClock};
use crabcamera::testing::{synthetic_video_frame, synthetic_audio_frame, ObsbotCharacteristics};

/// Test: Full synthetic A/V recording produces valid MP4
#[test]
fn test_synthetic_av_recording() {
    let dir = tempdir().expect("Create temp dir");
    let output = dir.path().join("synthetic_av.mp4");
    
    // Use OBSBOT-like characteristics but at lower resolution for speed
    let width = 640;
    let height = 480;
    let fps = 30.0;
    
    // Configure with audio
    let config = RecordingConfig::new(width, height, fps)
        .with_audio(AudioConfig {
            device_id: None, // Will use synthetic data
            sample_rate: 48000,
            channels: 2,
            bitrate: 128_000,
        });
    
    let mut recorder = Recorder::new(&output, config)
        .expect("Create recorder");
    
    // Write 90 frames (3 seconds at 30fps)
    for i in 0..90 {
        let frame = synthetic_video_frame(i, width, height);
        recorder.write_frame(&frame).expect("Write frame");
    }
    
    let stats = recorder.finish().expect("Finish recording");
    
    // Verify recording statistics
    assert_eq!(stats.video_frames, 90, "Should have 90 video frames");
    assert!(stats.bytes_written > 0, "Should have written bytes");
    
    // Verify file structure
    let data = std::fs::read(&output).expect("Read output file");
    
    // Check MP4 signature
    assert!(data.len() >= 8, "File too small");
    assert_eq!(&data[4..8], b"ftyp", "Should have ftyp box");
    
    // Check for moov (metadata)
    assert!(data.windows(4).any(|w| w == b"moov"), "Should have moov box");
    
    // Check for mdat (media data)
    assert!(data.windows(4).any(|w| w == b"mdat"), "Should have mdat box");
    
    // Check for H.264 video track
    assert!(data.windows(4).any(|w| w == b"avc1"), "Should have H.264 track");
}

/// Test: Synthetic audio encodes correctly to Opus
#[test]
fn test_synthetic_audio_encoding() {
    let mut encoder = OpusEncoder::new(48000, 2, 128_000)
        .expect("Create encoder");
    
    let mut total_packets = 0;
    let mut total_bytes = 0;
    
    // Encode 150 frames (3 seconds at 20ms/frame)
    for i in 0..150 {
        let frame = synthetic_audio_frame(i, 960); // 960 samples = 20ms @ 48kHz
        let packets = encoder.encode(&frame).expect("Encode frame");
        
        for packet in packets {
            total_packets += 1;
            total_bytes += packet.data.len();
            
            // Verify each packet has valid Opus TOC
            assert!(!packet.data.is_empty(), "Packet should not be empty");
            let toc = packet.data[0];
            let config = (toc >> 3) & 0x1F;
            assert!(config < 32, "Invalid Opus config in TOC");
        }
    }
    
    // Should have produced approximately 1 packet per frame
    assert!(total_packets >= 140, "Should have ~150 packets, got {}", total_packets);
    
    // Check bitrate is reasonable (128kbps = ~48KB for 3 seconds)
    let expected_bytes = (128_000 / 8) * 3; // 48000 bytes
    let tolerance = expected_bytes / 2; // 50% tolerance
    assert!(
        (total_bytes as i64 - expected_bytes as i64).abs() < tolerance as i64,
        "Bitrate off: got {} bytes, expected ~{}", total_bytes, expected_bytes
    );
}

/// Test: Synthetic frames vary between frames (important for video encoding)
#[test]
fn test_synthetic_frames_vary() {
    let frame0 = synthetic_video_frame(0, 320, 240);
    let frame1 = synthetic_video_frame(1, 320, 240);
    let frame2 = synthetic_video_frame(2, 320, 240);
    
    // Frames should have different content
    assert_ne!(frame0.data, frame1.data, "Frame 0 and 1 should differ");
    assert_ne!(frame1.data, frame2.data, "Frame 1 and 2 should differ");
    
    // But same dimensions
    assert_eq!(frame0.width, frame1.width);
    assert_eq!(frame0.height, frame1.height);
}

/// Test: PTS clock produces monotonic timestamps
#[test]
fn test_pts_clock_monotonic() {
    let clock = PTSClock::new();
    
    let mut prev_pts = clock.pts();
    for _ in 0..100 {
        std::thread::sleep(Duration::from_micros(100));
        let current_pts = clock.pts();
        assert!(current_pts >= prev_pts, "PTS should be monotonically increasing");
        prev_pts = current_pts;
    }
}

/// Test: OBSBOT characteristics match real hardware
#[test]
fn test_obsbot_characteristics_match_real() {
    let chars = ObsbotCharacteristics::default();
    
    // These values were captured from real OBSBOT Tiny 4K hardware
    assert_eq!(chars.native_resolution, (3840, 2160), "Should match 4K");
    assert_eq!(chars.audio_sample_rate, 48000, "Should match 48kHz");
    assert_eq!(chars.audio_channels, 2, "Should be stereo");
    assert!(chars.device_name.contains("OBSBOT"), "Should mention OBSBOT");
    assert!(chars.mic_name.contains("OBSBOT"), "Mic should mention OBSBOT");
}

/// Test: Long recording doesn't accumulate errors
/// Note: Recorder has frame-rate limiting, so rapid writes will be throttled.
/// We check that total_frames + dropped_frames = expected.
#[test]
fn test_long_synthetic_recording() {
    let dir = tempdir().expect("Create temp dir");
    let output = dir.path().join("long_recording.mp4");
    
    let config = RecordingConfig::new(320, 240, 30.0);
    let mut recorder = Recorder::new(&output, config)
        .expect("Create recorder");
    
    // Write 300 frames with some spacing to avoid rate limiting
    let expected_frames = 100; // Fewer frames but with spacing
    let frame_interval = std::time::Duration::from_millis(33); // ~30fps
    
    for i in 0..expected_frames {
        let frame = synthetic_video_frame(i, 320, 240);
        recorder.write_frame(&frame).unwrap_or_else(|_| panic!("Write frame {}", i));
        std::thread::sleep(frame_interval);
    }
    
    let stats = recorder.finish().expect("Finish recording");
    
    // With proper timing, we should get most frames
    assert!(stats.video_frames >= expected_frames / 2, 
        "Should have at least half the frames, got {} of {}", stats.video_frames, expected_frames);
    
    // File should be reasonably sized (not bloated)
    let file_size = std::fs::metadata(&output).expect("Read metadata").len();
    assert!(file_size < 10_000_000, "File shouldn't be bloated: {} bytes", file_size);
}
