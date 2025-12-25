//! WebRTC Performance Benchmarks for CrabCamera
//!
//! Run with: cargo bench --features "webrtc,audio" --bench webrtc_benchmarks
//!
//! These benchmarks measure WebRTC-specific performance characteristics
//! to establish excellence baselines and detect regressions.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

/// Generate test H.264 encoded data (simplified)
fn generate_test_h264_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    // Fill with some pattern to simulate H.264 NAL units
    for i in 0..size {
        data[i] = (i % 256) as u8;
    }
    // Set NAL unit start codes
    if size >= 4 {
        data[0..4].copy_from_slice(&[0x00, 0x00, 0x00, 0x01]);
    }
    data
}

/// Generate test Opus encoded data
fn generate_test_opus_data(size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    for i in 0..size {
        data[i] = (i % 256) as u8;
    }
    data
}

/// Generate a test frame with random-ish RGB data
fn generate_test_rgb(width: u32, height: u32) -> Vec<u8> {
    let size = (width * height * 3) as usize;
    let mut data = vec![0u8; size];

    // Fill with a gradient pattern (more realistic than zeros)
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 3) as usize;
            data[idx] = (x % 256) as u8; // R
            data[idx + 1] = (y % 256) as u8; // G
            data[idx + 2] = ((x + y) % 256) as u8; // B
        }
    }

    data
}

fn bench_rtp_packetization(c: &mut Criterion) {
    let mut group = c.benchmark_group("WebRTC RTP Packetization");
    group.measurement_time(Duration::from_secs(5));

    // Test different payload sizes
    let payload_sizes = [
        (1024, "1KB"),     // Small packet
        (8192, "8KB"),     // Medium packet
        (32768, "32KB"),   // Large packet (typical max)
    ];

    for (size, name) in payload_sizes {
        let h264_data = generate_test_h264_data(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("h264_packetize", name),
            &h264_data,
            |b, data| {
                // Create packetizer and measure packetization
                let mut packetizer = crabcamera::webrtc::streaming::H264RTPPacketizer::new(1200);
                b.iter(|| {
                    let _packets = packetizer.packetize(black_box(data), black_box(0));
                });
            },
        );
    }

    group.finish();
}

fn bench_opus_rtp_packetization(c: &mut Criterion) {
    let mut group = c.benchmark_group("WebRTC Opus RTP Packetization");
    group.measurement_time(Duration::from_secs(5));

    // Test different payload sizes
    let payload_sizes = [
        (960, "20ms-48kHz"),   // 20ms at 48kHz mono
        (1920, "40ms-48kHz"),  // 40ms at 48kHz mono
    ];

    for (size, name) in payload_sizes {
        let opus_data = generate_test_opus_data(size);

        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::new("opus_packetize", name),
            &opus_data,
            |b, data| {
                // Create packetizer and measure packetization
                let mut packetizer = crabcamera::webrtc::streaming::OpusRTPPacketizer::new();
                b.iter(|| {
                    let _packets = packetizer.packetize(black_box(data), black_box(0));
                });
            },
        );
    }

    group.finish();
}

fn bench_webrtc_config_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("WebRTC Configuration");

    group.bench_function("rtc_config_default", |b| {
        b.iter(|| {
            let config = crabcamera::webrtc::RTCConfiguration::default();
            black_box(config);
        });
    });

    group.bench_function("stream_config_default", |b| {
        b.iter(|| {
            let config = crabcamera::webrtc::StreamConfig::default();
            black_box(config);
        });
    });

    group.bench_function("stream_config_builder", |b| {
        b.iter(|| {
            let config = crabcamera::webrtc::StreamConfig {
                bitrate: black_box(4_000_000),
                max_fps: black_box(60),
                width: black_box(1920),
                height: black_box(1080),
                codec: crabcamera::webrtc::streaming::VideoCodec::H264,
            };
            black_box(config);
        });
    });

    group.finish();
}

fn bench_sdp_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("WebRTC SDP Parsing");
    group.measurement_time(Duration::from_secs(5));

    // Sample SDP offer (simplified)
    let sdp_offer = "v=0\r\no=- 123456789 123456789 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\na=group:BUNDLE 0\r\nm=video 9 UDP/TLS/RTP/SAVPF 96\r\nc=IN IP4 0.0.0.0\r\na=rtpmap:96 H264/90000\r\na=fmtp:96 profile-level-id=42e01f;packetization-mode=1\r\na=sendrecv\r\n";

    group.bench_function("parse_sdp_offer", |b| {
        b.iter(|| {
            // This would parse SDP - simplified test
            let lines: Vec<&str> = sdp_offer.split("\r\n").collect();
            black_box(lines.len());
        });
    });

    group.finish();
}

criterion_group!(
    webrtc_benches,
    bench_rtp_packetization,
    bench_opus_rtp_packetization,
    bench_webrtc_config_operations,
    bench_sdp_parsing,
);

criterion_main!(webrtc_benches);