//! Contract-level tests for WebRTC streaming.
//!
//! These are not browser-level E2E tests; they validate that the streamer
//! produces real encoded data and RTP payload group semantics under the
//! supported test mode.

#![cfg(feature = "webrtc")]

use std::time::Duration;

use crabcamera::webrtc::streaming::{StreamConfig, StreamMode, VideoCodec, WebRTCStreamer};
use tokio::time::timeout;

#[tokio::test]
async fn synthetic_stream_produces_encoded_frames_and_rtp_groups() {
    let config = StreamConfig {
        bitrate: 500_000,
        max_fps: 5,
        width: 320,
        height: 240,
        codec: VideoCodec::H264,
        simulcast: None,
    };

    let streamer = WebRTCStreamer::new("contract_stream".to_string(), config);
    streamer.set_mode(StreamMode::SyntheticTest).await;
    streamer.init_h264_packetizer(900).await;

    let mut frame_rx = streamer.subscribe_frames();

    let (rtp_tx, mut rtp_rx) = tokio::sync::mpsc::unbounded_channel();
    streamer.set_rtp_sender(rtp_tx).await;

    streamer
        .start_streaming("synthetic_device".to_string())
        .await
        .unwrap();

    // 1) Encoded frame contract: non-empty + looks like Annex B.
    let frame = timeout(Duration::from_secs(2), frame_rx.recv())
        .await
        .expect("timed out waiting for frame")
        .expect("frame receiver error");

    assert!(
        !frame.data.is_empty(),
        "encoded frame data should not be empty"
    );
    assert!(
        frame.data.starts_with(&[0, 0, 0, 1]) || frame.data.starts_with(&[0, 0, 1]),
        "encoded frame should look like Annex B (start code)"
    );

    // 2) RTP grouping contract: each access unit yields a contiguous run of RTP payloads
    // with constant timestamp, ending with exactly one marker packet.
    let mut marker_frames_seen = 0usize;
    let mut current_ts: Option<u64> = None;

    let res = timeout(Duration::from_secs(3), async {
        while marker_frames_seen < 2 {
            let pkt = rtp_rx.recv().await.expect("rtp channel closed");
            assert!(!pkt.data.is_empty(), "RTP payload must not be empty");

            match current_ts {
                Some(ts) => assert_eq!(
                    pkt.timestamp, ts,
                    "timestamp must be constant within a frame"
                ),
                None => current_ts = Some(pkt.timestamp),
            }

            // packetizer budgets RTP header externally; payload must fit within mtu-12
            assert!(
                pkt.data.len() <= 900 - 12,
                "payload must respect MTU budget"
            );

            if pkt.marker {
                marker_frames_seen += 1;
                current_ts = None;
            }
        }
    })
    .await;

    assert!(res.is_ok(), "timed out waiting for RTP marker frames");

    streamer.stop_streaming().await.unwrap();
}

#[tokio::test]
#[ignore = "Requires a real camera; run manually with: cargo test --test webrtc_stream_contract_test --features webrtc -- --ignored"]
async fn real_camera_stream_produces_frames_when_available() {
    // This is a manual, hardware-dependent contract test.
    let cameras = crabcamera::CameraSystem::list_cameras().unwrap_or_default();
    if cameras.is_empty() {
        return;
    }

    let device_id = cameras[0].id.clone();

    let config = StreamConfig {
        bitrate: 1_000_000,
        max_fps: 10,
        width: 640,
        height: 480,
        codec: VideoCodec::H264,
        simulcast: None,
    };

    let streamer = WebRTCStreamer::new("real_camera_contract".to_string(), config);
    streamer.set_mode(StreamMode::RealCamera).await;

    let mut frame_rx = streamer.subscribe_frames();

    // If the camera can't start, we want a clear failure signal.
    streamer
        .start_streaming(device_id)
        .await
        .expect("start_streaming should succeed");

    let frame = timeout(Duration::from_secs(5), frame_rx.recv())
        .await
        .expect("timed out waiting for a real-camera frame")
        .expect("frame receiver error");

    assert!(!frame.data.is_empty(), "encoded frame should not be empty");

    streamer.stop_streaming().await.unwrap();
}
