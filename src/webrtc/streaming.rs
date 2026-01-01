use openh264::encoder::Encoder;
use openh264::formats::YUVBuffer;
#[cfg(feature = "recording")]
use openh264::encoder::FrameType;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use crate::types::CameraFrame;

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
    pub simulcast: Option<SimulcastConfig>, // Optional simulcast configuration
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            bitrate: 2_000_000, // 2 Mbps
            max_fps: 30,
            width: 1280,
            height: 720,
            codec: VideoCodec::H264,
            simulcast: None,
        }
    }
}

/// Simulcast configuration for multiple video layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulcastConfig {
    pub layers: Vec<SimulcastLayer>,
}

impl Default for SimulcastConfig {
    fn default() -> Self {
        Self {
            layers: vec![
                SimulcastLayer {
                    rid: "f".to_string(), // Full resolution
                    width: 1280,
                    height: 720,
                    bitrate: 2_000_000,
                    fps: 30,
                },
                SimulcastLayer {
                    rid: "h".to_string(), // Half resolution
                    width: 640,
                    height: 360,
                    bitrate: 500_000,
                    fps: 15,
                },
                SimulcastLayer {
                    rid: "q".to_string(), // Quarter resolution
                    width: 320,
                    height: 180,
                    bitrate: 150_000,
                    fps: 10,
                },
            ],
        }
    }
}

/// Individual simulcast layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulcastLayer {
    pub rid: String,     // RTP Stream ID (f, h, q, etc.)
    pub width: u32,      // Layer width
    pub height: u32,     // Layer height
    pub bitrate: u32,    // Layer bitrate in bps
    pub fps: u32,        // Layer frame rate
}

/// Simulcast encoder for a specific layer
#[derive(Clone)]
pub struct SimulcastEncoder {
    pub rid: String,
    pub encoder: Arc<RwLock<H264WebRTCEncoder>>,
    pub packetizer: Arc<RwLock<H264RTPPacketizer>>,
    pub config: SimulcastLayer,
}

/// Supported video codecs for WebRTC streaming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    VP8,
    VP9,
    AV1,
}

/// Streaming mode for WebRTC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamMode {
    RealCamera,     // Use physical camera
    SyntheticTest,  // Generate synthetic frames for testing
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
    device_id: Arc<RwLock<String>>,
    rtp_sender: Arc<RwLock<Option<tokio::sync::mpsc::UnboundedSender<RtpPayload>>>>,
    paused: Arc<RwLock<bool>>,
    failure_count: Arc<RwLock<u32>>,
    max_failures: u32,
    mode: Arc<RwLock<StreamMode>>,
    camera_status: Arc<RwLock<CameraStatus>>,
}

/// Camera availability status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CameraStatus {
    Available,
    Unavailable(String), // Reason for unavailability
    Failed(String),      // Runtime failure reason
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

        for (idx, nal_unit) in nal_units.iter().enumerate() {
            let is_last_nal = idx + 1 == nal_units.len();

            if nal_unit.len() <= self.mtu - 12 { // 12 bytes RTP header
                // Single NAL unit packet
                let mut payload = vec![0u8; 1]; // FU header will be NAL header
                payload[0] = nal_unit[0]; // Copy NAL header
                payload.extend_from_slice(&nal_unit[1..]);

                payloads.push(RtpPayload {
                    data: payload,
                    timestamp,
                    sequence_number: self.sequence_number,
                    marker: is_last_nal,
                });
                self.sequence_number = self.sequence_number.wrapping_add(1);
            } else {
                // Fragment using FU-A
                payloads.extend(self.fragment_nal_unit(nal_unit, timestamp, is_last_nal)?);
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
    fn fragment_nal_unit(
        &mut self,
        nal_unit: &[u8],
        timestamp: u64,
        is_last_nal_in_access_unit: bool,
    ) -> Result<Vec<RtpPayload>, String> {
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
                marker: is_last && is_last_nal_in_access_unit,
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
            device_id: Arc::new(RwLock::new(String::new())),
            rtp_sender: Arc::new(RwLock::new(None)),
            paused: Arc::new(RwLock::new(false)),
            failure_count: Arc::new(RwLock::new(0)),
            max_failures: 10, // Back to reasonable limit
            mode: Arc::new(RwLock::new(StreamMode::RealCamera)),
            camera_status: Arc::new(RwLock::new(CameraStatus::Available)),
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

        // Store device ID for the streaming loop
        {
            let mut stored_device_id = self.device_id.write().unwrap();
            *stored_device_id = device_id.clone();
        }

        // Start frame processing task
        let streamer = self.clone();
        tokio::spawn(async move {
            log::info!("Spawned streaming task for device {}", device_id);
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

        // Clear device ID
        {
            let mut stored_device_id = self.device_id.write().unwrap();
            *stored_device_id = String::new();
        }

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

    /// Set RTP sender for forwarding packets to peer connection
    pub async fn set_rtp_sender(&self, sender: tokio::sync::mpsc::UnboundedSender<RtpPayload>) {
        let mut rtp_sender = self.rtp_sender.write().unwrap();
        *rtp_sender = Some(sender);
    }

    /// Clear RTP sender
    pub async fn clear_rtp_sender(&self) {
        let mut rtp_sender = self.rtp_sender.write().unwrap();
        *rtp_sender = None;
    }

    /// Pause streaming (stop sending RTP packets)
    pub async fn pause_stream(&self) {
        let mut paused = self.paused.write().unwrap();
        *paused = true;
        log::info!("Stream {} paused", self.stream_id);
    }

    /// Resume streaming
    pub async fn resume_stream(&self) {
        let mut paused = self.paused.write().unwrap();
        *paused = false;
        log::info!("Stream {} resumed", self.stream_id);
    }

    /// Set streaming mode
    pub async fn set_mode(&self, mode: StreamMode) {
        let mut current_mode = self.mode.write().unwrap();
        *current_mode = mode.clone();
        log::info!("Stream {} mode set to {:?}", self.stream_id, mode);
    }

    /// Get current streaming mode
    pub async fn get_mode(&self) -> StreamMode {
        (*self.mode.read().unwrap()).clone()
    }

    /// Get camera status
    pub async fn get_camera_status(&self) -> CameraStatus {
        (*self.camera_status.read().unwrap()).clone()
    }

    /// Set target bitrate
    pub async fn set_bitrate(&self, bitrate: u32) {
        let mut config = self.config.write().unwrap();
        config.bitrate = bitrate;
        log::info!("Stream {} bitrate set to {} bps", self.stream_id, bitrate);
    }

    /// Handle streaming failure
    async fn handle_failure(&self) {
        let mut count = self.failure_count.write().unwrap();
        *count += 1;
        if *count > self.max_failures {
            log::error!("Too many failures ({}), stopping stream {}", *count, self.stream_id);
            let mut streaming = self.is_streaming.write().unwrap();
            *streaming = false;
        } else {
            log::warn!("Stream failure {} for {}", *count, self.stream_id);
        }
    }

    /// Reset failure count
    async fn reset_failures(&self) {
        let mut count = self.failure_count.write().unwrap();
        *count = 0;
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
        let mode = self.get_mode().await;
        let camera_status = self.get_camera_status().await;

        StreamStats {
            stream_id: self.stream_id.clone(),
            is_active,
            target_bitrate: config.bitrate,
            current_fps: if is_active { config.max_fps } else { 0 },
            resolution: (config.width, config.height),
            codec: config.codec,
            subscribers: self.frame_sender.receiver_count(),
            mode,
            camera_status,
        }
    }

    /// Process camera frames for WebRTC streaming
    async fn stream_processing_loop(&self, device_id: String) {
        log::info!("Starting stream processing loop for device {}", device_id);

        let mode = self.get_mode().await;
        let mut camera: Option<crate::platform::PlatformCamera> = None;
        let mut camera_available = false;

        // Initialize camera based on mode
        match mode {
            StreamMode::RealCamera => {
                let config = self.get_config().await;
                let camera_params = crate::types::CameraInitParams {
                    device_id: device_id.clone(),
                    format: crate::types::CameraFormat {
                        width: config.width,
                        height: config.height,
                        fps: config.max_fps as f32,
                        format_type: "MJPEG".to_string(),
                    },
                    controls: crate::types::CameraControls::default(),
                };

                match crate::platform::PlatformCamera::new(camera_params) {
                    Ok(cam) => {
                        camera = Some(cam);
                        if let Some(ref mut camera) = camera {
                            if let Err(e) = camera.start_stream() {
                                log::warn!("Failed to start camera stream: {:?}", e);
                                let mut status = self.camera_status.write().unwrap();
                                *status = CameraStatus::Failed(format!("Start failed: {}", e));
                            } else {
                                camera_available = true;
                                let mut status = self.camera_status.write().unwrap();
                                *status = CameraStatus::Available;
                                log::info!("Camera stream started successfully");
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to initialize camera: {:?}", e);
                        let mut status = self.camera_status.write().unwrap();
                        *status = CameraStatus::Unavailable(format!("Init failed: {}", e));
                    }
                }
            }
            StreamMode::SyntheticTest => {
                log::info!("Using synthetic test frames for device {}", device_id);
                let mut status = self.camera_status.write().unwrap();
                *status = CameraStatus::Unavailable("Synthetic test mode".to_string());
            }
        }

        let mut frame_counter = 0u64;
        let mut last_keyframe = 0u64;
        let keyframe_interval = 30; // Keyframe every 30 frames

        while *self.is_streaming.read().unwrap() {
            // Capture frame based on mode and camera availability
            let camera_frame = match mode {
                StreamMode::RealCamera => {
                    if camera_available {
                        if let Some(ref mut camera) = camera {
                            match camera.capture_frame() {
                                Ok(frame) => Some(frame),
                                Err(e) => {
                                    log::warn!("Camera capture failed: {:?}", e);
                                    let mut status = self.camera_status.write().unwrap();
                                    *status = CameraStatus::Failed(format!("Capture failed: {}", e));
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                StreamMode::SyntheticTest => None, // Always use synthetic
            };

            let camera_frame = match camera_frame {
                Some(frame) => frame,
                None => self.generate_synthetic_frame(frame_counter),
            };

            let frame_type = if frame_counter - last_keyframe >= keyframe_interval {
                last_keyframe = frame_counter;
                WebRTCFrameType::Keyframe
            } else {
                WebRTCFrameType::Delta
            };

// Encode the captured frame
                let encoded_frame = match self.encode_camera_frame(&camera_frame, &frame_type).await {
                    Ok(encoded) => encoded,
                    Err(e) => {
                        log::error!("Failed to encode frame: {:?}", e);
                    self.handle_failure().await;
                    continue;
                }
            };

            // Send frame to subscribers
            if self.frame_sender.send(encoded_frame.clone()).is_err() {
                log::warn!("No subscribers for frame on stream {}", device_id);
            }

            // Packetize and send RTP packets if sender is set and not paused
            let is_paused = *self.paused.read().unwrap();
            if !is_paused {
                let rtp_sender_opt = {
                    let rtp_sender_guard = self.rtp_sender.read().unwrap();
                    rtp_sender_guard.clone()
                };
                
                if let Some(rtp_sender) = rtp_sender_opt {
                    let rtp_packets = match self.packetize_h264_frame(&encoded_frame).await {
                        Ok(packets) => packets,
                        Err(e) => {
                            log::error!("Failed to packetize frame: {:?}", e);
                            self.handle_failure().await;
                            continue;
                        }
                    };

                    for packet in rtp_packets {
                        if rtp_sender.send(packet).is_err() {
                            log::warn!("Failed to send RTP packet - peer may be disconnected");
                            self.handle_failure().await;
                            break;
                        }
                    }
                }
            }

            // Reset failures on successful frame
            self.reset_failures().await;

            frame_counter += 1;

            // Frame rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(
                1000 / self.get_config().await.max_fps as u64,
            ))
            .await;
        }

        // Stop camera stream if it was started
        if camera_available {
            if let Some(ref mut camera) = camera {
                if let Err(e) = camera.stop_stream() {
                    log::error!("Failed to stop camera stream: {:?}", e);
                }
            }
        }

        log::info!("Stream processing loop ended for device {}", device_id);
    }

    /// Encode a captured camera frame for WebRTC
    async fn encode_camera_frame(
        &self,
        frame: &crate::types::CameraFrame,
        frame_type: &WebRTCFrameType,
    ) -> Result<EncodedFrame, String> {
        // Create encoder instance (in practice, you'd want to cache this)
        let mut encoder = H264WebRTCEncoder::new(frame.width, frame.height)?;

        // Force keyframe if needed
        if matches!(frame_type, WebRTCFrameType::Keyframe) {
            encoder.force_keyframe();
        }

        // Encode the frame
        let encoded = encoder.encode_frame(frame)?;

        Ok(encoded)
    }

    /// Generate a synthetic test frame for fallback when camera is unavailable
    fn generate_synthetic_frame(&self, frame_counter: u64) -> crate::types::CameraFrame {
        let config = self.config.read().unwrap();
        let width = config.width;
        let height = config.height;
        
        // Generate a simple pattern: alternating black/white frames
        let pattern = (frame_counter / 30) % 2; // Change every 30 frames
        let pixel_value = if pattern == 0 { 0u8 } else { 255u8 };
        
        let mut data = vec![pixel_value; (width * height * 3) as usize];
        
        // Add some variation based on frame counter
        for i in 0..data.len() {
            data[i] = data[i].saturating_add((frame_counter as u8).wrapping_mul(5));
        }

        let size_bytes = data.len();

        crate::types::CameraFrame {
            id: uuid::Uuid::new_v4().to_string(),
            data,
            width,
            height,
            format: "RGB8".to_string(),
            timestamp: chrono::Utc::now(),
            device_id: "synthetic".to_string(),
            size_bytes,
            metadata: crate::types::FrameMetadata::default(),
        }
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
    pub mode: StreamMode,
    pub camera_status: CameraStatus,
}

/// Convert camera frame to WebRTC-compatible format
/// Resizes frame to match stream configuration using Lanczos3 algorithm
pub fn prepare_frame_for_webrtc(
    frame: &CameraFrame,
    config: &StreamConfig,
) -> Result<Vec<u8>, String> {
    // If dimensions match, return original data
    if frame.width == config.width && frame.height == config.height {
        return Ok(frame.data.clone());
    }

    log::debug!(
        "Resizing frame from {}x{} to {}x{}",
        frame.width,
        frame.height,
        config.width,
        config.height
    );

    // Assume RGB8 format for resizing (most common)
    // Create image buffer from frame data
    let img = match image::RgbImage::from_raw(frame.width, frame.height, frame.data.clone()) {
        Some(img) => img,
        None => {
            return Err(format!(
                "Failed to create image buffer from frame data (expected {} bytes for {}x{} RGB, got {})",
                frame.width as usize * frame.height as usize * 3,
                frame.width,
                frame.height,
                frame.data.len()
            ));
        }
    };

    // Resize using Lanczos3 algorithm for quality
    let resized = image::imageops::resize(
        &img,
        config.width,
        config.height,
        image::imageops::FilterType::Lanczos3,
    );

    // Convert back to RGB bytes
    Ok(resized.into_raw())
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
            simulcast: None,
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
