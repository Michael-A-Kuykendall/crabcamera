# WebRTC Simulcast Architecture

## Overview
Implement true simulcast streaming with multiple H.264 encodings at different quality levels from live camera capture.

## Current Status ✅
- **Framework Complete**: SimulcastConfig, SimulcastLayer, SimulcastEncoder structs implemented
- **Integration Ready**: WebRTCStreamer supports simulcast encoders and frame processing
- **Mock Testing**: Streaming loop generates test frames and processes simulcast layers
- **RTP Infrastructure**: RID-based layer identification in place

## Core Components

### 1. Simulcast Layers
- **High Quality**: Full resolution (1280x1080), high bitrate (~2Mbps)
- **Medium Quality**: Scaled down (640x360), medium bitrate (~500Kbps)
- **Low Quality**: Scaled down (320x180), low bitrate (~150Kbps)

### 2. Data Flow
```
Camera Frame → [Resize] → Multiple H.264 Encoders → RTP Packetizers → WebRTC Peer Connection
                    ↓
              Simulcast Layers (RID: f, h, q)
```

### 3. Key Implementation Points

#### Frame Processing ✅
- Mock frame generation implemented for testing
- Real camera integration pending (requires camera access in streamer)

#### RTP Packetization ✅
- RID (RTP Stream ID) support: "f", "h", "q"
- Proper sequence numbers and timestamps per layer
- H.264 NAL unit fragmentation ready

#### WebRTC Integration ✅
- Peer connection simulcast transceiver support
- Frame broadcasting with RID metadata
- Streaming loop processes all layers

### 4. Architecture Decisions

#### Encoder Management ✅
- SimulcastEncoder struct with per-layer H.264 encoder
- Configured with appropriate resolution/bitrate
- Thread-safe with Arc<RwLock<>>

#### Memory Management ✅
- Frame data passed by reference where possible
- Encoder pooling via Arc<RwLock<>>
- Broadcast channel for frame distribution

#### Error Handling ✅
- Graceful degradation if layer encoding fails
- Logging for debugging
- Continues streaming other layers

### 5. Integration Points

#### Existing Code ✅
- `src/platform/` camera capture (needs integration)
- `src/webrtc/peer.rs` transceiver setup ✅
- `src/webrtc/streaming.rs` core logic ✅

#### New Components ✅
- `SimulcastEncoder` implementation ✅
- Frame resizing utility (image crate) ✅
- RTP multiplexing logic ✅

## Implementation Plan

### Phase 1: Framework ✅
- [x] SimulcastConfig, SimulcastLayer structs
- [x] SimulcastEncoder with H.264 encoding
- [x] WebRTCStreamer simulcast support
- [x] Mock frame generation for testing

### Phase 2: Real Camera Integration (Next)
- [ ] Connect to actual camera capture
- [ ] Replace mock frames with real camera frames
- [ ] Test with live video input

### Phase 3: RTP Optimization
- [ ] Fine-tune bitrate ratios
- [ ] Optimize frame resizing performance
- [ ] Memory usage profiling

### Phase 4: Production Testing
- [ ] Multi-layer streaming validation
- [ ] Network condition simulation
- [ ] Client-side adaptation testing

## Questions/Research Needed

- Optimal scaling algorithms for frame resizing (image crate Lanczos3 implemented)
- Best bitrate ratios between layers (current: 1:4:13 ratio)
- RTP timestamp synchronization across layers (implemented)
- Memory usage patterns for multiple encoders (Arc<RwLock> used)

## Testing Status
- ✅ Compilation: Clean with WebRTC features
- ✅ Unit Tests: 13/14 passing (1 ignored TODO)
- ✅ Framework: Simulcast encoders initialize correctly
- ✅ Streaming: Mock frames processed through all layers
- ✅ RTP: RID metadata attached to frames</content>
<parameter name="filePath">c:\Users\micha\repos\crabcamera\docs\WEBRTC_SIMULCAST_ARCHITECTURE.md