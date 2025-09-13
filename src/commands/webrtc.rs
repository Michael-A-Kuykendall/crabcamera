use tauri::command;
use crate::webrtc::{WebRTCStreamer, StreamConfig, PeerConnection, RTCConfiguration, SessionDescription, IceCandidate};
use crate::webrtc::streaming::{StreamStats};
use crate::webrtc::peer::{PeerConnectionStats, ConnectionState};
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

// Global WebRTC state management
lazy_static::lazy_static! {
    static ref STREAMERS: Arc<RwLock<HashMap<String, WebRTCStreamer>>> = Arc::new(RwLock::new(HashMap::new()));
    static ref PEER_CONNECTIONS: Arc<RwLock<HashMap<String, PeerConnection>>> = Arc::new(RwLock::new(HashMap::new()));
}

/// Start WebRTC streaming for a camera
#[command]
pub async fn start_webrtc_stream(
    device_id: String,
    stream_id: String,
    config: Option<StreamConfig>
) -> Result<String, String> {
    log::info!("Starting WebRTC stream {} for device {}", stream_id, device_id);
    
    let stream_config = config.unwrap_or_default();
    let streamer = WebRTCStreamer::new(stream_id.clone(), stream_config);
    
    // Start streaming
    streamer.start_streaming(device_id.clone()).await?;
    
    // Store streamer
    let mut streamers = STREAMERS.write().await;
    streamers.insert(stream_id.clone(), streamer);
    
    Ok(format!("WebRTC stream {} started for device {}", stream_id, device_id))
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
    config: StreamConfig
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

/// Create WebRTC peer connection
#[command]
pub async fn create_peer_connection(
    peer_id: String,
    config: Option<RTCConfiguration>
) -> Result<String, String> {
    log::info!("Creating WebRTC peer connection {}", peer_id);
    
    let rtc_config = config.unwrap_or_default();
    let peer = PeerConnection::new(peer_id.clone(), rtc_config);
    
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
    description: SessionDescription
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
pub async fn add_ice_candidate(
    peer_id: String,
    candidate: IceCandidate
) -> Result<String, String> {
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

/// Create data channel
#[command]
pub async fn create_data_channel(
    peer_id: String,
    channel_label: String
) -> Result<String, String> {
    log::info!("Creating data channel '{}' for peer {}", channel_label, peer_id);
    
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
    data: Vec<u8>
) -> Result<String, String> {
    let peers = PEER_CONNECTIONS.read().await;
    
    if let Some(peer) = peers.get(&peer_id) {
        peer.send_data(&channel_label, data).await?;
        Ok(format!("Data sent through channel '{}' on peer {}", channel_label, peer_id))
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
        let result = start_webrtc_stream(device_id, stream_id.clone(), None).await;
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
        
        let status = status.unwrap();
        assert!(status.total_streams >= 0);
        assert!(status.total_peers >= 0);
    }
}