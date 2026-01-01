use crate::webrtc::peer::{PeerConnection, RTCConfiguration};
use crate::webrtc::streaming::WebRTCStreamer;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Conference state for synchronization
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConferenceState {
    pub conference_id: String,
    pub participants: Vec<String>,
    pub is_streaming: bool,
    pub created_at: std::time::SystemTime,
}

/// WebRTC conference manager for multi-peer streaming
#[derive(Clone)]
pub struct WebRTCConference {
    id: String,
    streamer: WebRTCStreamer,
    participants: Arc<RwLock<HashMap<String, PeerConnection>>>,
    config: RTCConfiguration,
    created_at: std::time::SystemTime,
}

impl WebRTCConference {
    /// Create a new conference
    pub fn new(id: String, streamer: WebRTCStreamer, config: RTCConfiguration) -> Self {
        Self {
            id,
            streamer,
            participants: Arc::new(RwLock::new(HashMap::new())),
            config,
            created_at: std::time::SystemTime::now(),
        }
    }

    /// Get conference ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Add a participant to the conference
    pub async fn join(&self, participant_id: String) -> Result<String, String> {
        log::info!("Participant {} joining conference {}", participant_id, self.id);

        // Check if participant already exists
        {
            let participants = self.participants.read().await;
            if participants.contains_key(&participant_id) {
                return Err(format!("Participant {} already in conference {}", participant_id, self.id));
            }
        }

        // Create peer connection for the participant
        let peer = PeerConnection::new(participant_id.clone(), self.config.clone()).await?;

        // Store the participant
        {
            let mut participants = self.participants.write().await;
            participants.insert(participant_id.clone(), peer);
        }

        // Connect participant to the stream
        self.connect_participant_to_stream(&participant_id).await?;

        log::info!("Participant {} successfully joined conference {}", participant_id, self.id);

        // Broadcast updated state to all participants
        if let Err(e) = self.broadcast_state().await {
            log::warn!("Failed to broadcast state after join: {}", e);
        }

        // Send participant discovery info to the new participant
        if let Err(e) = self.send_participant_discovery(&participant_id).await {
            log::warn!("Failed to send discovery info to {}: {}", participant_id, e);
        }

        Ok(format!("Participant {} joined conference {}", participant_id, self.id))
    }

    /// Remove a participant from the conference
    pub async fn leave(&self, participant_id: &str) -> Result<String, String> {
        log::info!("Participant {} leaving conference {}", participant_id, self.id);

        // Disconnect from stream first
        self.disconnect_participant_from_stream(participant_id).await?;

        let mut participants = self.participants.write().await;
        if let Some(peer) = participants.remove(participant_id) {
            // Close the peer connection
            peer.close().await?;
            log::info!("Participant {} successfully left conference {}", participant_id, self.id);

            // Broadcast updated state to remaining participants
            if let Err(e) = self.broadcast_state().await {
                log::warn!("Failed to broadcast state after leave: {}", e);
            }

            Ok(format!("Participant {} left conference {}", participant_id, self.id))
        } else {
            Err(format!("Participant {} not found in conference {}", participant_id, self.id))
        }
    }

    /// Get list of participant IDs
    pub async fn get_participants(&self) -> Vec<String> {
        let participants = self.participants.read().await;
        participants.keys().cloned().collect()
    }

    /// Get conference state for synchronization
    pub async fn get_state(&self) -> ConferenceState {
        let participants = self.participants.read().await;
        let participant_ids: Vec<String> = participants.keys().cloned().collect();

        ConferenceState {
            conference_id: self.id.clone(),
            participants: participant_ids,
            is_streaming: self.streamer.is_streaming().await,
            created_at: self.created_at,
        }
    }

    /// Broadcast conference state to all participants
    pub async fn broadcast_state(&self) -> Result<(), String> {
        let state = self.get_state().await;
        let state_json = serde_json::to_string(&state)
            .map_err(|e| format!("Failed to serialize state: {}", e))?;

        let participants = self.participants.read().await;
        for (participant_id, peer) in participants.iter() {
            if let Err(e) = peer.send_data_channel_message("conference-state", &state_json).await {
                log::warn!("Failed to send state to participant {}: {}", participant_id, e);
            }
        }

        Ok(())
    }

    /// Send participant discovery information to a specific participant
    pub async fn send_participant_discovery(&self, target_participant_id: &str) -> Result<(), String> {
        let participants = self.participants.read().await;
        let target_peer = participants.get(target_participant_id)
            .ok_or_else(|| format!("Target participant {} not found", target_participant_id))?;

        // Get all other participants
        let other_participants: Vec<String> = participants.keys()
            .filter(|&id| id != target_participant_id)
            .cloned()
            .collect();

        let discovery_info = serde_json::json!({
            "conference_id": self.id,
            "other_participants": other_participants,
            "total_participants": participants.len()
        });

        let discovery_json = serde_json::to_string(&discovery_info)
            .map_err(|e| format!("Failed to serialize discovery info: {}", e))?;

        target_peer.send_data_channel_message("participant-discovery", &discovery_json).await
    }

    /// Get participant count
    pub async fn participant_count(&self) -> usize {
        let participants = self.participants.read().await;
        participants.len()
    }

    /// Get a specific participant
    pub async fn get_participant(&self, participant_id: &str) -> Option<PeerConnection> {
        let participants = self.participants.read().await;
        participants.get(participant_id).cloned()
    }

    /// Connect a participant to the stream (set up RTP forwarding)
    pub async fn connect_participant_to_stream(&self, participant_id: &str) -> Result<(), String> {
        let participants = self.participants.read().await;
        if let Some(peer) = participants.get(participant_id) {
            // Create a channel for RTP packets
            let (rtp_sender, mut rtp_receiver) = tokio::sync::mpsc::unbounded_channel();

            // Add the sender to the streamer
            self.streamer.add_rtp_sender(rtp_sender).await;

            // Spawn a task to receive RTP packets and send to peer tracks
            let peer_clone = peer.clone();
            let participant_id = participant_id.to_string();
            tokio::spawn(async move {
                while let Some(rtp_payload) = rtp_receiver.recv().await {
                    // Send RTP packet to the peer's video track
                    // Use the default RID for now
                    if let Err(e) = peer_clone.send_rtp_to_track("f", &rtp_payload.data).await {
                        log::warn!("Failed to send RTP to participant {}: {}", participant_id, e);
                        break;
                    }
                }
                log::info!("RTP forwarding stopped for participant {}", participant_id);
            });

            Ok(())
        } else {
            Err(format!("Participant {} not found in conference {}", participant_id, self.id))
        }
    }

    /// Disconnect a participant from the stream
    pub async fn disconnect_participant_from_stream(&self, participant_id: &str) -> Result<(), String> {
        // For now, just clear all senders and reconnect others
        // In a more sophisticated implementation, we'd track which sender belongs to which peer
        self.streamer.clear_rtp_senders().await;

        // Reconnect remaining participants
        let participants = self.participants.read().await;
        for (id, _) in participants.iter() {
            if id != participant_id {
                if let Err(e) = self.connect_participant_to_stream(id).await {
                    log::warn!("Failed to reconnect participant {}: {}", id, e);
                }
            }
        }

        Ok(())
    }

    /// Close the entire conference
    pub async fn close(&self) -> Result<String, String> {
        log::info!("Closing conference {}", self.id);

        // Close all participant connections
        let mut participants = self.participants.write().await;
        for (participant_id, peer) in participants.drain() {
            if let Err(e) = peer.close().await {
                log::warn!("Error closing participant {}: {}", participant_id, e);
            }
        }

        // Stop the streamer
        self.streamer.stop_streaming().await?;

        log::info!("Conference {} closed", self.id);
        Ok(format!("Conference {} closed", self.id))
    }
}

/// Conference statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConferenceStats {
    pub conference_id: String,
    pub participant_count: usize,
    pub streamer_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl WebRTCConference {
    /// Monitor network quality across all participants
    pub async fn monitor_network_quality(&self) -> crate::webrtc::streaming::NetworkQuality {
        let participants = self.participants.read().await;

        if participants.is_empty() {
            return crate::webrtc::streaming::NetworkQuality::default();
        }

        let mut total_packet_loss = 0.0;
        let mut total_rtt = 0;
        let mut min_bandwidth = u32::MAX;
        let mut max_jitter = 0;
        let mut count = 0;

        for (_id, peer) in participants.iter() {
            let quality = peer.monitor_connection_quality().await;
            total_packet_loss += quality.packet_loss_rate;
            total_rtt += quality.round_trip_time;
            min_bandwidth = min_bandwidth.min(quality.available_bandwidth);
            max_jitter = max_jitter.max(quality.jitter);
            count += 1;
        }

        // Average the metrics
        crate::webrtc::streaming::NetworkQuality {
            packet_loss_rate: total_packet_loss / count as f32,
            round_trip_time: total_rtt / count as u32,
            available_bandwidth: min_bandwidth, // Use minimum bandwidth as limiting factor
            jitter: max_jitter, // Use maximum jitter as worst case
        }
    }

    /// Get conference statistics
    pub async fn get_stats(&self) -> ConferenceStats {
        ConferenceStats {
            conference_id: self.id.clone(),
            participant_count: self.participant_count().await,
            streamer_active: self.streamer.is_streaming().await,
            created_at: chrono::Utc::now(), // In a real implementation, you'd track creation time
        }
    }

    /// Clean up stale or failed participants
    pub async fn cleanup_stale_participants(&self) -> Result<(), String> {
        let mut participants = self.participants.write().await;
        let mut to_remove = Vec::new();

        for (participant_id, peer) in participants.iter() {
            if !peer.is_healthy().await || peer.is_timed_out(std::time::Duration::from_secs(60)).await {
                log::info!("Cleaning up stale participant {} from conference {}", participant_id, self.id);
                to_remove.push(participant_id.clone());

                // Force cleanup of peer resources
                if let Err(e) = peer.force_cleanup().await {
                    log::warn!("Failed to cleanup peer {}: {}", participant_id, e);
                }
            }
        }

        // Remove stale participants
        for participant_id in to_remove {
            participants.remove(&participant_id);
        }

        Ok(())
    }

    /// Force cleanup of entire conference
    pub async fn force_cleanup(&self) -> Result<(), String> {
        log::info!("Force cleanup of conference {}", self.id);

        // Clean up all participants
        let participants = self.participants.read().await;
        for (participant_id, peer) in participants.iter() {
            if let Err(e) = peer.force_cleanup().await {
                log::warn!("Failed to cleanup participant {}: {}", participant_id, e);
            }
        }

        // Clear participants map
        {
            let mut participants = self.participants.write().await;
            participants.clear();
        }

        // Stop streamer
        if let Err(e) = self.streamer.force_cleanup().await {
            log::warn!("Failed to cleanup streamer: {}", e);
        }

        Ok(())
    }
}