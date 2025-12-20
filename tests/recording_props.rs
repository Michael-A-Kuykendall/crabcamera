//! Property-Based Tests for CrabCamera Recording Module
//!
//! These tests verify invariants and contracts of the recording subsystem
//! using proptest for input generation and shrinking.
//!
//! Run with: cargo test --test recording_props --features recording

use proptest::prelude::*;
use tempfile::tempdir;

#[cfg(feature = "recording")]
mod recording_tests {
    use super::*;
    use crabcamera::recording::{H264Encoder, Recorder, RecordingConfig};

    // ═══════════════════════════════════════════════════════════════════════════
    // H264 ENCODER INVARIANTS
    // ═══════════════════════════════════════════════════════════════════════════

    proptest! {
        /// INVARIANT: Encoder accepts valid dimension ranges
        /// Dimensions must be multiples of 16 for h264, so we test 16-aligned values
        #[test]
        fn encoder_accepts_valid_dimensions(
            width in (1u32..120).prop_map(|w| w * 16),   // 16 to 1920
            height in (1u32..68).prop_map(|h| h * 16),   // 16 to 1080
            fps in 15.0f64..60.0,
            bitrate in 500_000u32..10_000_000,
        ) {
            let result = H264Encoder::new(width, height, fps, bitrate);
            prop_assert!(result.is_ok(), "Encoder should accept {}x{} @ {}fps: {:?}",
                width, height, fps, result.err());
        }

        /// INVARIANT: Encoded frames are valid Annex B format
        /// Every h264 frame must start with NAL unit prefix [0,0,0,1] or [0,0,1]
        #[test]
        fn encoded_frames_are_annex_b(
            gray_level in 0u8..255,
        ) {
            let width = 320u32;
            let height = 240u32;

            let mut encoder = H264Encoder::new(width, height, 30.0, 1_000_000)
                .expect("Encoder creation should succeed");

            // Create a uniform gray frame
            let rgb = vec![gray_level; (width * height * 3) as usize];

            let encoded = encoder.encode_rgb(&rgb)
                .expect("Encoding should succeed");

            if !encoded.data.is_empty() {
                // Check for Annex B start codes
                let starts_with_4byte = encoded.data.starts_with(&[0, 0, 0, 1]);
                let starts_with_3byte = encoded.data.starts_with(&[0, 0, 1]);

                prop_assert!(
                    starts_with_4byte || starts_with_3byte,
                    "Encoded frame should start with Annex B prefix, got: {:02x?}",
                    &encoded.data[..encoded.data.len().min(10)]
                );
            }
        }

        /// INVARIANT: First encoded frame is always a keyframe
        #[test]
        fn first_frame_is_keyframe(
            r in 0u8..255,
            g in 0u8..255,
            b in 0u8..255,
        ) {
            let width = 320u32;
            let height = 240u32;

            let mut encoder = H264Encoder::new(width, height, 30.0, 1_000_000)
                .expect("Encoder creation should succeed");

            // Create a colored frame
            let mut rgb = Vec::with_capacity((width * height * 3) as usize);
            for _ in 0..(width * height) {
                rgb.push(r);
                rgb.push(g);
                rgb.push(b);
            }

            let encoded = encoder.encode_rgb(&rgb)
                .expect("Encoding should succeed");

            prop_assert!(encoded.is_keyframe, "First frame must be a keyframe");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RECORDER INVARIANTS
    // ═══════════════════════════════════════════════════════════════════════════

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        /// INVARIANT: Recorder frame count matches write count
        #[test]
        fn recorder_frame_count_matches(
            frame_count in 1usize..20,
        ) {
            let dir = tempdir().expect("tempdir");
            let output = dir.path().join("test_output.mp4");

            let width = 320u32;
            let height = 240u32;

            let config = RecordingConfig::new(width, height, 30.0);
            let mut recorder = Recorder::new(&output, config)
                .expect("Recorder creation should succeed");

            // Write frames
            for i in 0..frame_count {
                let gray = ((i * 17) % 256) as u8;
                let rgb = vec![gray; (width * height * 3) as usize];
                recorder.write_rgb_frame(&rgb, width, height)
                    .expect("Frame write should succeed");
            }

            let stats = recorder.finish()
                .expect("Finish should succeed");

            prop_assert_eq!(
                stats.video_frames as usize,
                frame_count,
                "Frame count mismatch: expected {}, got {}",
                frame_count, stats.video_frames
            );
        }

        /// INVARIANT: Output file size is bounded
        /// The output should never be larger than raw uncompressed data
        #[test]
        fn output_size_bounded(
            frame_count in 1usize..10,
        ) {
            let dir = tempdir().expect("tempdir");
            let output = dir.path().join("test_bounded.mp4");

            let width = 320u32;
            let height = 240u32;

            let config = RecordingConfig::new(width, height, 30.0);
            let mut recorder = Recorder::new(&output, config)
                .expect("Recorder creation should succeed");

            let raw_frame_size = (width * height * 3) as usize;

            for i in 0..frame_count {
                let gray = ((i * 31) % 256) as u8;
                let rgb = vec![gray; raw_frame_size];
                recorder.write_rgb_frame(&rgb, width, height)
                    .expect("Frame write should succeed");
            }

            let stats = recorder.finish()
                .expect("Finish should succeed");

            let max_reasonable_size = (raw_frame_size * frame_count) as u64;

            prop_assert!(
                stats.bytes_written < max_reasonable_size,
                "Compressed output ({} bytes) should be smaller than raw ({} bytes)",
                stats.bytes_written, max_reasonable_size
            );
        }

        /// INVARIANT: Bytes written is always positive after finish
        #[test]
        fn bytes_written_positive(
            frame_count in 1usize..5,
        ) {
            let dir = tempdir().expect("tempdir");
            let output = dir.path().join("test_positive.mp4");

            let config = RecordingConfig::new(320, 240, 30.0);
            let mut recorder = Recorder::new(&output, config)
                .expect("Recorder creation should succeed");

            for i in 0..frame_count {
                let gray = ((i * 41) % 256) as u8;
                let rgb = vec![gray; 320 * 240 * 3];
                recorder.write_rgb_frame(&rgb, 320, 240)
                    .expect("Frame write should succeed");
            }

            let stats = recorder.finish()
                .expect("Finish should succeed");

            prop_assert!(stats.bytes_written > 0, "Bytes written must be positive");
        }

        /// INVARIANT: Duration is proportional to frame count
        #[test]
        fn duration_proportional_to_frames(
            frame_count in 10usize..50,
            fps in prop::sample::select(vec![15.0f64, 30.0, 60.0]),
        ) {
            let dir = tempdir().expect("tempdir");
            let output = dir.path().join("test_duration.mp4");

            let config = RecordingConfig::new(320, 240, fps);
            let mut recorder = Recorder::new(&output, config)
                .expect("Recorder creation should succeed");

            for i in 0..frame_count {
                let gray = ((i * 47) % 256) as u8;
                let rgb = vec![gray; 320 * 240 * 3];
                recorder.write_rgb_frame(&rgb, 320, 240)
                    .expect("Frame write should succeed");
            }

            let stats = recorder.finish()
                .expect("Finish should succeed");

            let expected_duration = frame_count as f64 / fps;
            let tolerance = 0.1; // 100ms tolerance

            prop_assert!(
                (stats.duration_secs - expected_duration).abs() < tolerance,
                "Duration mismatch: expected ~{:.2}s, got {:.2}s (frames={}, fps={})",
                expected_duration, stats.duration_secs, frame_count, fps
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RECORDING CONFIG INVARIANTS
    // ═══════════════════════════════════════════════════════════════════════════

    proptest! {
        /// INVARIANT: Config preserves all set values
        #[test]
        fn config_preserves_values(
            width in 160u32..4096,
            height in 120u32..2160,
            fps in 1.0f64..120.0,
            title in "[a-zA-Z0-9 ]{0,50}",
        ) {
            let config = RecordingConfig::new(width, height, fps)
                .with_title(&title);

            prop_assert_eq!(config.width, width);
            prop_assert_eq!(config.height, height);
            prop_assert!((config.fps - fps).abs() < 0.001);
            prop_assert_eq!(config.title.as_deref(), Some(title.as_str()));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FORMAT VALIDATION INVARIANTS (no recording feature required)
// ═══════════════════════════════════════════════════════════════════════════════

mod format_tests {
    use super::*;
    use crabcamera::types::CameraFormat;

    proptest! {
        /// INVARIANT: CameraFormat preserves all set values
        #[test]
        fn camera_format_preserves(
            width in 1u32..8192,
            height in 1u32..4320,
            fps in 0.1f32..240.0,
        ) {
            let format = CameraFormat::new(width, height, fps);

            prop_assert_eq!(format.width, width);
            prop_assert_eq!(format.height, height);
            prop_assert!((format.fps - fps).abs() < 0.001);
        }

        /// INVARIANT: Format with zero dimensions fails gracefully
        /// (This tests expected behavior - formats with 0 should be rejected somewhere)
        #[test]
        fn zero_dimensions_handled(
            zero_width in prop::bool::ANY,
            zero_height in prop::bool::ANY,
        ) {
            let width = if zero_width { 0 } else { 640 };
            let height = if zero_height { 0 } else { 480 };

            // Currently CameraFormat doesn't validate - this documents current behavior
            let format = CameraFormat::new(width, height, 30.0);

            // If either is zero, the format has zero pixels
            if zero_width || zero_height {
                prop_assert!(
                    format.width == 0 || format.height == 0,
                    "Zero dimension should be preserved (current behavior)"
                );
            }
        }
    }
}
