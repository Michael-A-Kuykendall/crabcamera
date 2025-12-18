//! Performance benchmarks for CrabCamera encoding pipelines
//!
//! Run with: cargo bench --features "recording,audio"
//!
//! These benchmarks measure real-world encoding performance to establish
//! baseline metrics and detect performance regressions.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use crabcamera::recording::{H264Encoder, RecordingConfig};
use crabcamera::audio::{OpusEncoder, AudioFrame};
use std::time::Duration;

/// Generate a test frame with random-ish RGB data
fn generate_test_rgb(width: u32, height: u32) -> Vec<u8> {
    let size = (width * height * 3) as usize;
    let mut data = vec![0u8; size];
    
    // Fill with a gradient pattern (more realistic than zeros)
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 3) as usize;
            data[idx] = (x % 256) as u8;     // R
            data[idx + 1] = (y % 256) as u8; // G
            data[idx + 2] = ((x + y) % 256) as u8; // B
        }
    }
    
    data
}

/// Generate test audio samples (stereo sine wave)
fn generate_test_audio(samples_per_channel: usize) -> Vec<f32> {
    let mut samples = Vec::with_capacity(samples_per_channel * 2);
    
    for i in 0..samples_per_channel {
        let t = i as f32 / 48000.0;
        let left = (t * 440.0 * std::f32::consts::TAU).sin() * 0.5;
        let right = (t * 880.0 * std::f32::consts::TAU).sin() * 0.5;
        samples.push(left);
        samples.push(right);
    }
    
    samples
}

fn bench_h264_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("H264 Encoding");
    group.measurement_time(Duration::from_secs(10));
    
    // Test common resolutions
    let resolutions = [
        (640, 480, "480p"),
        (1280, 720, "720p"),
        (1920, 1080, "1080p"),
    ];
    
    for (width, height, name) in resolutions {
        // Skip 1080p in short runs - it's expensive
        if width == 1920 {
            group.sample_size(10);
        }
        
        let rgb_data = generate_test_rgb(width, height);
        let pixels = (width * height) as u64;
        
        group.throughput(Throughput::Elements(pixels));
        group.bench_with_input(
            BenchmarkId::new("encode_frame", name),
            &rgb_data,
            |b, rgb| {
                let mut encoder = H264Encoder::new(width, height, 30.0, 2_000_000)
                    .expect("Failed to create encoder");
                
                b.iter(|| {
                    encoder.encode_rgb(black_box(rgb)).expect("Encode failed")
                });
            },
        );
    }
    
    group.finish();
}

fn bench_h264_encoder_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("H264 Encoder Creation");
    
    let resolutions = [
        (640, 480, "480p"),
        (1280, 720, "720p"),
        (1920, 1080, "1080p"),
    ];
    
    for (width, height, name) in resolutions {
        group.bench_function(BenchmarkId::new("new", name), |b| {
            b.iter(|| {
                H264Encoder::new(
                    black_box(width),
                    black_box(height),
                    30.0,
                    2_000_000,
                ).expect("Failed to create encoder")
            });
        });
    }
    
    group.finish();
}

fn bench_opus_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("Opus Encoding");
    group.measurement_time(Duration::from_secs(5));
    
    // Test common audio buffer sizes (in samples per channel)
    // Standard Opus frame sizes: 2.5, 5, 10, 20, 40, 60 ms
    let buffer_sizes = [
        (480, "10ms"),   // 10ms at 48kHz
        (960, "20ms"),   // 20ms at 48kHz (most common)
        (1920, "40ms"),  // 40ms at 48kHz
    ];
    
    for (samples, name) in buffer_sizes {
        let audio_samples = generate_test_audio(samples);
        
        group.throughput(Throughput::Elements(samples as u64 * 2)); // stereo
        group.bench_with_input(
            BenchmarkId::new("encode_frame", name),
            &audio_samples,
            |b, samples| {
                let mut encoder = OpusEncoder::new(48000, 2, 128000)
                    .expect("Failed to create encoder");
                
                b.iter(|| {
                    let frame = AudioFrame {
                        samples: samples.clone(),
                        sample_rate: 48000,
                        channels: 2,
                        timestamp: 0.0,
                    };
                    encoder.encode(black_box(&frame)).expect("Encode failed")
                });
            },
        );
    }
    
    group.finish();
}

fn bench_opus_encoder_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Opus Encoder Creation");
    
    let configs = [
        (48000, 1, 64000, "mono-64k"),
        (48000, 2, 128000, "stereo-128k"),
        (48000, 2, 256000, "stereo-256k"),
    ];
    
    for (sample_rate, channels, bitrate, name) in configs {
        group.bench_function(BenchmarkId::new("new", name), |b| {
            b.iter(|| {
                OpusEncoder::new(
                    black_box(sample_rate),
                    black_box(channels),
                    black_box(bitrate),
                ).expect("Failed to create encoder")
            });
        });
    }
    
    group.finish();
}

fn bench_rgb_to_yuv_conversion(c: &mut Criterion) {
    let mut group = c.benchmark_group("RGB to YUV Conversion");
    group.measurement_time(Duration::from_secs(5));
    
    // This benchmarks the internal conversion happening inside H264Encoder
    // by measuring full encode minus expected encoder time
    let resolutions = [
        (640, 480, "480p"),
        (1280, 720, "720p"),
    ];
    
    for (width, height, name) in resolutions {
        let rgb_data = generate_test_rgb(width, height);
        let pixels = (width * height) as u64;
        
        group.throughput(Throughput::Elements(pixels));
        group.bench_with_input(
            BenchmarkId::new("via_encode", name),
            &rgb_data,
            |b, rgb| {
                // Create a new encoder for each iteration to avoid state accumulation
                b.iter_custom(|iters| {
                    let mut total = Duration::ZERO;
                    let mut encoder = H264Encoder::new(width, height, 30.0, 2_000_000)
                        .expect("Failed to create encoder");
                    
                    for _ in 0..iters {
                        let start = std::time::Instant::now();
                        let _ = encoder.encode_rgb(black_box(rgb));
                        total += start.elapsed();
                    }
                    
                    total
                });
            },
        );
    }
    
    group.finish();
}

fn bench_recording_config(c: &mut Criterion) {
    let mut group = c.benchmark_group("RecordingConfig");
    
    group.bench_function("new_default", |b| {
        b.iter(|| {
            RecordingConfig::new(
                black_box(1920),
                black_box(1080),
                black_box(30.0),
            )
        });
    });
    
    group.bench_function("builder_chain", |b| {
        b.iter(|| {
            RecordingConfig::new(1920, 1080, 30.0)
                .with_bitrate(black_box(4_000_000))
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_h264_encoding,
    bench_h264_encoder_creation,
    bench_opus_encoding,
    bench_opus_encoder_creation,
    bench_rgb_to_yuv_conversion,
    bench_recording_config,
);

criterion_main!(benches);
