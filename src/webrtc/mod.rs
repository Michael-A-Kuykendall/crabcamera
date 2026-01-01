//! # WebRTC Module
//!
//! **Library Selection: webrtc-rs (Apache-2.0)**
//!
//! This module implements production WebRTC streaming using the pure-Rust
//! `webrtc` crate (webrtc-rs/webrtc). Most mock implementations have been
//! replaced with real WebRTC protocol handling, but full camera integration
//! is still in progress.
//!
//! ## Architecture
//! - Peer connections via `webrtc::peer_connection::RTCPeerConnection`
//! - SDP handling via `webrtc::sdp`
//! - ICE via `webrtc::ice_transport`
//! - RTP/SRTP via `webrtc::rtp_transceiver`
//! - Data channels via `webrtc::data_channel`
//!
//! ## Integration
//! - Video encoding: openh264 → RTP packetization
//! - Audio encoding: libopus_sys → RTP packetization
//! - Camera capture: nokhwa frames → encoding pipeline (TODO)
//! - Audio capture: cpal samples → encoding pipeline (TODO)

#[cfg(feature = "webrtc")]
pub mod peer;
#[cfg(feature = "webrtc")]
pub mod streaming;

#[cfg(feature = "webrtc")]
pub use peer::{IceCandidate, PeerConnection, RTCConfiguration, SessionDescription};
#[cfg(feature = "webrtc")]
pub use streaming::{StreamConfig, WebRTCStreamer, SimulcastConfig, SimulcastLayer, StreamMode, CameraStatus};
