use openh264::encoder::Encoder;
use openh264::formats::YUVBuffer;
#[cfg(feature = "recording")]
use openh264::encoder::FrameType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use crate::CameraFrame;

// Opus encoder imports
#[cfg(feature = "audio")]
use libopus_sys::{
    opus_encode_float, opus_encoder_create, opus_encoder_ctl, opus_encoder_destroy,
    OpusEncoder, OPUS_APPLICATION_VOIP, OPUS_OK, OPUS_SET_BITRATE_REQUEST,
};

/// Convert RGB24 to YUV420 planar format
#[cfg(feature = "recording")]
fn rgb_to_yuv420(rgb: &[u8], width: u32, height: u32) -> Vec<u8> {
    let w = width as usize;
    let h = height as usize;

    // YUV420: Y plane (w*h) + U plane (w/2 * h/2) + V plane (w/2 * h/2)
    let y_size = w * h;
    let uv_size = (w / 2) * (h / 2);
    let mut yuv = vec![0u8; y_size + uv_size * 2];

    let (y_plane, uv_planes) = yuv.split_at_mut(y_size);
    let (u_plane, v_plane) = uv_planes.split_at_mut(uv_size);

    // Convert each pixel
    for y in 0..h {
        for x in 0..w {
            let rgb_idx = (y * w + x) * 3;
            let r = rgb[rgb_idx] as i32;
            let g = rgb[rgb_idx + 1] as i32;
            let b = rgb[rgb_idx + 2] as i32;

            // BT.601 conversion
            let y_val = ((66 * r + 129 * g + 25 * b + 128) >> 8) + 16;
            y_plane[y * w + x] = y_val.clamp(0, 255) as u8;

            // Subsample U and V (2x2 blocks)
            if y % 2 == 0 && x % 2 == 0 {
                let uv_idx = (y / 2) * (w / 2) + (x / 2);
                let u_val = ((-38 * r - 74 * g + 112 * b + 128) >> 8) + 128;
                let v_val = ((112 * r - 94 * g - 18 * b + 128) >> 8) + 128;
                u_plane[uv_idx] = u_val.clamp(0, 255) as u8;
                v_plane[uv_idx] = v_val.clamp(0, 255) as u8;
            }
        }
    }

    yuv
}

/// WebRTC streaming configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamConfig {
    pub bitrate: u32,      // Target bitrate in bps
    pub max_fps: u32,      // Maximum frames per second
    pub width: u32,        // Stream width
    pub height: u32,       // Stream height
    pub codec: VideoCodec, // Video codec
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            bitrate: 2_000_000, // 2 Mbps
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
    h264_packetizer: Arc<RwLock<Option<H264RTPPacketizer>>>,
    opus_packetizer: Arc<RwLock<Option<OpusRTPPacketizer>>>,
}

/// Encoded frame for WebRTC transmission
#[derive(Debug, Clone)]
pub struct EncodedFrame {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub frame_type: WebRTCFrameType,
    pub width: u32,
    pub height: u32,
}

/// Frame type for WebRTC streaming
#[derive(Debug, Clone)]
pub enum WebRTCFrameType {
    Keyframe, // I-frame
    Delta,    // P-frame
}

#[cfg(feature = "recording")]
/// H.264 encoder for WebRTC streaming
pub struct H264WebRTCEncoder {
    encoder: Encoder,
    width: u32,
    height: u32,
    frame_count: u64,
}

#[cfg(feature = "recording")]
impl H264WebRTCEncoder {
    /// Create a new H.264 encoder for WebRTC
    /// 
    /// Note: The openh264 crate used (v0.9.0) doesn't expose bitrate/fps configuration
    /// in its public API. For production use, consider switching to a more configurable
    /// encoder or use encoder.set_option() if available in newer versions.
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let encoder = Encoder::new()
            .map_err(|e| format!("Failed to create H.264 encoder: {}", e))?;

        Ok(Self {
            encoder,
            width,
            height,
            frame_count: 0,
        })
    }

    /// Encode a frame to H.264 access unit
    pub fn encode_frame(&mut self, frame: &CameraFrame) -> Result<EncodedFrame, String> {
        // Convert RGB to YUV420 if needed
        let yuv_data = if frame.format == "RGB8" {
            rgb_to_yuv420(&frame.data, frame.width, frame.height)
        } else {
            // Assume YUV420
            frame.data.clone()
        };

        let yuv_buffer = YUVBuffer::from_vec(yuv_data, self.width as usize, self.height as usize);

        let bitstream = self.encoder.encode(&yuv_buffer)
            .map_err(|e| format!("H.264 encoding failed: {}", e))?;

        self.frame_count += 1;

        let is_keyframe = matches!(bitstream.frame_type(), FrameType::IDR | FrameType::I);
        let frame_type = if is_keyframe { WebRTCFrameType::Keyframe } else { WebRTCFrameType::Delta };

        Ok(EncodedFrame {
            data: bitstream.to_vec(),
            timestamp: frame.timestamp.timestamp_millis() as u64,
            frame_type,
            width: self.width,
            height: self.height,
        })
    }

    /// Force next frame to be keyframe
    pub fn force_keyframe(&mut self) {
        self.encoder.force_intra_frame();
    }
}

#[cfg(feature = "audio")]
/// Opus encoder for WebRTC streaming
pub struct OpusWebRTCEncoder {
    encoder: *mut OpusEncoder,
    sample_rate: i32,
    channels: i32,
    frame_size: usize, // in samples
}

#[cfg(feature = "audio")]
impl OpusWebRTCEncoder {
    /// Create a new Opus encoder for WebRTC
    pub fn new(sample_rate: i32, channels: i32, bitrate: i32) -> Result<Self, String> {
        let mut error: i32 = 0;
        let encoder = unsafe {
            opus_encoder_create(sample_rate, channels, OPUS_APPLICATION_VOIP as i32, &mut error)
        };

        if error != OPUS_OK as i32 {
            return Err(format!("Failed to create Opus encoder: {}", error));
        }

        unsafe {
            opus_encoder_ctl(encoder, OPUS_SET_BITRATE_REQUEST as i32, bitrate);
        }

        let frame_size = (sample_rate as usize / 50) * channels as usize; // 20ms frames

        Ok(Self {
            encoder,
            sample_rate,
            channels,
            frame_size,
        })
    }

    /// Encode PCM audio to Opus packet
    pub fn encode(&mut self, pcm: &[f32]) -> Result<WebRTCAudioPacket, String> {
        if pcm.len() != self.frame_size {
            return Err(format!("Invalid PCM size: expected {}, got {}", self.frame_size, pcm.len()));
        }

        let mut output = vec![0u8; 4000]; // Max Opus packet size
        let len = unsafe {
            opus_encode_float(self.encoder, pcm.as_ptr(), (self.frame_size / self.channels as usize) as i32, output.as_mut_ptr(), output.len() as i32)
        };

        if len < 0 {
            return Err(format!("Opus encoding failed: {}", len));
        }

        output.truncate(len as usize);

        Ok(WebRTCAudioPacket {
            data: output,
            timestamp: 0, // Will be set by caller
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
}

#[cfg(feature = "audio")]
impl Drop for OpusWebRTCEncoder {
    fn drop(&mut self) {
        unsafe {
            opus_encoder_destroy(self.encoder);
        }
    }
}

/// Encoded audio packet for WebRTC transmission
#[derive(Debug, Clone)]
pub struct WebRTCAudioPacket {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub sample_rate: i32,
    pub channels: i32,
}

/// RTP payload for WebRTC transmission
#[derive(Debug, Clone)]
pub struct RtpPayload {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub sequence_number: u16,
    pub marker: bool,
}

/// H.264 RTP packetizer implementing RFC 6184
pub struct H264RTPPacketizer {
    mtu: usize,
    sequence_number: u16,
}

impl H264RTPPacketizer {
    /// Create a new H.264 RTP packetizer
    pub fn new(mtu: usize) -> Self {
        Self {
            mtu: mtu.min(1200), // Default MTU budget
            sequence_number: 0,
        }
    }

    /// Packetize an Annex B H.264 access unit into RTP payloads
    pub fn packetize(&mut self, access_unit: &[u8], timestamp: u64) -> Result<Vec<RtpPayload>, String> {
        let nal_units = Self::parse_annex_b_nal_units(access_unit)?;
        let mut payloads = Vec::new();

        for nal_unit in nal_units {
            if nal_unit.len() <= self.mtu - 12 { // 12 bytes RTP header
                // Single NAL unit packet
                let mut payload = vec![0u8; 1]; // FU header will be NAL header
                payload[0] = nal_unit[0]; // Copy NAL header
                payload.extend_from_slice(&nal_unit[1..]);

                payloads.push(RtpPayload {
                    data: payload,
                    timestamp,
                    sequence_number: self.sequence_number,
                    marker: true, // Last packet in frame
                });
                self.sequence_number = self.sequence_number.wrapping_add(1);
            } else {
                // Fragment using FU-A
                payloads.extend(self.fragment_nal_unit(nal_unit, timestamp)?);
            }
        }

        Ok(payloads)
    }

    /// Parse Annex B start codes to extract NAL units
    fn parse_annex_b_nal_units(data: &[u8]) -> Result<Vec<&[u8]>, String> {
        let mut nal_units = Vec::new();
        let mut start = 0;

        while start < data.len() {
            // Find next start code
            let start_code_len = if start + 3 < data.len() && &data[start..start + 4] == &[0, 0, 0, 1] {
                4
            } else if start + 2 < data.len() && &data[start..start + 3] == &[0, 0, 1] {
                3
            } else {
                break;
            };

            // Find end of NAL unit (next start code or end of data)
            let mut end = start + start_code_len;
            while end < data.len() {
                if end + 2 < data.len() && &data[end..end + 3] == &[0, 0, 1] {
                    break;
                }
                if end + 3 < data.len() && &data[end..end + 4] == &[0, 0, 0, 1] {
                    break;
                }
                end += 1;
            }

            if end > start + start_code_len {
                nal_units.push(&data[start + start_code_len..end]);
            }

            start = end;
        }

        if nal_units.is_empty() {
            return Err("No valid NAL units found in access unit".to_string());
        }

        Ok(nal_units)
    }

    /// Fragment a large NAL unit using FU-A
    fn fragment_nal_unit(&mut self, nal_unit: &[u8], timestamp: u64) -> Result<Vec<RtpPayload>, String> {
        if nal_unit.is_empty() {
            return Ok(Vec::new());
        }

        let nal_header = nal_unit[0];
        let nal_type = nal_header & 0x1F;
        let fu_header_byte = (nal_header & 0xE0) | 28; // FU-A type

        let payload_size = self.mtu - 12 - 2; // RTP header + FU header
        let mut payloads = Vec::new();
        let mut offset = 1; // Skip NAL header

        while offset < nal_unit.len() {
            let end = (offset + payload_size).min(nal_unit.len());
            let is_last = end == nal_unit.len();

            let mut fu_header = vec![fu_header_byte];
            let fu_indicator = if offset == 1 { 0x80 } else { 0x00 } | // Start bit
                             if is_last { 0x40 } else { 0x00 }; // End bit
            fu_header.push(fu_indicator | nal_type);

            let mut payload = fu_header;
            payload.extend_from_slice(&nal_unit[offset..end]);

            payloads.push(RtpPayload {
                data: payload,
                timestamp,
                sequence_number: self.sequence_number,
                marker: is_last,
            });

            self.sequence_number = self.sequence_number.wrapping_add(1);
            offset = end;
        }

        Ok(payloads)
    }
}

/// Opus RTP packetizer implementing RFC 7587
pub struct OpusRTPPacketizer {
    sequence_number: u16,
    timestamp: u64,
}

impl OpusRTPPacketizer {
    /// Create a new Opus RTP packetizer
    pub fn new() -> Self {
        Self {
            sequence_number: 0,
            timestamp: 0,
        }
    }

    /// Packetize an Opus packet into RTP payload
    pub fn packetize(&mut self, opus_packet: &[u8], samples: u32) -> Result<RtpPayload, String> {
        let payload = RtpPayload {
            data: opus_packet.to_vec(),
            timestamp: self.timestamp,
            sequence_number: self.sequence_number,
            marker: true, // Opus packets are typically single packets
        };

        // Increment sequence number
        self.sequence_number = self.sequence_number.wrapping_add(1);

        // Increment timestamp based on sample count (48kHz clock)
        self.timestamp = self.timestamp.wrapping_add(samples as u64);

        Ok(payload)
    }
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
            h264_packetizer: Arc::new(RwLock::new(None)),
            opus_packetizer: Arc::new(RwLock::new(None)),
        }
    }

    /// Start streaming camera frames
    pub async fn start_streaming(&self, device_id: String) -> Result<(), String> {
        let mut is_streaming = self.is_streaming.write().unwrap();
        if *is_streaming {
            return Err("Stream already active".to_string());
        }

        *is_streaming = true;
        log::info!(
            "Starting WebRTC stream {} for device {}",
            self.stream_id,
            device_id
        );

        // Start frame processing task
        let streamer = self.clone();
        tokio::spawn(async move {
            streamer.stream_processing_loop(device_id).await;
        });

        Ok(())
    }

    /// Stop streaming
    pub async fn stop_streaming(&self) -> Result<(), String> {
        let mut is_streaming = self.is_streaming.write().unwrap();
        if !*is_streaming {
            return Ok(());
        }

        *is_streaming = false;
        log::info!("Stopping WebRTC stream {}", self.stream_id);
        Ok(())
    }

    /// Check if currently streaming
    pub async fn is_streaming(&self) -> bool {
        *self.is_streaming.read().unwrap()
    }

    /// Get current stream configuration
    pub async fn get_config(&self) -> StreamConfig {
        (*self.config.read().unwrap()).clone()
    }

    /// Update stream configuration
    pub async fn update_config(&self, config: StreamConfig) -> Result<(), String> {
        let mut current_config = self.config.write().unwrap();
        *current_config = config;
        log::info!("Updated WebRTC stream configuration for {}", self.stream_id);
        Ok(())
    }

    /// Initialize H.264 RTP packetizer
    pub async fn init_h264_packetizer(&self, mtu: usize) {
        let mut packetizer = self.h264_packetizer.write().unwrap();
        *packetizer = Some(H264RTPPacketizer::new(mtu));
    }

    /// Initialize Opus RTP packetizer
    pub async fn init_opus_packetizer(&self) {
        let mut packetizer = self.opus_packetizer.write().unwrap();
        *packetizer = Some(OpusRTPPacketizer::new());
    }

    /// Subscribe to encoded frames
    pub fn subscribe_frames(&self) -> broadcast::Receiver<EncodedFrame> {
        self.frame_sender.subscribe()
    }

    /// Packetize H.264 frame into RTP payloads
    pub async fn packetize_h264_frame(&self, frame: &EncodedFrame) -> Result<Vec<RtpPayload>, String> {
        let mut packetizer = self.h264_packetizer.write().unwrap();
        if let Some(ref mut p) = *packetizer {
            p.packetize(&frame.data, frame.timestamp)
        } else {
            Err("H.264 packetizer not initialized".to_string())
        }
    }

    /// Packetize Opus packet into RTP payload
    pub async fn packetize_opus_packet(&self, packet: &WebRTCAudioPacket) -> Result<RtpPayload, String> {
        let mut packetizer = self.opus_packetizer.write().unwrap();
        if let Some(ref mut p) = *packetizer {
            // Calculate samples based on packet size and sample rate
            // This is a rough estimate; in practice, this should be provided
            let samples = (packet.sample_rate / 50) as u32; // Assume 20ms frames
            p.packetize(&packet.data, samples)
        } else {
            Err("Opus packetizer not initialized".to_string())
        }
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

        while *self.is_streaming.read().unwrap() {
            // TODO: Integrate with real camera capture from nokhwa
            // For now, simulate frame capture and encoding
            // In production: capture frame from camera, encode to H264, send

            let config = self.get_config().await;
            let frame_type = if frame_counter - last_keyframe >= keyframe_interval {
                last_keyframe = frame_counter;
                WebRTCFrameType::Keyframe
            } else {
                WebRTCFrameType::Delta
            };

            let encoded_frame = EncodedFrame {
                data: self.create_encoded_frame(&config, &frame_type).await,
                timestamp: frame_counter * (1000 / config.max_fps as u64),
                frame_type,
                width: config.width,
                height: config.height,
            };

            // Send frame to subscribers
            if self.frame_sender.send(encoded_frame).is_err() {
                log::warn!("No subscribers for frame on stream {}", device_id);
            }

            frame_counter += 1;

            // Frame rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(
                1000 / config.max_fps as u64,
            ))
            .await;
        }

        log::info!("Stream processing loop ended for device {}", device_id);
    }

    /// Create encoded frame data using real H.264 encoding
    async fn create_encoded_frame(
        &self,
        config: &StreamConfig,
        _frame_type: &WebRTCFrameType,
    ) -> Vec<u8> {
        // Create a dummy YUV420 frame (black) for now - TODO: integrate with real camera capture
        let width = config.width;
        let height = config.height;
        let y_size = (width * height) as usize;
        let uv_size = ((width / 2) * (height / 2)) as usize;
        let mut yuv_data = vec![0u8; y_size + 2 * uv_size];
        // Y plane: 0 (black)
        // U and V: 128 (neutral)

        for i in y_size..y_size + uv_size {
            yuv_data[i] = 128;
        }
        for i in y_size + uv_size..y_size + 2 * uv_size {
            yuv_data[i] = 128;
        }

        // Create YUVBuffer from the data
        let yuv_buffer = YUVBuffer::from_vec(yuv_data, width as usize, height as usize);

        // Create encoder
        let mut encoder = Encoder::new().expect("Failed to create encoder");

        // Encode
        let encoded = encoder.encode(&yuv_buffer).expect("Failed to encode");
        encoded.to_vec()
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
/// NOTE: Image resizing requires additional dependency (image crate resize feature)
/// Current implementation returns original data - implement when real WebRTC integration is added
pub fn prepare_frame_for_webrtc(
    frame: &CameraFrame,
    config: &StreamConfig,
) -> Result<Vec<u8>, String> {
    // Resize frame if needed
    if frame.width != config.width || frame.height != config.height {
        log::debug!(
            "Resizing frame from {}x{} to {}x{}",
            frame.width,
            frame.height,
            config.width,
            config.height
        );

        // NOTE: Implement with image::imageops::resize() when needed
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
        tokio::time::timeout(tokio::time::Duration::from_millis(100), receiver.recv())
            .await
            .expect("Should receive frame")
            .expect("Frame should be valid");

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
