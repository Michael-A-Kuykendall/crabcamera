# 🦀 CrabCamera: The Desktop Camera & Audio Plugin for Tauri 📷🎙️

```
     __________________________
    < Hello fellow Rustaceans! >
     --------------------------
            \
             \
                _~^~^~_
            \) /  o o  \ (/
              '_   -   _'
              / '-----' \
```

[![Crates.io](https://img.shields.io/crates/v/crabcamera.svg)](https://crates.io/crates/crabcamera)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![Tests](https://img.shields.io/badge/tests-157+-brightgreen.svg)](https://github.com/Michael-A-Kuykendall/crabcamera/releases)
[![Sponsor](https://img.shields.io/badge/❤️-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

**🦀 CrabCamera will be free forever. 🦀** No asterisks. No "free for now." No pivot to paid.

## 🦀 What is CrabCamera?

**🦀 CrabCamera is the **first production-ready desktop camera + audio plugin** for Tauri applications. It provides unified camera and audio access across Windows, macOS, and Linux with professional controls, synchronized A/V recording, and complete headless operation. It's designed to be the **invisible infrastructure** that makes desktop media apps just work.

| Feature | CrabCamera | Web APIs | Other Plugins |
|---------|------------|----------|---------------|
| **Desktop Native** | Windows/macOS/Linux 🏆 | Limited browser | Mobile-only |
| **Hardware Access** | Direct camera + audio 🏆 | Browser restricted | Basic video only |
| **Audio Recording** | Opus/AAC + sync 🏆 | Unreliable | N/A |
| **A/V Synchronization** | PTS-based sync 🏆 | Async/unreliable | N/A |
| **Professional Controls** | Auto-focus, exposure 🏆 | Limited | Basic |
| **Cross-Platform** | Unified API 🏆 | Platform dependent | Single platform |
| **Production Ready** | **115+ tests, 80% coverage** 🏆 | No guarantees | Proof-of-concept |
| **Memory Safety** | Zero unsafe code 🏆 | N/A | Manual management |

## 🎯 Perfect for Desktop Applications 🦀

- **Media Creation**: Screen recorders, podcast editors, video production tools
- **Photography**: Photo booth apps, image editors, content creation tools with audio
- **Security**: Surveillance systems with audio monitoring, access control, live recording
- **Medical**: Imaging interfaces, patient documentation, diagnostic video tools
- **Industrial**: Quality control with audio logging, inspection systems, documentation
- **Education**: Interactive learning tools with recording, virtual labs, presentation software
- **Communication**: Video chat apps with perfect A/V sync, streaming tools, conference recording
- **Entertainment**: Game streaming, Twitch/YouTube capture with sync'd audio

**BONUS:** Professional camera controls with platform-optimized settings for maximum image quality.

## 🚀 Headless Operation (NEW in v0.6.0!) 🦀

CrabCamera v0.6.0 introduces **complete headless operation** - run camera and audio recording without any GUI or Tauri app. Perfect for:

- **Server-side recording** in production environments
- **Automated testing** and CI/CD pipelines
- **Scheduled captures** and monitoring systems
- **API-first integrations** for custom applications

```bash
# Install the CLI tool
cargo install crabcamera-cli

# List all cameras and audio devices
crabcamera-cli list-devices

# Record 30 seconds of video + audio
crabcamera-cli record --duration 30 --output recording.mp4 --camera 0 --audio default
```

## 🔗 Powered by Muxide 🦀

CrabCamera uses **[Muxide](https://github.com/Michael-A-Kuykendall/muxide)** - the pure-Rust MP4 muxer that ensures your recordings are standards-compliant and immediately playable. No FFmpeg, no external dependencies, just reliable MP4 output.

> **Check out Muxide** if you need MP4 muxing in your Rust project! It handles H.264, HEVC, AV1, AAC, and Opus with perfect timing.

## 🦀 Quick Start (30 seconds) 📷🎙️

### Installation

```toml
[dependencies]
crabcamera = { version = "0.6", features = ["recording", "audio", "headless"] }
tauri = { version = "2.0", features = ["protocol-asset"] }
```

### Tauri Integration

```rust
// src-tauri/src/main.rs
use crabcamera;

fn main() {
    tauri::Builder::default()
        .plugin(crabcamera::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

```json
// tauri.conf.json
{
  "plugins": {
    "crabcamera": {
      "audio": true
    }
  }
}
```

### Frontend: Capture a Photo

```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Initialize camera system
await invoke('initialize_camera_system');

// Get available cameras
const cameras = await invoke('get_available_cameras');
const format = await invoke('get_recommended_format');

// Capture a photo
const photo = await invoke('capture_single_photo', {
  deviceId: cameras[0].id,
  format: format
});
```

### Frontend: Record Video + Audio

```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Initialize both camera and audio
await invoke('initialize_camera_system');
const devices = await invoke('list_audio_devices');
const audioDevice = devices.find(d => d.is_default);

// Start recording with A/V sync
await invoke('start_recording', {
  outputPath: 'recording.mp4',
  videoConfig: {
    deviceId: cameras[0].id,
    codec: 'h264',
    width: 1920,
    height: 1080,
    fps: 30.0
  },
  audioConfig: {
    deviceId: audioDevice.id,
    codec: 'opus',           // or 'aac'
    sampleRate: 48000,
    channels: 2
  }
});

// Recording is automatically synchronized (no manual sync needed!)
await invoke('stop_recording');
console.log('✅ Recording saved with perfect A/V sync');
```

## 📦 Professional Media Features 🦀

### 🎥 Video Capture
- **Device Enumeration**: Automatic discovery of all connected cameras
- **Format Negotiation**: Resolution, FPS, and color format selection
- **Professional Settings**: Auto-focus, auto-exposure, white balance
- **Multi-camera Support**: Switch between multiple cameras seamlessly
- **H.264 Encoding**: Industry-standard video codec

### 🎙️ Audio Recording (Enhanced in v0.6.0!)
- **Audio Device Enumeration**: Discover all audio input devices with capabilities
- **Opus Codec**: State-of-the-art compression (40-256 kbps, adaptive bitrate)
- **AAC Support**: Alternative codec for compatibility
- **Multi-Channel**: Mono, stereo, and future multi-channel support
- **Sample Rate Control**: 8kHz-48kHz configurable capture

### 🔄 Audio/Video Synchronization
- **PTS Clock**: Shared monotonic timebase for all timestamps
- **Bounded Drift**: ±40ms max sync error (proven in tests)
- **Automatic Interleaving**: No manual timing configuration needed
- **Keyframe Alignment**: Proper sample-to-frame mapping
- **Muxide Integration**: Custom MP4 muxer for precise timing

### 🖥️ Cross-Platform Native 🦀
- **Windows**: DirectShow/MediaFoundation with advanced camera controls
- **macOS**: AVFoundation for both capture and controls
- **Linux**: V4L2/ALSA with comprehensive device support
- **Unified API**: Same code works across all platforms
- **Professional Controls**: Focus, exposure, white balance on all platforms

### ⚡ Performance & Memory 🦀
- **Zero-Copy Operations**: Minimal memory allocations where possible
- **Async/Await**: Non-blocking operations throughout
- **Resource Management**: Automatic cleanup and device release
- **Memory Safety**: Built with Rust's memory safety guarantees
- **Thread Safety**: Concurrent access with proper synchronization

## 🔧 Available Commands 🦀

### Initialization & Discovery
```rust
// Initialize the camera system
initialize_camera_system() -> Result<String>

// Get all available cameras with capabilities
get_available_cameras() -> Result<Vec<CameraDeviceInfo>>

// Get platform-specific information
get_platform_info() -> Result<PlatformInfo>

// Test camera system functionality
test_camera_system() -> Result<SystemTestResult>
```

### Audio Devices (Enhanced in v0.6.0!)
```rust
// Enumerate all audio input devices
list_audio_devices() -> Result<Vec<AudioDeviceInfo>>

// Get info about specific audio device
get_audio_device_info(device_id: String) -> Result<AudioDeviceInfo>

// Audio device includes:
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub sample_rate: u32,        // 48000 Hz typical
    pub channels: u16,            // 1 (mono) or 2 (stereo)
    pub is_default: bool,
}
```

### Camera Operations
```rust
// Check if specific camera is available
check_camera_availability(device_id: String) -> Result<bool>

// Get supported formats for a camera
get_camera_formats(device_id: String) -> Result<Vec<CameraFormat>>

// Get recommended settings for quality photography
get_recommended_format() -> Result<CameraFormat>
get_optimal_settings() -> Result<CameraInitParams>
```

### Recording (Enhanced Audio Support in v0.6.0!)
```rust
// Start recording with video + optional audio
start_recording(RecordingConfig) -> Result<RecordingId>

// Recording config includes audio:
pub struct RecordingConfig {
    pub output_path: String,
    pub video_config: VideoConfig,
    pub audio_config: Option<AudioConfig>,  // Enhanced!
}

pub struct AudioConfig {
    pub device_id: String,
    pub codec: AudioCodec,  // Opus or AAC
    pub sample_rate: u32,
    pub channels: u16,
    pub bitrate: u32,       // bits per second
}

// Stop and finalize recording (auto A/V sync!)
stop_recording() -> Result<RecordingStatus>

// Get recording status with sync info
get_recording_status() -> Result<RecordingStatus>
```

### Capture & Streaming
```rust
// Single photo capture
capture_single_photo(device_id: String, format: CameraFormat) -> Result<CameraFrame>

// Photo sequence for burst mode
capture_photo_sequence(params: SequenceParams) -> Result<Vec<CameraFrame>>

// Real-time streaming
start_camera_preview(device_id: String) -> Result<()>
stop_camera_preview() -> Result<()>

// Save frames to disk
save_frame_to_disk(frame: CameraFrame, path: String) -> Result<()>
```

### Professional Camera Controls
```rust
// Apply camera controls (focus, exposure, white balance, etc.)
apply_camera_controls(device_id: String, controls: CameraControls) -> Result<()>

// Get current camera control values
get_camera_controls(device_id: String) -> Result<CameraControls>

// Test what controls are supported by camera
test_camera_capabilities(device_id: String) -> Result<CameraCapabilities>

// Get performance metrics
get_camera_performance(device_id: String) -> Result<CameraPerformanceMetrics>
```

### Permissions & Security
```rust
// Handle camera permissions properly
request_camera_permission() -> Result<bool>
check_camera_permission_status() -> Result<PermissionStatus>
```

## 🦀 Why CrabCamera Will Always Be Free 📷

I built CrabCamera because desktop applications deserve native camera access without the limitations of web APIs or mobile-only plugins.

**This is my commitment**: CrabCamera stays MIT licensed, forever. If you want to support development, [sponsor it](https://github.com/sponsors/Michael-A-Kuykendall). If you don't, just build something incredible with it.

> 🦀 CrabCamera saves developers weeks of cross-platform camera integration. If it's useful, consider sponsoring for $5/month — less than a coffee, infinitely more valuable than web API limitations. 🦀

## 📊 Performance Comparison 🦀

| Metric | CrabCamera | Web APIs | Mobile Plugins |
|--------|------------|----------|----------------|
| **Desktop Support** | **Full native** | Browser dependent | None |
| **Video Capture** | **Direct hardware** | getUserMedia limited | N/A |
| **Audio Capture** | **Direct hardware + sync** | Unreliable | N/A |
| **A/V Synchronization** | **PTS-based (±40ms)** | Async/broken | N/A |
| **Image Quality** | **Professional controls** | Basic settings | Basic |
| **Cross-Platform** | **Windows/macOS/Linux** | Browser variation | iOS/Android only |
| **Test Coverage** | **115 unit tests + 80%** | None | None |
| **Performance** | **Native speed** | Browser overhead | N/A |
| **Reliability** | **Production proven** | No guarantees | Varies |

### Benchmark Results (v0.5.0)
```
Video Encoding (H.264):
  1920x1080 @ 30fps: ~45ms per frame (native speed)
  
Audio Encoding (Opus):
  48kHz stereo: Real-time (no buffering needed)
  
A/V Synchronization:
  Drift over 60-minute recording: ±35ms (within 40ms guarantee)
  
Memory (per recording):
  1080p30 + 48kHz stereo: ~50MB buffer
```

## 🏗️ Technical Architecture 🦀

### Complete Media Pipeline Architecture
```
┌──────────────────────────────────────────────────────────────┐
│                    CrabCamera v0.5.0                          │
├──────────────────────────────────────────────────────────────┤
│                                                                │
│  VIDEO                            AUDIO                       │
│  ┌─────────────────┐             ┌──────────────────┐        │
│  │ Camera Capture  │             │ Microphone       │        │
│  │ (nokhwa/native) │             │ Capture (CPAL)   │        │
│  └────────┬────────┘             └────────┬─────────┘        │
│           │                                │                  │
│           ▼                                ▼                  │
│  ┌─────────────────┐             ┌──────────────────┐        │
│  │ H.264 Encoder   │             │ Opus Encoder     │        │
│  │ (openh264)      │             │ (libopus_sys)    │        │
│  └────────┬────────┘             └────────┬─────────┘        │
│           │                                │                  │
│           └────────────┬────────────────────┘                 │
│                        │                                      │
│                        ▼                                      │
│         ┌──────────────────────────┐                          │
│         │   PTS Clock              │                          │
│         │ (Shared Timebase)        │                          │
│         └──────────────┬───────────┘                          │
│                        │                                      │
│                        ▼                                      │
│         ┌──────────────────────────┐                          │
│         │ Muxide MP4 Muxer         │                          │
│         │ (A/V Interleaving)       │                          │
│         └──────────────┬───────────┘                          │
│                        │                                      │
│                        ▼                                      │
│              output.mp4 (PERFECT SYNC!)                       │
│                                                                │
└──────────────────────────────────────────────────────────────┘
```

### Hybrid Capture + Controls Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Capture       │    │ Platform Controls│    │  CrabCamera     │
│   (Best Source) │    │ (Advanced)       │    │  (Unified API)  │
├─────────────────┤    ├──────────────────┤    ├─────────────────┤
│ • nokhwa (video)│    │ • Focus control  │    │ • Generic types │
│ • CPAL (audio)  │    │ • Exposure       │    │ • Error handling│
│ • Resolution    │    │ • White balance  │    │ • Cross-platform│
│ • Format        │    │ • Brightness     │    │ • Thread safety │
│ • Start/Stop    │    │ • Saturation     │    │ • Async/await   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Platform-Specific Implementations
- **Windows**: nokhwa capture + MediaFoundation controls | WASAPI audio
- **macOS**: AVFoundation for both capture and controls | AVFoundation audio
- **Linux**: nokhwa capture + V4L2 controls | ALSA audio
- **Unified API**: Same interface across all platforms

### Key Technologies
- **Rust + Tokio**: Memory-safe, async performance
- **Tauri 2.0 Plugin**: Modern plugin architecture  
- **Platform Backends**: MediaFoundation, AVFoundation, V4L2, WASAPI, ALSA
- **Audio Codecs**: Opus (libopus_sys), AAC support
- **Video Codec**: H.264 (openh264 v0.9)
- **Muxing**: Custom Muxide library (Rust-native MP4 writer)
- **COM Interface Management**: Thread-safe Windows interfaces
- **Zero unsafe code**: Memory safety guaranteed (except platform bindings)

## 📚 API Reference 🦀

### Core Types
```rust
pub struct CameraDeviceInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_available: bool,
    pub supports_formats: Vec<CameraFormat>,
}

pub struct CameraFormat {
    pub width: u32,
    pub height: u32,
    pub fps: f32,
    pub format_type: String, // "RGB8", "JPEG", etc.
}

pub struct CameraFrame {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub timestamp: DateTime<Utc>,
}

pub struct CameraControls {
    pub auto_focus: Option<bool>,
    pub focus_distance: Option<f32>,     // 0.0 = infinity, 1.0 = closest
    pub auto_exposure: Option<bool>,
    pub exposure_time: Option<f32>,      // seconds
    pub white_balance: Option<WhiteBalance>,
    pub brightness: Option<f32>,         // -1.0 to 1.0
    pub contrast: Option<f32>,           // -1.0 to 1.0
    pub saturation: Option<f32>,         // -1.0 to 1.0
}

pub struct CameraCapabilities {
    pub supports_auto_focus: bool,
    pub supports_manual_focus: bool,
    pub supports_auto_exposure: bool,
    pub supports_manual_exposure: bool,
    pub supports_white_balance: bool,
    pub focus_range: Option<(f32, f32)>,
    pub exposure_range: Option<(f32, f32)>,
}
```

### Platform Detection
```rust
pub enum Platform {
    Windows,
    MacOS,
    Linux,
    Unknown,
}

// Automatic platform detection
let platform = Platform::current();
```

## 🦀 Community & Support 📷

- **🐛 Bug Reports**: [GitHub Issues](https://github.com/Michael-A-Kuykendall/crabcamera/issues)
- **💬 Discussions**: [GitHub Discussions](https://github.com/Michael-A-Kuykendall/crabcamera/discussions)
- **📖 Documentation**: [docs.rs/crabcamera](https://docs.rs/crabcamera)
- **💝 Sponsorship**: [GitHub Sponsors](https://github.com/sponsors/Michael-A-Kuykendall)

### Governance

CrabCamera is **open source, not open contribution**. The code is freely available under the MIT license, but pull requests are not accepted by default. See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

This model ensures consistent quality across all platforms and clear project direction.

### Sponsors

See our amazing [sponsors](SPONSORS.md) who make 🦀 CrabCamera possible! 🙏

**Sponsorship Tiers:**
- **$5/month**: Coffee tier - My eternal gratitude + sponsor badge
- **$25/month**: Developer supporter - Priority support + name in SPONSORS.md  
- **$100/month**: Corporate backer - Logo on README + monthly office hours
- **$500/month**: Enterprise partner - Direct support + feature requests

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## 🚀 Production Usage 🦀

**✅ Ready for production (v0.5.0):**
- Memory-safe Rust implementation
- **115+ comprehensive tests passing** (80%+ code coverage)
- Zero unsafe code in public API
- Comprehensive error handling with graceful degradation
- Async/await throughout for non-blocking operations
- Cross-platform compatibility verified (Windows/macOS/Linux)
- Real hardware validation (OBSBOT Tiny 4K + USB microphones)
- Audio/video sync tested (±40ms max drift guarantee)
- Security audits passing (openh264 v0.9, no vulnerabilities)

**✅ Use cases in production:**
- Desktop photography and image editing applications
- Security and surveillance systems with audio logging
- Medical imaging and patient documentation systems
- Industrial inspection tools with audio annotation
- Educational software platforms with screen recording
- Professional communication and streaming applications
- Podcast and content creation studios
- Conference recording and transcription tools

## 💡 Examples & Integration 🦀

### Photo Booth Application
```javascript
// Simple photo booth with camera selection
const cameras = await invoke('get_available_cameras');
const selectedCamera = cameras[0];
const format = await invoke('get_recommended_format');

// Take photo when user clicks
document.getElementById('capture').onclick = async () => {
    const photo = await invoke('capture_single_photo', {
        deviceId: selectedCamera.id,
        format: format
    });
    // Display photo in UI
    displayPhoto(photo);
};
```

### Professional Video Recorder
```javascript
// Video + audio recording with sync
const cameras = await invoke('get_available_cameras');
const audioDevices = await invoke('list_audio_devices');
const defaultAudio = audioDevices.find(d => d.is_default);

// Start recording with A/V sync
await invoke('start_recording', {
    outputPath: 'recording.mp4',
    videoConfig: {
        deviceId: cameras[0].id,
        codec: 'h264',
        width: 1920,
        height: 1080,
        fps: 30.0
    },
    audioConfig: {
        deviceId: defaultAudio.id,
        codec: 'opus',
        sampleRate: 48000,
        channels: 2
    }
});

// No sync configuration needed - automatic!
setTimeout(async () => {
    await invoke('stop_recording');
    console.log('✅ Recording with perfect A/V sync saved');
}, 30000); // 30 second recording
```

### Multi-Camera Security System
```javascript
// Monitor multiple cameras
const cameras = await invoke('get_available_cameras');
for (const camera of cameras) {
    await invoke('start_camera_preview', { deviceId: camera.id });
    // Set up streaming handlers for each camera
    setupCameraStream(camera);
}
```

### Podcast Studio
```javascript
// Record podcast with high-quality audio + optional video
const audioDevices = await invoke('list_audio_devices');

// Find a professional USB microphone
const usbMic = audioDevices.find(d => 
    d.name.includes('USB') && d.channels === 2
);

await invoke('start_recording', {
    outputPath: 'podcast_episode_42.mp4',
    audioConfig: {
        deviceId: usbMic.id,
        codec: 'opus',           // Best compression
        sampleRate: 48000,       // Professional standard
        channels: 2,
        bitrate: 128000          // 128kbps (transparent quality)
    }
    // Video optional for podcast
});
```

## 📜 License & Philosophy 🦀

MIT License - forever and always.

**Philosophy**: Desktop applications deserve native camera access. 🦀 CrabCamera is camera infrastructure. 📷

## 🚀 What's New in v0.5.0 — THE BIG ONE 📢

### 🎉 **v0.5.0: Full Audio Recording with Perfect A/V Sync** (December 2025)

This is the **game-changing release**. We added professional-grade audio recording with automatic synchronization.

#### ✨ Audio Pipeline (10 Components)
- ✅ **Device Enumeration**: `list_audio_devices()` discovers all audio inputs with sample rate, channels, default status
- ✅ **Audio Capture**: CPAL integration for platform-native audio (Windows WASAPI, macOS AVFoundation, Linux ALSA)
- ✅ **Opus Encoding**: Industry-standard codec at 48kHz (40-256 kbps adaptive bitrate)
- ✅ **AAC Alternative**: Fallback codec for compatibility
- ✅ **PTS Clock**: Shared monotonic timebase (±40ms drift guarantee)
- ✅ **A/V Sync**: Automatic interleaving, no configuration needed
- ✅ **Error Recovery**: Video continues if audio fails (graceful degradation)
- ✅ **Muxide Integration**: Custom MP4 muxer for precise frame timing
- ✅ **Feature Gating**: Audio is optional; core video unaffected
- ✅ **Comprehensive Testing**: 115 unit tests + 8 fuzz tests

#### 📊 Testing Arsenal
- **115 unit tests**: Audio device, encoder, clock, capture, recording integration
- **8 fuzz tests** (proptest): Property-based testing for encoder robustness
- **3 integration tests**: End-to-end A/V recording validation
- **80%+ code coverage**: Enforced per pull request
- **Real hardware**: Validated on OBSBOT Tiny 4K + standard USB microphones

#### 🎬 Codec Support
- **Video**: H.264 (industry standard, wide compatibility)
- **Audio**: Opus (primary, best quality/compression) + AAC (fallback)
- **Container**: MP4 with proper muxing
- **Future**: FLAC, WebM/VP9 (Q1 2026)

#### 🏗️ Architecture Highlights
```
Audio Pipeline:
  Microphone → CPAL Capture → PCM Frames → Opus Encoder → MP4 Muxing
  Video Pipeline:
  Camera → Nokhwa Capture → H.264 Encoder → MP4 Muxing
                                        ↓
                            Synchronized Playback (PTS-based)
```

#### 💻 Platform Support
- ✅ **Windows**: WASAPI audio capture + MediaFoundation video
- ✅ **macOS**: AVFoundation audio + AVFoundation video
- ✅ **Linux**: ALSA audio + V4L2 video

#### 🔒 Security & Stability
- ✅ openh264 upgraded (0.6 → 0.9, RUSTSEC-2025-0008 fixed)
- ✅ Cross-platform CI/CD (Linux, macOS, Windows matrix)
- ✅ All platform-specific issues resolved
- ✅ Zero unsafe code in public API

#### 📈 Stats
- **Lines of Code**: ~3,000 new (audio module)
- **Test Coverage**: 80%+
- **Compilation Time**: 45s (release), 15s (debug)
- **Binary Size**: +2.1MB (audio libs) vs v0.4.1

---

### 🎉 **v0.4.1: Bug Fixes, Performance & DX**
- **Critical Fix**: Mock camera was incorrectly used during `cargo run`
- **PNG Save Fixed**: `save_frame_to_disk()` now properly encodes PNG/JPEG
- **Performance**: Camera warmup reduced from 10 frames to 5 frames
- **macOS Fixed**: Objective-C block syntax and nokhwa API compatibility

### 🎉 **v0.4.0: Quality Intelligence & Configuration**
- **Quality Validation**: Automatic blur/exposure detection with retry
- **TOML Configuration**: Full runtime config with hot-reload
- **Focus Stacking**: Computational photography for macro shots
- **Device Monitoring**: Hot-plug detection for camera connect/disconnect

### 🎉 **v0.3.0: Windows MediaFoundation Camera Controls**
- **Professional Windows Controls**: Full focus, exposure, white balance control
- **Hybrid Architecture**: nokhwa capture + MediaFoundation controls
- **Thread-Safe COM**: Proper Windows COM interface management

---

**Forever maintainer**: Michael A. Kuykendall  
**Promise**: This will never become a paid product  
**Mission**: Making desktop camera development effortless

*"🦀 Native performance. Cross-platform compatibility. Zero hassle. 📷"*

```
       🦀🦀🦀 Happy Coding! 🦀🦀🦀
          Made with ❤️ and Rust
           📷 Capture the moment 📷
```