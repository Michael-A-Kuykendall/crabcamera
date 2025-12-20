pub mod peer;
/// WebRTC live preview streaming module
///
/// Provides real-time streaming capabilities for camera previews
/// using WebRTC technology for low-latency browser integration.
pub mod streaming;

pub use peer::{IceCandidate, PeerConnection, RTCConfiguration, SessionDescription};
pub use streaming::{StreamConfig, WebRTCStreamer};
