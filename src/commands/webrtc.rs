use crate::webrtc::peer::{ConnectionState, PeerConnectionStats};
use crate::webrtc::streaming::StreamStats;
use crate::webrtc::{
    IceCandidate, PeerConnection, RTCConfiguration, SessionDescription, StreamConfig,
    WebRTCStreamer,
};
use std::collections::HashMap;
use tauri::command;
use tokio::sync::RwLock;

// Global WebRTC state management
lazy_static::lazy_static! {
    static ref STREAMERS: RwLock<HashMap<String, WebRTCStreamer>> = RwLock::new(HashMap::new());
    static ref PEER_CONNECTIONS: RwLock<HashMap<String, PeerConnection>> = RwLock::new(HashMap::new());
    static ref STREAM_TO_PEER: RwLock<HashMap<String, String>> = RwLock::new(HashMap::new());
}

fn build_rtp_packet_bytes(payload_type: u8, ssrc: u32, rtp_packet: &crate::webrtc::streaming::RtpPayload) -> Vec<u8> {
    let mut rtp_bytes = Vec::with_capacity(12 + rtp_packet.data.len());

    // RTP header (12 bytes)
    // Version: 2, Padding: 0, Extension: 0, CSRC count: 0
    rtp_bytes.push(0x80);

    // Payload type (7 bits) + marker bit (MSB)
    let marker_bit = if rtp_packet.marker { 0x80 } else { 0x00 };
    rtp_bytes.push(marker_bit | (payload_type & 0x7F));

    // Sequence number (big endian)
    rtp_bytes.extend_from_slice(&rtp_packet.sequence_number.to_be_bytes());

    // Timestamp (RTP is 32-bit). Use low 32 bits to avoid widening the header.
    let ts32 = (rtp_packet.timestamp & 0xFFFF_FFFF) as u32;
    rtp_bytes.extend_from_slice(&ts32.to_be_bytes());

    // SSRC
    rtp_bytes.extend_from_slice(&ssrc.to_be_bytes());

    // Payload
    rtp_bytes.extend_from_slice(&rtp_packet.data);

    rtp_bytes
}

/// Start WebRTC streaming for a camera
#[command]
pub async fn start_webrtc_stream(
    device_id: String,
    stream_id: String,
    _config: Option<StreamConfig>,
    mode: Option<crate::webrtc::StreamMode>,
) -> Result<String, String> {
    log::info!(
        "Starting WebRTC stream {} for device {}",
        stream_id, device_id
    );

    // Prevent duplicate stream IDs up-front to avoid spawning a stream task that can't be tracked.
    {
        let streamers = STREAMERS.read().await;
        if streamers.contains_key(&stream_id) {
            return Err(format!("WebRTC stream {} already exists", stream_id));
        }
    }

    // Create streamer with default config if none provided
    let config = _config.unwrap_or_default();
    let streamer = WebRTCStreamer::new(stream_id.clone(), config);

    // Set mode if provided
    if let Some(stream_mode) = mode {
        streamer.set_mode(stream_mode).await;
    }

    // Initialize the RTP packetizer for H.264 so RTP forwarding works by default.
    // (This is required for associate_stream_with_peer to function.)
    streamer.init_h264_packetizer(1200).await;

    // Start the actual streaming
    streamer.start_streaming(device_id).await?;

    // Store in global map
    let mut streamers = STREAMERS.write().await;
    streamers.insert(stream_id.clone(), streamer);

    Ok(format!("WebRTC stream {} started", stream_id))
}

/// Stop WebRTC streaming
#[command]
pub async fn stop_webrtc_stream(stream_id: String) -> Result<String, String> {
    log::info!("Stopping WebRTC stream {}", stream_id);

    let mut streamers = STREAMERS.write().await;

    if let Some(streamer) = streamers.get(&stream_id) {
        streamer.stop_streaming().await?;
        streamers.remove(&stream_id);
        Ok(format!("WebRTC stream {} stopped", stream_id))
    } else {
        Err(format!("WebRTC stream {} not found", stream_id))
    }
}

/// Get WebRTC stream status
#[command]
pub async fn get_webrtc_stream_status(stream_id: String) -> Result<StreamStats, String> {
    let streamers = STREAMERS.read().await;

    if let Some(streamer) = streamers.get(&stream_id) {
        Ok(streamer.get_stats().await)
    } else {
        Err(format!("WebRTC stream {} not found", stream_id))
    }
}

/// Update WebRTC stream configuration
#[command]
pub async fn update_webrtc_config(
    stream_id: String,
    config: StreamConfig,
) -> Result<String, String> {
    log::info!("Updating WebRTC stream {} configuration", stream_id);

    let streamers = STREAMERS.read().await;

    if let Some(streamer) = streamers.get(&stream_id) {
        streamer.update_config(config).await?;
        Ok(format!("WebRTC stream {} configuration updated", stream_id))
    } else {
        Err(format!("WebRTC stream {} not found", stream_id))
    }
}

/// List all active WebRTC streams
#[command]
pub async fn list_webrtc_streams() -> Result<Vec<StreamStats>, String> {
    let streamers = STREAMERS.read().await;
    let mut streams = Vec::new();

    for (_, streamer) in streamers.iter() {
        streams.push(streamer.get_stats().await);
    }

    Ok(streams)
}

/// Associate WebRTC stream with peer connection
#[command]
pub async fn associate_stream_with_peer(
    stream_id: String,
    peer_id: String,
) -> Result<String, String> {
    log::info!("Associating stream {} with peer {}", stream_id, peer_id);

    // Check if stream exists
    let streamers = STREAMERS.read().await;
    let streamer = match streamers.get(&stream_id) {
        Some(s) => s,
        None => return Err(format!("WebRTC stream {} not found", stream_id)),
    };

    // Check if peer exists and clone it
    let peer = {
        let peers = PEER_CONNECTIONS.read().await;
        match peers.get(&peer_id) {
            Some(p) => p.clone(),
            None => return Err(format!("Peer connection {} not found", peer_id)),
        }
    };

    // Store association
    let mut associations = STREAM_TO_PEER.write().await;
    associations.insert(stream_id.clone(), peer_id.clone());

    // Set up RTP forwarding
    let (rtp_sender, mut rtp_receiver) = tokio::sync::mpsc::unbounded_channel();
    streamer.set_rtp_sender(rtp_sender).await;

    // Ensure peer has video transceivers set up
    let track_rids = peer.get_video_track_rids().await;
    if track_rids.is_empty() {
        // Set up default single-layer video transceiver
        let default_layer = crate::webrtc::streaming::SimulcastLayer {
            rid: "f".to_string(),
            width: 1280,
            height: 720,
            bitrate: 2_000_000,
            fps: 30,
        };
        peer.add_simulcast_video_transceivers(&[default_layer]).await
            .map_err(|e| format!("Failed to add video transceiver: {}", e))?;
    }

    // Get available track RIDs from peer (should have at least one now)
    let track_rids = peer.get_video_track_rids().await;
    if track_rids.is_empty() {
        return Err(format!("No video tracks available on peer {} after setup", peer_id));
    }

    // Use the first track RID (assuming single stream for now)
    let track_rid = track_rids[0].clone();

    // Start RTP forwarding task
    let peer_clone = peer.clone();
    let track_rid_clone = track_rid.clone();
    let stream_id_clone = stream_id.clone();
    let peer_id_clone = peer_id.clone();
    tokio::spawn(async move {
        log::info!("Starting RTP forwarding from stream {} to peer {} track {}", stream_id_clone, peer_id_clone, track_rid_clone);
        
        while let Some(rtp_packet) = rtp_receiver.recv().await {
            // Convert RtpPayload to raw RTP bytes for sending
            let rtp_bytes = build_rtp_packet_bytes(96, 12345u32, &rtp_packet);

            if let Err(e) = peer_clone.send_rtp_to_track(&track_rid_clone, &rtp_bytes).await {
                log::error!("Failed to send RTP packet to peer {}: {}", peer_id_clone, e);
                break;
            }
        }
        
        log::info!("RTP forwarding stopped for stream {} to peer {}", stream_id_clone, peer_id_clone);
    });

    Ok(format!("Stream {} associated with peer {} (track {})", stream_id, peer_id, track_rid))
}

#[cfg(test)]
mod rtp_tests {
    use super::*;
    use crate::webrtc::streaming::RtpPayload;

    #[test]
    fn rtp_packet_bytes_has_correct_header_shape() {
        let payload = RtpPayload {
            data: vec![1, 2, 3, 4, 5],
            timestamp: (u32::MAX as u64) + 123,
            sequence_number: 4242,
            marker: true,
        };

        let bytes = build_rtp_packet_bytes(96, 0xAABBCCDD, &payload);

        assert_eq!(bytes.len(), 12 + payload.data.len());
        assert_eq!(bytes[0], 0x80);
        assert_eq!(bytes[1] & 0x7F, 96);
        assert_eq!(bytes[1] & 0x80, 0x80);
        assert_eq!(u16::from_be_bytes([bytes[2], bytes[3]]), payload.sequence_number);

        let ts32 = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(ts32, (payload.timestamp & 0xFFFF_FFFF) as u32);

        let ssrc = u32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
        assert_eq!(ssrc, 0xAABBCCDD);

        assert_eq!(&bytes[12..], &payload.data);
    }
}

/// Create WebRTC peer connection
#[command]
pub async fn create_peer_connection(
    peer_id: String,
    config: Option<RTCConfiguration>,
) -> Result<String, String> {
    log::info!("Creating WebRTC peer connection {}", peer_id);

    let rtc_config = config.unwrap_or_default();
    let peer = PeerConnection::new(peer_id.clone(), rtc_config).await?;

    // Store peer connection
    let mut peers = PEER_CONNECTIONS.write().await;
    peers.insert(peer_id.clone(), peer);

    Ok(format!("Peer connection {} created", peer_id))
}

/// Create SDP offer
#[command]
pub async fn create_webrtc_offer(peer_id: String) -> Result<SessionDescription, String> {
    log::info!("Creating SDP offer for peer {}", peer_id);

    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.create_offer().await
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Create SDP answer
#[command]
pub async fn create_webrtc_answer(peer_id: String) -> Result<SessionDescription, String> {
    log::info!("Creating SDP answer for peer {}", peer_id);

    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.create_answer().await
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Set remote description
#[command]
pub async fn set_remote_description(
    peer_id: String,
    description: SessionDescription,
) -> Result<String, String> {
    log::info!("Setting remote description for peer {}", peer_id);

    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.set_remote_description(description).await?;
        Ok(format!("Remote description set for peer {}", peer_id))
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Add ICE candidate
#[command]
pub async fn add_ice_candidate(peer_id: String, candidate: IceCandidate) -> Result<String, String> {
    log::debug!("Adding ICE candidate for peer {}", peer_id);

    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.add_ice_candidate(candidate).await?;
        Ok(format!("ICE candidate added for peer {}", peer_id))
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Get local ICE candidates
#[command]
pub async fn get_local_ice_candidates(peer_id: String) -> Result<Vec<IceCandidate>, String> {
    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        Ok(peer.get_local_candidates().await)
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Add video transceivers for simulcast streaming
#[command]
pub async fn add_video_transceivers(peer_id: String, layers: Vec<crate::webrtc::streaming::SimulcastLayer>) -> Result<String, String> {
    log::info!("Adding video transceivers for peer {} with {} layers", peer_id, layers.len());

    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.add_simulcast_video_transceivers(&layers).await?;
        Ok(format!("Video transceivers added for peer {}", peer_id))
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Create data channel
#[command]
pub async fn create_data_channel(peer_id: String, channel_label: String) -> Result<String, String> {
    log::info!(
        "Creating data channel '{}' for peer {}",
        channel_label,
        peer_id
    );

    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.create_data_channel(channel_label).await
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Send data through data channel
#[command]
pub async fn send_data_channel_message(
    peer_id: String,
    channel_label: String,
    data: Vec<u8>,
) -> Result<String, String> {
    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        peer.send_data(&channel_label, data).await?;
        Ok(format!(
            "Data sent through channel '{}' on peer {}",
            channel_label, peer_id
        ))
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Get peer connection status
#[command]
pub async fn get_peer_connection_status(peer_id: String) -> Result<PeerConnectionStats, String> {
    let peers = PEER_CONNECTIONS.read().await;

    if let Some(peer) = peers.get(&peer_id) {
        Ok(peer.get_stats().await)
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// Close peer connection
#[command]
pub async fn close_peer_connection(peer_id: String) -> Result<String, String> {
    log::info!("Closing peer connection {}", peer_id);

    let mut peers = PEER_CONNECTIONS.write().await;

    if let Some(peer) = peers.remove(&peer_id) {
        peer.close().await?;
        Ok(format!("Peer connection {} closed", peer_id))
    } else {
        Err(format!("Peer connection {} not found", peer_id))
    }
}

/// List all peer connections
#[command]
pub async fn list_peer_connections() -> Result<Vec<PeerConnectionStats>, String> {
    let peers = PEER_CONNECTIONS.read().await;
    let mut connections = Vec::new();

    for (_, peer) in peers.iter() {
        connections.push(peer.get_stats().await);
    }

    Ok(connections)
}

/// Get WebRTC system status
#[command]
pub async fn get_webrtc_system_status() -> Result<WebRTCSystemStatus, String> {
    let streamers = STREAMERS.read().await;
    let peers = PEER_CONNECTIONS.read().await;

    let mut active_streams = 0;
    let mut total_subscribers = 0;

    for (_, streamer) in streamers.iter() {
        let stats = streamer.get_stats().await;
        if stats.is_active {
            active_streams += 1;
        }
        total_subscribers += stats.subscribers;
    }

    let mut connected_peers = 0;
    for (_, peer) in peers.iter() {
        let state = peer.get_connection_state().await;
        if matches!(state, ConnectionState::Connected) {
            connected_peers += 1;
        }
    }

    Ok(WebRTCSystemStatus {
        total_streams: streamers.len(),
        active_streams,
        total_subscribers,
        total_peers: peers.len(),
        connected_peers,
    })
}

/// Pause WebRTC stream
#[command]
pub async fn pause_webrtc_stream(stream_id: String) -> Result<String, String> {
    let streamers = STREAMERS.read().await;
    if let Some(streamer) = streamers.get(&stream_id) {
        streamer.pause_stream().await;
        Ok(format!("Stream {} paused", stream_id))
    } else {
        Err(format!("WebRTC stream {} not found", stream_id))
    }
}

/// Resume WebRTC stream
#[command]
pub async fn resume_webrtc_stream(stream_id: String) -> Result<String, String> {
    let streamers = STREAMERS.read().await;
    if let Some(streamer) = streamers.get(&stream_id) {
        streamer.resume_stream().await;
        Ok(format!("Stream {} resumed", stream_id))
    } else {
        Err(format!("WebRTC stream {} not found", stream_id))
    }
}

/// Set bitrate for WebRTC stream
#[command]
pub async fn set_webrtc_stream_bitrate(stream_id: String, bitrate: u32) -> Result<String, String> {
    let streamers = STREAMERS.read().await;
    if let Some(streamer) = streamers.get(&stream_id) {
        streamer.set_bitrate(bitrate).await;
        Ok(format!("Stream {} bitrate set to {} bps", stream_id, bitrate))
    } else {
        Err(format!("WebRTC stream {} not found", stream_id))
    }
}

/// WebRTC system status
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WebRTCSystemStatus {
    pub total_streams: usize,
    pub active_streams: usize,
    pub total_subscribers: usize,
    pub total_peers: usize,
    pub connected_peers: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webrtc_stream_lifecycle() {
        let device_id = "test_device".to_string();
        let stream_id = "test_stream".to_string();

        // Start stream
        let result = start_webrtc_stream(
            device_id,
            stream_id.clone(),
            None,
            Some(crate::webrtc::StreamMode::SyntheticTest),
        )
        .await;
        assert!(result.is_ok());

        // Check status
        let status = get_webrtc_stream_status(stream_id.clone()).await;
        assert!(status.is_ok());

        // Stop stream
        let result = stop_webrtc_stream(stream_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_peer_connection_lifecycle() {
        let peer_id = "test_peer".to_string();

        // Create peer connection
        let result = create_peer_connection(peer_id.clone(), None).await;
        assert!(result.is_ok());

        // Create offer
        let offer = create_webrtc_offer(peer_id.clone()).await;
        assert!(offer.is_ok());

        // Get status
        let status = get_peer_connection_status(peer_id.clone()).await;
        assert!(status.is_ok());

        // Close connection
        let result = close_peer_connection(peer_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_system_status() {
        let status = get_webrtc_system_status().await;
        assert!(status.is_ok());

        let _status = status.unwrap();
        // total_streams is u32, always >= 0
        // total_peers is u32, always >= 0
    }
}
