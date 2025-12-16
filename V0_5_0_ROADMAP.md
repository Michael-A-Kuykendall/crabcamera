# CrabCamera v0.5.0 Roadmap - Video Recording & Beyond

**Created**: December 14, 2025  
**Target Release**: Q1 2025  
**Focus**: Video Recording as the flagship feature

---

## 🎯 Executive Summary

v0.4.1 delivered bug fixes, performance improvements, and 157 passing tests with real hardware validation. v0.5.0 will add **video recording** - the #1 missing feature compared to competitors like Cap (15.8k stars).

### Competitive Analysis
| Feature | CrabCamera v0.4.1 | Cap | nokhwa |
|---------|-------------------|-----|--------|
| Photo Capture | ✅ | ✅ | ✅ |
| **Video Recording** | ❌ | ✅ | ❌ |
| Quality Validation | ✅ (unique!) | ❌ | ❌ |
| Focus Stacking | ✅ (unique!) | ❌ | ❌ |
| HDR Bracketing | ✅ | ❌ | ❌ |
| Tauri 2.0 Plugin | ✅ | ✅ (uses Tauri) | ❌ |
| Cross-platform | ✅ | ✅ | ✅ |

**Key Insight**: CrabCamera has unique photography features (quality validation, focus stacking). Adding video recording makes us a complete camera solution.

---

## 🏆 Priority 1: Video Recording (MVP)

### Core Requirements
1. **Start/Stop Recording** - Simple API to begin/end video capture
2. **Codec Selection** - H.264 (universal), VP9 (quality), or platform-native
3. **Quality Settings** - Resolution, FPS, bitrate configuration
4. **Output Formats** - MP4 (universal), WebM (web-friendly)
5. **Duration Limits** - Optional max duration, file size limits
6. **Progress Callbacks** - Recording time, file size, frame count

### API Design (Proposed)
```rust
// Start recording
start_video_recording(device_id: String, config: VideoRecordingConfig) -> Result<RecordingHandle>

// Stop recording
stop_video_recording(handle: RecordingHandle) -> Result<VideoFile>

// Check status
get_recording_status(handle: RecordingHandle) -> Result<RecordingStatus>

// Pause/resume (optional)
pause_video_recording(handle: RecordingHandle) -> Result<()>
resume_video_recording(handle: RecordingHandle) -> Result<()>
```

### VideoRecordingConfig Structure
```rust
pub struct VideoRecordingConfig {
    pub output_path: String,
    pub codec: VideoCodec,           // H264, VP9, HEVC
    pub container: VideoContainer,   // MP4, WebM, MKV
    pub resolution: (u32, u32),      // Match camera or downscale
    pub fps: f32,                    // 24, 30, 60
    pub bitrate: u32,                // kbps
    pub max_duration_secs: Option<u32>,
    pub max_file_size_mb: Option<u32>,
    pub audio: Option<AudioConfig>,  // Future: audio capture
}
```

### Architecture Options

#### Option A: FFmpeg via subprocess
**Pros**: Battle-tested, all codecs, simple integration  
**Cons**: External dependency, installation complexity, larger bundle

```
Camera (nokhwa) -> Frames (RGB) -> Pipe to FFmpeg -> Output File
```

#### Option B: Native Rust encoding (gstreamer-rs or ffmpeg-next)
**Pros**: Single binary, better control, no external deps  
**Cons**: More complex, platform build issues

```
Camera (nokhwa) -> Frames (RGB) -> Rust encoder -> Output File
```

#### Option C: Platform-native APIs
**Pros**: Optimal performance, hardware encoding  
**Cons**: Different code per platform, maintenance burden

- **Windows**: Media Foundation (IMFSinkWriter)
- **macOS**: AVFoundation (AVAssetWriter)
- **Linux**: GStreamer or V4L2

### Recommended Approach: Hybrid

1. **Phase 1**: FFmpeg subprocess for MVP (fastest to ship)
2. **Phase 2**: Platform-native for performance (optional optimization)

---

## 🏆 Priority 2: Real-time Streaming with Frame Callbacks

### Current State
- `start_camera_preview()` exists but doesn't stream frames to frontend
- WebRTC module exists but uses mock implementation

### Goal
Enable real-time frame access for:
- Live preview in Tauri frontend
- Frame-by-frame processing
- Custom overlays and filters

### Implementation Options

#### Option A: Event-based frame streaming
```rust
// Emit frames as Tauri events
start_frame_stream(device_id: String, config: StreamConfig) -> Result<()>
// Frontend receives: Event<"camera:frame", { data: base64, timestamp: u64 }>
```

#### Option B: Shared memory buffer
```rust
// Create shared memory region
create_frame_buffer(device_id: String) -> Result<FrameBufferHandle>
// Frontend reads directly from shared memory
```

#### Option C: WebRTC to browser video element
- Already have WebRTC stubs in `src/webrtc/`
- Need real implementation with webrtc-rs or libwebrtc

### Recommended: Start with events, optimize later

---

## 🏆 Priority 3: PTZ Camera Controls

### Current State
- Focus, exposure, white balance all working
- Zoom detection exists but not fully implemented
- Pan/Tilt not implemented

### Target Hardware
- OBSBOT Tiny 4K (user's camera) - has PTZ capabilities
- Logitech PTZ Pro 2
- Other USB PTZ cameras

### API Design
```rust
// PTZ control
set_ptz_position(device_id: String, pan: f32, tilt: f32, zoom: f32) -> Result<()>
get_ptz_position(device_id: String) -> Result<PtzPosition>
reset_ptz_position(device_id: String) -> Result<()>

// Preset positions
save_ptz_preset(device_id: String, name: String) -> Result<()>
load_ptz_preset(device_id: String, name: String) -> Result<()>
list_ptz_presets(device_id: String) -> Result<Vec<String>>
```

---

## 📋 Implementation Phases

### Phase 1: Video Recording MVP (2-3 weeks)
- [ ] Research FFmpeg Rust bindings vs subprocess
- [ ] Design VideoRecordingConfig and RecordingStatus types
- [ ] Implement start_video_recording (FFmpeg subprocess)
- [ ] Implement stop_video_recording
- [ ] Add basic codec support (H.264 in MP4)
- [ ] Add recording duration/size tracking
- [ ] Test with OBSBOT Tiny 4K
- [ ] Add Tauri commands
- [ ] Write tests

### Phase 2: Recording Polish (1-2 weeks)
- [ ] Add VP9/WebM support
- [ ] Add bitrate/quality presets (Low, Medium, High, Ultra)
- [ ] Add pause/resume
- [ ] Add recording events (started, progress, stopped, error)
- [ ] Handle errors gracefully (disk full, codec failure)
- [ ] Cross-platform testing

### Phase 3: Streaming (2 weeks)
- [ ] Implement frame event streaming
- [ ] Create Tauri frontend example
- [ ] Profile performance (target: 30fps with <100ms latency)
- [ ] Add frame decimation option (send every Nth frame)
- [ ] Add resolution downscaling option

### Phase 4: PTZ Controls (1 week)
- [ ] Research OBSBOT control protocol
- [ ] Implement zoom control via MediaFoundation
- [ ] Implement pan/tilt if camera supports
- [ ] Add preset save/load

### Phase 5: Polish & Release (1 week)
- [ ] Update CHANGELOG
- [ ] Update README with video recording docs
- [ ] Create video recording example
- [ ] 200+ tests target
- [ ] Release v0.5.0

---

## 🔬 Technical Research Needed

### Video Encoding Libraries
1. **ffmpeg-next** - Rust bindings to FFmpeg
   - Pros: Full FFmpeg power, all codecs
   - Cons: Build complexity, FFmpeg dependency

2. **gstreamer-rs** - Rust bindings to GStreamer
   - Pros: Pipeline-based, flexible
   - Cons: Complex API, GStreamer dependency

3. **rav1e** - Rust AV1 encoder
   - Pros: Pure Rust, no deps
   - Cons: AV1 only, slower than hardware

4. **x264-rs** - Rust bindings to x264
   - Pros: Fast H.264
   - Cons: x264 dependency

### Platform Video APIs
- **Windows**: Media Foundation IMFSinkWriter
- **macOS**: AVAssetWriter with AVAssetWriterInput
- **Linux**: GStreamer or ffmpeg

### Recommendation
Start with **FFmpeg subprocess** for MVP:
```rust
let ffmpeg = Command::new("ffmpeg")
    .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-s", "1920x1080", "-r", "30"])
    .arg("-i").arg("-")  // stdin
    .args(["-c:v", "libx264", "-preset", "fast", "-crf", "23"])
    .arg(&output_path)
    .stdin(Stdio::piped())
    .spawn()?;
```

---

## 📊 Success Metrics

| Metric | Target | Notes |
|--------|--------|-------|
| Test Count | 200+ | Up from 157 |
| Recording Latency | <500ms start | Time from API call to first frame captured |
| Recording FPS | 30fps sustained | At 1080p |
| File Size | <500MB/min at 1080p30 | H.264 CRF 23 |
| Cross-platform | Windows, macOS, Linux | FFmpeg available on all |

---

## 🚫 Out of Scope for v0.5.0

- ❌ Audio recording (v0.6.0)
- ❌ Video editing/trimming (not our domain)
- ❌ Cloud upload (not our domain)
- ❌ Hardware encoding optimization (v0.6.0)
- ❌ Multi-camera recording (v0.6.0)

---

## 📦 New Dependencies (Tentative)

```toml
# Option A: FFmpeg subprocess (no new deps, just spawn process)

# Option B: FFmpeg bindings
ffmpeg-next = "6.0"  # Full FFmpeg power

# Option C: GStreamer
gstreamer = "0.21"
gstreamer-video = "0.21"
```

---

## 🎬 Example Usage (Target API)

```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Start recording
const handle = await invoke('start_video_recording', {
  deviceId: 'camera-0',
  config: {
    outputPath: './my-video.mp4',
    codec: 'H264',
    container: 'MP4',
    resolution: [1920, 1080],
    fps: 30,
    bitrate: 5000,  // 5 Mbps
    maxDurationSecs: 300  // 5 minutes max
  }
});

// Check status periodically
const status = await invoke('get_recording_status', { handle });
console.log(`Recording: ${status.durationSecs}s, ${status.fileSizeMb}MB`);

// Stop recording
const file = await invoke('stop_video_recording', { handle });
console.log(`Saved to: ${file.path}, Duration: ${file.durationSecs}s`);
```

---

## 📝 Notes from Prior Conversations

### From Competitive Analysis
- Cap (15.8k stars) has video recording - this is why they're bigger
- nokhwa (720 stars) doesn't have recording - opportunity to differentiate
- Video recording is the #1 requested feature for camera libs

### From Control Audit
- Current controls working: auto_focus, focus_distance, auto_exposure, exposure_time, white_balance, brightness, contrast, saturation
- Not yet wired: ISO (type exists), sharpness (type exists), zoom (partial)
- PTZ cameras like OBSBOT would benefit from full pan/tilt/zoom support

### From Hardware Testing
- OBSBOT Tiny 4K: 3840x2160 MJPEG @ 1fps (native), other formats available
- Quality validation working: blur=0.30, exposure=1.00
- Camera warmup optimized to 5 frames

---

*Last Updated: December 14, 2025*
