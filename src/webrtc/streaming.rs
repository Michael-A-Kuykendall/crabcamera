use crate::types::{CameraFrame};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

/// WebRTC streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub bitrate: u32,           // Target bitrate in bps
    pub max_fps: u32,          // Maximum frames per second
    pub width: u32,            // Stream width
    pub height: u32,           // Stream height
    pub codec: VideoCodec,     // Video codec
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            bitrate: 2_000_000,    // 2 Mbps
            max_fps: 30,
            width: 1280,
            height: 720,
            codec: VideoCodec::H264,
        }
    }
}

/// Supported video codecs for WebRTC streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    VP8,
    VP9,
    AV1,
}

/// WebRTC streaming manager
#[derive(Clone)]
pub struct WebRTCStreamer {
    config: Arc<RwLock<StreamConfig>>,
    frame_sender: Arc<broadcast::Sender<EncodedFrame>>,
    is_streaming: Arc<RwLock<bool>>,
    stream_id: String,
}

/// Encoded frame for WebRTC transmission
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub frame_type: FrameType,
    pub width: u32,
    pub height: u32,
}

/// Frame type for WebRTC streaming
#[derive(Debug, Clone)]
pub enum FrameType {
    Keyframe,   // I-frame
    Delta,      // P-frame
}

impl WebRTCStreamer {
    /// Create a new WebRTC streamer
    pub fn new(stream_id: String, config: StreamConfig) -> Self {
        let (frame_sender, _) = broadcast::channel(100); // Buffer 100 frames
        
        Self {
            config: Arc::new(RwLock::new(config)),
            frame_sender: Arc::new(frame_sender),
            is_streaming: Arc::new(RwLock::new(false)),
            stream_id,
        }
    }
    
    /// Start streaming camera frames
    pub async fn start_streaming(&self, device_id: String) -> Result<(), String> {
        let mut is_streaming = self.is_streaming.write().await;
        if *is_streaming {
            return Err("Stream already active".to_string());
        }
        
        *is_streaming = true;
        log::info!("Starting WebRTC stream {} for device {}", self.stream_id, device_id);
        
        // Start frame processing task
        let streamer = self.clone();
        tokio::spawn(async move {
            streamer.stream_processing_loop(device_id).await;
        });
        
        Ok(())
    }
    
    /// Stop streaming
    pub async fn stop_streaming(&self) -> Result<(), String> {
        let mut is_streaming = self.is_streaming.write().await;
        if !*is_streaming {
            return Ok(());
        }
        
        *is_streaming = false;
        log::info!("Stopping WebRTC stream {}", self.stream_id);
        Ok(())
    }
    
    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        *self.is_streaming.read().await
    }
    
    /// Get current stream configuration
    pub async fn get_config(&self) -> StreamConfig {
        self.config.read().await.clone()
    }
    
    /// Update stream configuration
    pub async fn update_config(&self, config: StreamConfig) -> Result<(), String> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        log::info!("Updated WebRTC stream configuration for {}", self.stream_id);
        Ok(())
    }
    
    /// Subscribe to encoded frames
    pub fn subscribe_frames(&self) -> broadcast::Receiver<EncodedFrame> {
        self.frame_sender.subscribe()
    }
    
    /// Get stream statistics
    pub async fn get_stats(&self) -> StreamStats {
        let config = self.get_config().await;
        let is_active = self.is_streaming().await;
        
        StreamStats {
            stream_id: self.stream_id.clone(),
            is_active,
            target_bitrate: config.bitrate,
            current_fps: if is_active { config.max_fps } else { 0 },
            resolution: (config.width, config.height),
            codec: config.codec,
            subscribers: self.frame_sender.receiver_count(),
        }
    }
    
    /// Process camera frames for WebRTC streaming
    async fn stream_processing_loop(&self, device_id: String) {
        log::info!("Starting stream processing loop for device {}", device_id);
        
        let mut frame_counter = 0u64;
        let mut last_keyframe = 0u64;
        let keyframe_interval = 30; // Keyframe every 30 frames
        
        while *self.is_streaming.read().await {
            // Simulate frame capture and encoding
            // In a real implementation, this would:
            // 1. Capture frame from camera
            // 2. Encode to H264/VP8/VP9
            // 3. Send encoded frame
            
            let config = self.get_config().await;
            let frame_type = if frame_counter - last_keyframe >= keyframe_interval {
                last_keyframe = frame_counter;
                FrameType::Keyframe
            } else {
                FrameType::Delta
            };
            
            let encoded_frame = EncodedFrame {
                data: self.create_mock_encoded_frame(&config, &frame_type).await,
                timestamp: frame_counter * (1000 / config.max_fps as u64),
                frame_type,
                width: config.width,
                height: config.height,
            };
            
            // Send frame to subscribers
            if let Err(_) = self.frame_sender.send(encoded_frame) {
                log::debug!("No subscribers for stream {}", self.stream_id);
            }
            
            frame_counter += 1;
            
            // Frame rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(
                1000 / config.max_fps as u64
            )).await;
        }
        
        log::info!("Stream processing loop ended for device {}", device_id);
    }
    
    /// Create mock encoded frame data
    /// TODO: Replace with actual video encoding
    async fn create_mock_encoded_frame(&self, config: &StreamConfig, frame_type: &FrameType) -> Vec<u8> {
        let base_size = match frame_type {
            FrameType::Keyframe => config.bitrate / (8 * config.max_fps), // Full frame
            FrameType::Delta => (config.bitrate / (8 * config.max_fps)) / 4, // Quarter size
        };
        
        vec![0u8; base_size as usize]
    }
}

/// Stream statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub stream_id: String,
    pub is_active: bool,
    pub target_bitrate: u32,
    pub current_fps: u32,
    pub resolution: (u32, u32),
    pub codec: VideoCodec,
    pub subscribers: usize,
}

/// Convert camera frame to WebRTC-compatible format
pub fn prepare_frame_for_webrtc(frame: &CameraFrame, config: &StreamConfig) -> Result<Vec<u8>, String> {
    // Resize frame if needed
    if frame.width != config.width || frame.height != config.height {
        log::debug!("Resizing frame from {}x{} to {}x{}", 
                   frame.width, frame.height, config.width, config.height);
        
        // TODO: Implement actual image resizing
        // For now, just return the original data
        Ok(frame.data.clone())
    } else {
        Ok(frame.data.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_webrtc_streamer_creation() {
        let config = StreamConfig::default();
        let streamer = WebRTCStreamer::new("test_stream".to_string(), config);
        
        assert!(!streamer.is_streaming().await);
        assert_eq!(streamer.stream_id, "test_stream");
    }
    
    #[tokio::test]
    async fn test_start_stop_streaming() {
        let config = StreamConfig::default();
        let streamer = WebRTCStreamer::new("test_stream".to_string(), config);
        
        // Start streaming
        let result = streamer.start_streaming("mock_camera".to_string()).await;
        assert!(result.is_ok());
        assert!(streamer.is_streaming().await);
        
        // Stop streaming
        let result = streamer.stop_streaming().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_config_update() {
        let config = StreamConfig::default();
        let streamer = WebRTCStreamer::new("test_stream".to_string(), config);
        
        let new_config = StreamConfig {
            bitrate: 4_000_000,
            max_fps: 60,
            width: 1920,
            height: 1080,
            codec: VideoCodec::VP9,
        };
        
        let result = streamer.update_config(new_config.clone()).await;
        assert!(result.is_ok());
        
        let current_config = streamer.get_config().await;
        assert_eq!(current_config.bitrate, 4_000_000);
        assert_eq!(current_config.max_fps, 60);
    }
    
    #[tokio::test]
    async fn test_frame_subscription() {
        let config = StreamConfig::default();
        let streamer = WebRTCStreamer::new("test_stream".to_string(), config);
        
        let mut receiver = streamer.subscribe_frames();
        
        // Start streaming
        let _ = streamer.start_streaming("mock_camera".to_string()).await;
        
        // Should receive frames
        tokio::time::timeout(
            tokio::time::Duration::from_millis(100), 
            receiver.recv()
        ).await.expect("Should receive frame").expect("Frame should be valid");
        
        // Stop streaming
        let _ = streamer.stop_streaming().await;
    }
    
    #[tokio::test]
    async fn test_stream_stats() {
        let config = StreamConfig::default();
        let streamer = WebRTCStreamer::new("test_stream".to_string(), config);
        
        let stats = streamer.get_stats().await;
        assert_eq!(stats.stream_id, "test_stream");
        assert!(!stats.is_active);
        assert_eq!(stats.subscribers, 0);
        
        // Subscribe to frames
        let _receiver = streamer.subscribe_frames();
        let stats = streamer.get_stats().await;
        assert_eq!(stats.subscribers, 1);
    }
}