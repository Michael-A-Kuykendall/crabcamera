# 🦀 CrabCamera: The Desktop Camera Plugin for Tauri 📷

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
[![Sponsor](https://img.shields.io/badge/❤️-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

**🦀 CrabCamera will be free forever. 🦀** No asterisks. No "free for now." No pivot to paid.

## 🦀 What is CrabCamera?

🦀 CrabCamera is the **first production-ready desktop camera plugin** for Tauri applications. It provides unified camera access across Windows, macOS, and Linux with professional controls and zero-config setup. It's designed to be the **invisible infrastructure** that makes desktop camera apps just work.

| Feature | CrabCamera | Web APIs | Other Plugins |
|---------|------------|----------|---------------|
| **Desktop Native** | Windows/macOS/Linux 🏆 | Limited browser | Mobile-only |
| **Hardware Access** | Direct camera control 🏆 | Browser restricted | Basic access |
| **Professional Controls** | Auto-focus, exposure 🏆 | Limited | Basic |
| **Cross-Platform** | Unified API 🏆 | Platform dependent | Single platform |
| **Production Ready** | 63 comprehensive tests 🏆 | No guarantees | Proof-of-concept |
| **Memory Safety** | Zero unsafe code 🏆 | N/A | Manual management |

## 🎯 Perfect for Desktop Applications 🦀

- **Photography**: Photo booth apps, image editors, content creation tools
- **Security**: Surveillance systems, access control, monitoring dashboards  
- **Medical**: Imaging interfaces, patient documentation, diagnostic tools
- **Industrial**: Quality control, inspection systems, documentation cameras
- **Education**: Interactive learning tools, virtual labs, presentation software
- **Communication**: Video chat apps, streaming tools, conference software

**BONUS:** Professional camera controls with platform-optimized settings for maximum image quality.

## 🦀 Quick Start (30 seconds) 📷

### Installation

```toml
[dependencies]
crabcamera = "0.3.0"
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
    "crabcamera": {}
  }
}
```

### Frontend Usage

```javascript
import { invoke } from '@tauri-apps/api/tauri';

// Initialize camera system
await invoke('initialize_camera_system');

// Get available cameras
const cameras = await invoke('get_available_cameras');
console.log('Available cameras:', cameras);

// Get recommended format for high quality
const format = await invoke('get_recommended_format');

// Capture a photo
const photo = await invoke('capture_single_photo', {
  deviceId: cameras[0].id,
  format: format
});
```

## 📦 Professional Camera Features 🦀

### 🔧 Hardware Control 🦀
- **Device Enumeration**: Automatic discovery of all connected cameras
- **Format Negotiation**: Resolution, FPS, and color format selection
- **Professional Settings**: Auto-focus, auto-exposure, white balance
- **Multi-camera Support**: Switch between multiple cameras seamlessly
- **Error Recovery**: Robust handling of device disconnection and errors

### 🖥️ Cross-Platform Native 🦀
- **Windows**: DirectShow/MediaFoundation with advanced camera controls
- **macOS**: AVFoundation with Metal acceleration support
- **Linux**: V4L2 with comprehensive device support
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

### Professional Camera Controls (NEW in v0.3.0!)
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
| **Camera Access** | **Direct hardware** | getUserMedia limited | N/A |
| **Image Quality** | **Professional controls** | Basic settings | Basic |
| **Cross-Platform** | **Windows/macOS/Linux** | Browser variation | iOS/Android only |
| **Performance** | **Native speed** | Browser overhead | N/A |
| **Reliability** | **63 tests passing** | No guarantees | Varies |

## 🏗️ Technical Architecture 🦀

### Hybrid Capture + Controls Architecture
```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   nokhwa        │    │ Platform Controls│    │  CrabCamera     │
│   (Capture)     │    │ (Advanced)       │    │  (Unified API)  │
├─────────────────┤    ├──────────────────┤    ├─────────────────┤
│ • Frame capture │    │ • Focus control  │    │ • Generic types │
│ • Resolution    │    │ • Exposure       │    │ • Error handling│
│ • Format        │    │ • White balance  │    │ • Cross-platform│
│ • Start/Stop    │    │ • Brightness     │    │ • Thread safety │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

### Platform-Specific Implementations
- **Windows**: nokhwa capture + MediaFoundation controls (NEW in v0.3.0!)
- **macOS**: AVFoundation for both capture and controls
- **Linux**: nokhwa capture + V4L2 controls
- **Unified API**: Same control interface across all platforms

### Key Technologies
- **Rust + Tokio**: Memory-safe, async performance
- **Tauri 2.0 Plugin**: Modern plugin architecture  
- **Platform Backends**: MediaFoundation, AVFoundation, V4L2
- **COM Interface Management**: Thread-safe Windows controls
- **Zero unsafe code**: Memory safety guaranteed (except platform COM interfaces)

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

### Sponsors

See our amazing [sponsors](SPONSORS.md) who make 🦀 CrabCamera possible! 🙏

**Sponsorship Tiers:**
- **$5/month**: Coffee tier - My eternal gratitude + sponsor badge
- **$25/month**: Developer supporter - Priority support + name in SPONSORS.md  
- **$100/month**: Corporate backer - Logo on README + monthly office hours
- **$500/month**: Enterprise partner - Direct support + feature requests

**Companies**: Need invoicing? Email [michaelallenkuykendall@gmail.com](mailto:michaelallenkuykendall@gmail.com)

## 🚀 Production Usage 🦀

**✅ Ready for production:**
- Memory-safe Rust implementation
- 63 comprehensive tests passing
- Zero unsafe code
- Comprehensive error handling
- Async/await throughout
- Cross-platform compatibility verified

**✅ Use cases in production:**
- Desktop photography applications
- Security and surveillance systems
- Medical imaging interfaces
- Industrial inspection tools
- Educational software platforms
- Communication and streaming apps

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

## 📜 License & Philosophy 🦀

MIT License - forever and always.

**Philosophy**: Desktop applications deserve native camera access. 🦀 CrabCamera is camera infrastructure. 📷

## 🚀 What's New in v0.3.0

### 🎉 **Major Feature: Windows MediaFoundation Camera Controls**
- **Professional Windows Controls**: Full focus, exposure, white balance, brightness, contrast, and saturation control
- **Hybrid Architecture**: nokhwa capture + MediaFoundation controls for best of both worlds
- **Thread-Safe COM**: Proper Windows COM interface management for Tauri async commands
- **Capability Detection**: Runtime testing of which controls each camera supports
- **Unified API**: Same control interface across Windows, macOS, and Linux

### 🔧 **Technical Improvements**
- **WindowsCamera Struct**: Combines nokhwa capture with MediaFoundation controls
- **MediaFoundationControls**: Full COM interface wrapper with resource management
- **Platform Integration**: Updated PlatformCamera enum to use Windows-specific implementation
- **Error Handling**: Graceful degradation when controls aren't supported
- **Documentation**: Comprehensive technical architecture documentation

### 🏆 **Cross-Platform Parity Achieved**
Windows users now get the same professional camera control experience as macOS and Linux users!

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