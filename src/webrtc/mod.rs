/// WebRTC live preview streaming module
/// 
/// Provides real-time streaming capabilities for camera previews
/// using WebRTC technology for low-latency browser integration.

pub mod streaming;
pub mod peer;

pub use streaming::{WebRTCStreamer, StreamConfig};
pub use peer::{PeerConnection, RTCConfiguration, SessionDescription, IceCandidate};