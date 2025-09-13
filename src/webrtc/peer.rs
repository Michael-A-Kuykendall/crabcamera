use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// WebRTC peer connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RTCConfiguration {
    pub ice_servers: Vec<IceServer>,
    pub ice_transport_policy: IceTransportPolicy,
    pub bundle_policy: BundlePolicy,
}

impl Default for RTCConfiguration {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                IceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    username: None,
                    credential: None,
                }
            ],
            ice_transport_policy: IceTransportPolicy::All,
            bundle_policy: BundlePolicy::MaxBundle,
        }
    }
}

/// ICE server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

/// ICE transport policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IceTransportPolicy {
    None,
    Relay,
    All,
}

/// Bundle policy for RTC connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BundlePolicy {
    Balanced,
    MaxCompat,
    MaxBundle,
}

/// WebRTC peer connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionState {
    New,
    Connecting,
    Connected,
    Disconnected,
    Failed,
    Closed,
}

/// SDP (Session Description Protocol) type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SdpType {
    Offer,
    Answer,
    Pranswer,
    Rollback,
}

/// Session description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    pub sdp_type: SdpType,
    pub sdp: String,
}

/// ICE candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IceCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
}

/// WebRTC peer connection manager
pub struct PeerConnection {
    id: String,
    config: RTCConfiguration,
    state: Arc<RwLock<ConnectionState>>,
    local_description: Arc<RwLock<Option<SessionDescription>>>,
    remote_description: Arc<RwLock<Option<SessionDescription>>>,
    ice_candidates: Arc<RwLock<Vec<IceCandidate>>>,
    data_channels: Arc<RwLock<HashMap<String, DataChannel>>>,
}

/// Data channel for peer-to-peer communication
#[derive(Debug, Clone)]
pub struct DataChannel {
    pub id: String,
    pub label: String,
    pub ordered: bool,
    pub max_retransmits: Option<u16>,
    pub state: DataChannelState,
}

/// Data channel state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataChannelState {
    Connecting,
    Open,
    Closing,
    Closed,
}

impl PeerConnection {
    /// Create a new peer connection
    pub fn new(id: String, config: RTCConfiguration) -> Self {
        Self {
            id,
            config,
            state: Arc::new(RwLock::new(ConnectionState::New)),
            local_description: Arc::new(RwLock::new(None)),
            remote_description: Arc::new(RwLock::new(None)),
            ice_candidates: Arc::new(RwLock::new(Vec::new())),
            data_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Get peer connection ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// Get current connection state
    pub async fn get_connection_state(&self) -> ConnectionState {
        self.state.read().await.clone()
    }
    
    /// Create SDP offer
    pub async fn create_offer(&self) -> Result<SessionDescription, String> {
        log::info!("Creating SDP offer for peer {}", self.id);
        
        // In a real implementation, this would generate actual SDP
        let sdp_content = self.generate_mock_sdp_offer().await;
        
        let offer = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: sdp_content,
        };
        
        // Set as local description
        *self.local_description.write().await = Some(offer.clone());
        *self.state.write().await = ConnectionState::Connecting;
        
        Ok(offer)
    }
    
    /// Create SDP answer
    pub async fn create_answer(&self) -> Result<SessionDescription, String> {
        log::info!("Creating SDP answer for peer {}", self.id);
        
        // Ensure we have a remote offer
        let remote_desc = self.remote_description.read().await;
        if remote_desc.is_none() {
            return Err("No remote offer to answer".to_string());
        }
        
        let sdp_content = self.generate_mock_sdp_answer().await;
        
        let answer = SessionDescription {
            sdp_type: SdpType::Answer,
            sdp: sdp_content,
        };
        
        // Set as local description
        *self.local_description.write().await = Some(answer.clone());
        *self.state.write().await = ConnectionState::Connected;
        
        Ok(answer)
    }
    
    /// Set remote description
    pub async fn set_remote_description(&self, desc: SessionDescription) -> Result<(), String> {
        log::info!("Setting remote description for peer {}", self.id);
        
        *self.remote_description.write().await = Some(desc);
        
        // Update state based on description type
        let current_state = self.get_connection_state().await;
        match current_state {
            ConnectionState::New => {
                *self.state.write().await = ConnectionState::Connecting;
            }
            ConnectionState::Connecting => {
                *self.state.write().await = ConnectionState::Connected;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Add ICE candidate
    pub async fn add_ice_candidate(&self, candidate: IceCandidate) -> Result<(), String> {
        log::debug!("Adding ICE candidate for peer {}: {}", self.id, candidate.candidate);
        
        self.ice_candidates.write().await.push(candidate);
        Ok(())
    }
    
    /// Get local ICE candidates
    pub async fn get_local_candidates(&self) -> Vec<IceCandidate> {
        // In a real implementation, this would gather actual ICE candidates
        vec![
            IceCandidate {
                candidate: "candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host".to_string(),
                sdp_mid: Some("0".to_string()),
                sdp_mline_index: Some(0),
            }
        ]
    }
    
    /// Create data channel
    pub async fn create_data_channel(&self, label: String) -> Result<String, String> {
        log::info!("Creating data channel '{}' for peer {}", label, self.id);
        
        let channel_id = format!("{}_{}", self.id, label);
        let channel = DataChannel {
            id: channel_id.clone(),
            label: label.clone(),
            ordered: true,
            max_retransmits: None,
            state: DataChannelState::Connecting,
        };
        
        self.data_channels.write().await.insert(label, channel);
        Ok(channel_id)
    }
    
    /// Send data through channel
    pub async fn send_data(&self, channel_label: &str, data: Vec<u8>) -> Result<(), String> {
        let channels = self.data_channels.read().await;
        if let Some(channel) = channels.get(channel_label) {
            match channel.state {
                DataChannelState::Open => {
                    log::debug!("Sending {} bytes through channel '{}'", data.len(), channel_label);
                    // In a real implementation, send data through WebRTC data channel
                    Ok(())
                }
                _ => Err(format!("Data channel '{}' is not open", channel_label))
            }
        } else {
            Err(format!("Data channel '{}' not found", channel_label))
        }
    }
    
    /// Close peer connection
    pub async fn close(&self) -> Result<(), String> {
        log::info!("Closing peer connection {}", self.id);
        
        *self.state.write().await = ConnectionState::Closed;
        
        // Close all data channels
        let mut channels = self.data_channels.write().await;
        for (_, channel) in channels.iter_mut() {
            channel.state = DataChannelState::Closed;
        }
        
        Ok(())
    }
    
    /// Get connection statistics
    pub async fn get_stats(&self) -> PeerConnectionStats {
        let state = self.get_connection_state().await;
        let ice_candidates = self.ice_candidates.read().await;
        let data_channels = self.data_channels.read().await;
        
        PeerConnectionStats {
            peer_id: self.id.clone(),
            state,
            ice_candidates_count: ice_candidates.len(),
            data_channels_count: data_channels.len(),
            has_local_description: self.local_description.read().await.is_some(),
            has_remote_description: self.remote_description.read().await.is_some(),
        }
    }
    
    /// Generate mock SDP offer
    async fn generate_mock_sdp_offer(&self) -> String {
        format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             m=video 9 UDP/TLS/RTP/SAVPF 96\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=rtcp:9 IN IP4 0.0.0.0\r\n\
             a=ice-ufrag:test\r\n\
             a=ice-pwd:testpassword\r\n\
             a=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\n\
             a=setup:actpass\r\n\
             a=mid:0\r\n\
             a=sendrecv\r\n\
             a=rtcp-mux\r\n\
             a=rtpmap:96 H264/90000\r\n",
            chrono::Utc::now().timestamp()
        )
    }
    
    /// Generate mock SDP answer
    async fn generate_mock_sdp_answer(&self) -> String {
        format!(
            "v=0\r\n\
             o=- {} 2 IN IP4 127.0.0.1\r\n\
             s=-\r\n\
             t=0 0\r\n\
             m=video 9 UDP/TLS/RTP/SAVPF 96\r\n\
             c=IN IP4 0.0.0.0\r\n\
             a=rtcp:9 IN IP4 0.0.0.0\r\n\
             a=ice-ufrag:test\r\n\
             a=ice-pwd:testpassword\r\n\
             a=fingerprint:sha-256 00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00:00\r\n\
             a=setup:active\r\n\
             a=mid:0\r\n\
             a=sendrecv\r\n\
             a=rtcp-mux\r\n\
             a=rtpmap:96 H264/90000\r\n",
            chrono::Utc::now().timestamp()
        )
    }
}

/// Peer connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConnectionStats {
    pub peer_id: String,
    pub state: ConnectionState,
    pub ice_candidates_count: usize,
    pub data_channels_count: usize,
    pub has_local_description: bool,
    pub has_remote_description: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_peer_connection_creation() {
        let config = RTCConfiguration::default();
        let peer = PeerConnection::new("test_peer".to_string(), config);
        
        assert_eq!(peer.id(), "test_peer");
        assert!(matches!(peer.get_connection_state().await, ConnectionState::New));
    }
    
    #[tokio::test]
    async fn test_sdp_offer_creation() {
        let config = RTCConfiguration::default();
        let peer = PeerConnection::new("test_peer".to_string(), config);
        
        let offer = peer.create_offer().await;
        assert!(offer.is_ok());
        
        let offer = offer.unwrap();
        assert!(matches!(offer.sdp_type, SdpType::Offer));
        assert!(!offer.sdp.is_empty());
        assert!(matches!(peer.get_connection_state().await, ConnectionState::Connecting));
    }
    
    #[tokio::test]
    async fn test_sdp_answer_creation() {
        let config = RTCConfiguration::default();
        let peer = PeerConnection::new("test_peer".to_string(), config);
        
        // Set remote offer first
        let remote_offer = SessionDescription {
            sdp_type: SdpType::Offer,
            sdp: "mock offer sdp".to_string(),
        };
        
        let result = peer.set_remote_description(remote_offer).await;
        assert!(result.is_ok());
        
        // Now create answer
        let answer = peer.create_answer().await;
        assert!(answer.is_ok());
        
        let answer = answer.unwrap();
        assert!(matches!(answer.sdp_type, SdpType::Answer));
        assert!(matches!(peer.get_connection_state().await, ConnectionState::Connected));
    }
    
    #[tokio::test]
    async fn test_ice_candidate_handling() {
        let config = RTCConfiguration::default();
        let peer = PeerConnection::new("test_peer".to_string(), config);
        
        let candidate = IceCandidate {
            candidate: "test candidate".to_string(),
            sdp_mid: Some("0".to_string()),
            sdp_mline_index: Some(0),
        };
        
        let result = peer.add_ice_candidate(candidate).await;
        assert!(result.is_ok());
        
        let local_candidates = peer.get_local_candidates().await;
        assert!(!local_candidates.is_empty());
    }
    
    #[tokio::test]
    async fn test_data_channel_creation() {
        let config = RTCConfiguration::default();
        let peer = PeerConnection::new("test_peer".to_string(), config);
        
        let channel_id = peer.create_data_channel("test_channel".to_string()).await;
        assert!(channel_id.is_ok());
        
        let stats = peer.get_stats().await;
        assert_eq!(stats.data_channels_count, 1);
    }
    
    #[tokio::test]
    async fn test_connection_close() {
        let config = RTCConfiguration::default();
        let peer = PeerConnection::new("test_peer".to_string(), config);
        
        let result = peer.close().await;
        assert!(result.is_ok());
        assert!(matches!(peer.get_connection_state().await, ConnectionState::Closed));
    }
}