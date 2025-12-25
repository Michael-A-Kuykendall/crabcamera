//! Fuzz-style tests using proptest
//!
//! These provide fuzz-like testing without requiring nightly Rust or cargo-fuzz.
//! Run with: cargo test --test fuzz_tests --features "recording,audio"

#[cfg(feature = "audio")]
mod audio_fuzz {
    use super::*;
    use crabcamera::audio::{AudioFrame, OpusEncoder};

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(1000))]

        /// Fuzz the Opus encoder with random inputs
        /// The encoder should never panic, only return errors
        #[test]
        fn fuzz_opus_encoder_creation(
            sample_rate in 0u32..100000,
            channels in 0u16..10,
            bitrate in 0u32..10_000_000,
        ) {
            // Should not panic - may return error
            let _ = OpusEncoder::new(sample_rate, channels, bitrate);
        }

        /// Fuzz encoding with random sample data
        #[test]
        fn fuzz_opus_encode_samples(
            samples in prop::collection::vec(-2.0f32..2.0f32, 0..10000),
            timestamp in 0.0f64..100000.0,
        ) {
            // Create valid encoder
            let mut encoder = match OpusEncoder::new(48000, 2, 128000) {
                Ok(e) => e,
                Err(_) => return Ok(()),
            };

            let frame = AudioFrame {
                samples,
                sample_rate: 48000,
                channels: 2,
                timestamp,
            };

            // Should not panic
            let _ = encoder.encode(&frame);
            let _ = encoder.flush();
        }

        /// Fuzz with mismatched sample rates
        #[test]
        fn fuzz_opus_mismatched_sample_rate(
            frame_sample_rate in 0u32..100000,
            samples in prop::collection::vec(-1.0f32..1.0f32, 0..5000),
        ) {
            let mut encoder = match OpusEncoder::new(48000, 2, 128000) {
                Ok(e) => e,
                Err(_) => return Ok(()),
            };

            let frame = AudioFrame {
                samples,
                sample_rate: frame_sample_rate,
                channels: 2,
                timestamp: 0.0,
            };

            // Should return error, not panic
            let _ = encoder.encode(&frame);
        }

        /// Fuzz with mismatched channel counts
        #[test]
        fn fuzz_opus_mismatched_channels(
            frame_channels in 0u16..10,
            samples in prop::collection::vec(-1.0f32..1.0f32, 0..5000),
        ) {
            let mut encoder = match OpusEncoder::new(48000, 2, 128000) {
                Ok(e) => e,
                Err(_) => return Ok(()),
            };

            let frame = AudioFrame {
                samples,
                sample_rate: 48000,
                channels: frame_channels,
                timestamp: 0.0,
            };

            // Should return error, not panic
            let _ = encoder.encode(&frame);
        }
    }
}

#[cfg(feature = "recording")]
mod recording_fuzz {
    use super::*;
    use crabcamera::recording::{H264Encoder, RecordingConfig};

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(500))]

        /// Fuzz recording config creation
        #[test]
        fn fuzz_recording_config(
            width in 0u32..10000,
            height in 0u32..10000,
            fps in -100.0f64..1000.0,
            bitrate in 0u32..1_000_000_000,
        ) {
            // Should not panic
            let config = RecordingConfig::new(width, height, fps);
            let _ = config.with_bitrate(bitrate);
        }

        /// Fuzz H264 encoder creation
        #[test]
        fn fuzz_h264_encoder_creation(
            width in 0u32..10000,
            height in 0u32..10000,
            fps in -100.0f64..1000.0,
            bitrate in 0u32..100_000_000,
        ) {
            // Should not panic - may return error
            let _ = H264Encoder::new(width, height, fps, bitrate);
        }

        /// Fuzz H264 encoder with random RGB data
        #[test]
        fn fuzz_h264_encode_rgb(
            // Use dimensions that are multiples of 16 (H264 requirement)
            width_mult in 1u32..30,
            height_mult in 1u32..30,
            rgb_values in prop::collection::vec(0u8..255, 100..50000),
        ) {
            let width = width_mult * 16;
            let height = height_mult * 16;
            let expected_size = (width * height * 3) as usize;

            // Create valid encoder
            let mut encoder = match H264Encoder::new(width, height, 30.0, 1_000_000) {
                Ok(e) => e,
                Err(_) => return Ok(()),
            };

            // Pad or truncate RGB data to match expected size
            let rgb: Vec<u8> = if rgb_values.len() >= expected_size {
                rgb_values[..expected_size].to_vec()
            } else {
                let mut padded = rgb_values;
                padded.resize(expected_size, 128);
                padded
            };

            // Should not panic
            let _ = encoder.encode_rgb(&rgb);
        }
    }
}

#[cfg(feature = "recording")]
mod muxer_fuzz {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Fuzz muxer with random video data
        #[test]
        fn fuzz_muxer_write_video(
            pts in 0.0f64..100000.0,
            data in prop::collection::vec(0u8..255, 0..10000),
            is_keyframe in proptest::bool::ANY,
        ) {
            use muxide::api::{Muxer, MuxerConfig};

            let dir = match tempdir() {
                Ok(d) => d,
                Err(_) => return Ok(()),
            };
            let path = dir.path().join("fuzz_test.mp4");

            let file = match File::create(&path) {
                Ok(f) => f,
                Err(_) => return Ok(()),
            };

            let config = MuxerConfig::new(640, 480, 30.0);
            let mut muxer = match Muxer::new(file, config) {
                Ok(m) => m,
                Err(_) => return Ok(()),
            };

            // Should not panic - may return error for invalid data
            let _ = muxer.write_video(pts, &data, is_keyframe);
        }
    }
}
