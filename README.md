# CrabCamera 🦀

**Production-ready desktop camera & audio plugin for Tauri applications.**

![CrabCamera Logo](https://raw.githubusercontent.com/Michael-A-Kuykendall/crabcamera/main/assets/logo.png)

[![Crates.io](https://img.shields.io/crates/v/crabcamera.svg)](https://crates.io/crates/crabcamera)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://rustup.rs/)
[![Tests](https://img.shields.io/badge/tests-196%2F196-brightgreen.svg)](https://github.com/Michael-A-Kuykendall/crabcamera)
[![Sponsor](https://img.shields.io/badge/%E2%9D%A4%EF%B8%8F-Sponsor-ea4aaa?logo=github)](https://github.com/sponsors/Michael-A-Kuykendall)

CrabCamera is the first production-ready desktop camera + audio plugin for Tauri—unified camera and audio access across Windows, macOS, and Linux with professional controls, synchronized A/V recording, and zero-config setup.

**Free forever. MIT license. No asterisks.**

---

## Quick Start

### Installation

```toml
[dependencies]
crabcamera = { version = "0.9", features = ["recording", "audio"] }
tauri = { version = "2.0" }
```

### Register the plugin

```rust
// src-tauri/src/main.rs
fn main() {
    tauri::Builder::default()
        .plugin(crabcamera::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Initialize and capture

```javascript
import { invoke } from '@tauri-apps/api/core';

// Initialize (no parameters needed—CrabCamera discovers cameras automatically)
await invoke('plugin:crabcamera|initialize_camera_system');

// Capture a photo (device_id and format are optional; defaults to camera 0)
const frame = await invoke('plugin:crabcamera|capture_single_photo', {
  deviceId: "0",
  format: null
});
```

> All commands use the `plugin:crabcamera|` prefix when called via `invoke`.

For vanilla JS (no bundler), enable `withGlobalTauri: true` in `tauri.conf.json` and use `window.__TAURI__.core.invoke`.

---

## Using CrabCamera from Rust

CrabCamera is not Tauri-only. Three additional entry points exist for direct Rust usage:

```rust
// Direct platform camera (no Tauri required)
use crabcamera::platform::{PlatformCamera, CameraInitParams};
let camera = PlatformCamera::new(CameraInitParams::default())?;
let frame = camera.capture_frame()?;

// Headless session (server / CLI context)
use crabcamera::headless::HeadlessSession;
let session = HeadlessSession::new(config)?;

// CLI binary
// cargo run --bin crabcamera-cli -- --help
```

See [`examples/quick_test.rs`](examples/quick_test.rs) to verify your hardware works before building anything else.

---

## Examples

Every example runs with `cargo run --example <name>`.

### Verify hardware

| Example | What it does |
|---------|-------------|
| [`quick_test`](examples/quick_test.rs) | List cameras, warm up, take a photo—start here |
| [`hardware_audit`](examples/hardware_audit.rs) | Tests every CrabCamera command against real hardware |
| [`functional_test`](examples/functional_test.rs) | Full capture + recording flow with proper warm-up |
| [`visual_camera_test`](examples/visual_camera_test.rs) | Saves actual frames so you can see what the camera sees |

```bash
cargo run --example quick_test
cargo run --example hardware_audit
cargo run --example functional_test --features recording --release
cargo run --example visual_camera_test
```

### Recording

| Example | What it does |
|---------|-------------|
| [`record_video`](examples/record_video.rs) | Records 5 seconds of video to MP4 |
| [`live_av_recording`](examples/live_av_recording.rs) | Full A/V recording with microphone sync |
| [`live_audio_test`](examples/live_audio_test.rs) | Audio pipeline: enumerate device, capture PCM, encode Opus |
| [`save_test_output`](examples/save_test_output.rs) | Saves a raw frame, a CrabCamera frame, and a 3-second MP4 |

```bash
cargo run --example record_video --features recording
cargo run --example live_av_recording --features "recording,audio"
cargo run --example live_audio_test --features audio
```

### Advanced capture

| Example | What it does |
|---------|-------------|
| [`smart_capture_demo`](examples/smart_capture_demo.rs) | Smart Trigger—auto-captures when image quality is stable |
| [`camera_preview`](examples/camera_preview.rs) | Start/stop preview stream and capture a frame |
| [`camera_warmup_analysis`](examples/camera_warmup_analysis.rs) | How long does your camera need before frames are valid? |

```bash
cargo run --example smart_capture_demo
cargo run --example camera_preview
cargo run --example camera_warmup_analysis --release
```

### Low-level debugging

| Example | What it does |
|---------|-------------|
| [`direct_capture`](examples/direct_capture.rs) | Raw nokhwa capture at native resolution (bypasses CrabCamera) |
| [`raw_nokhwa_test`](examples/raw_nokhwa_test.rs) | Tests the nokhwa layer directly |
| [`format_debug`](examples/format_debug.rs) | Inspects camera format negotiation |
| [`reuse_debug`](examples/reuse_debug.rs) | Debugs camera handle reuse behavior |
| [`audit_flow_debug`](examples/audit_flow_debug.rs) | Step-through trace of what hardware_audit does |
| [`test_encoder_output`](examples/test_encoder_output.rs) | Tests openh264 output format |

---

## Features

### Camera capture
- **Device discovery**—automatic enumeration with capability detection
- **Format selection**—resolution, FPS, and pixel format control
- **Professional controls**—auto/manual focus, exposure, white balance
- **Quality retry**—blur and exposure scoring; retries until threshold is met
- **Smart Trigger**—waits for quality to stabilize before capturing

### A/V recording
- **H.264 video** via openh264
- **Opus audio** (primary) and AAC (fallback) via CPAL
- **PTS-based sync**—shared monotonic timebase, ±40ms max drift over a 60-minute recording
- **MP4 container** via Muxide

### Focus stacking and HDR
- **Focus stacking**—capture focus-bracketed sequences, merge via Laplacian pyramid blending
- **HDR sequences**—exposure-bracketed burst capture

### Reliability
- **Invariant Superhighway**— 40+ runtime correctness checks across all critical paths
- **Feature Registry**—every capability declared as `Implemented`, `Beta`, `Stub`, or `Planned`
- **196/196 lib tests** passing; property-based tests for encoder and sync invariants
- **Platform transparency**—hardware-unsupported controls log warnings; structural errors return `Err`

---

## Command Reference

### Initialization

```rust
initialize_camera_system(params: CameraInitParams) -> Result<String>
get_available_cameras() -> Result<Vec<CameraDeviceInfo>>
get_platform_info() -> Result<PlatformInfo>
test_camera_system() -> Result<SystemTestResult>
release_camera() -> Result<()>
```

### Capture

```rust
// Consolidated capture command (preferred)
capture(options: CaptureOptions) -> Result<CaptureResult>
//   modes: CaptureMode::Single | Sequence { count, interval_ms } | QualityRetry { max_attempts, min_quality_score }

// Granular commands (available for backward compatibility)
capture_single_photo(device_id: Option<String>, format: Option<CameraFormat>) -> Result<CameraFrame>
capture_with_quality_retry(params: QualityRetryParams) -> Result<CameraFrame>
capture_photo_sequence(params: SequenceParams) -> Result<Vec<CameraFrame>>
capture_burst_sequence(params: BurstParams) -> Result<Vec<CameraFrame>>
save_frame_to_disk(frame: CameraFrame, path: String) -> Result<()>
save_frame_compressed(frame: CameraFrame, path: String, quality: u8) -> Result<()>
```

### Camera controls

```rust
// Consolidated settings command (preferred)
apply_camera_settings(settings: CameraSettingsInput) -> Result<ControlApplicationResult>
//   fields: focus_distance, exposure_time, iso_sensitivity, white_balance, controls

// Granular commands (available for backward compatibility)
get_camera_controls(device_id: String) -> Result<CameraControls>
set_camera_controls(device_id: String, controls: CameraControls) -> Result<ControlApplicationResult>
set_manual_focus(device_id: String, value: f32) -> Result<ControlApplicationResult>
set_manual_exposure(device_id: String, value: f32) -> Result<ControlApplicationResult>
set_white_balance(device_id: String, wb: WhiteBalance) -> Result<ControlApplicationResult>
test_camera_capabilities(device_id: String) -> Result<CameraCapabilities>
```

### Recording (`recording` feature)

```rust
start_recording(
    output_path: String,
    device_id: String,
    width: u32, height: u32, fps: f64,
    audio_device_id: Option<String>,
    // ...codec options
) -> Result<String>
stop_recording() -> Result<RecordingStatus>
get_recording_status() -> Result<RecordingStatus>
```

### Quality analysis

```rust
analyze_frame_blur(frame: CameraFrame) -> Result<BlurMetrics>
analyze_frame_exposure(frame: CameraFrame) -> Result<ExposureMetrics>
validate_frame_quality(frame: CameraFrame) -> Result<QualityScore>
```

### Advanced / focus stacking

```rust
// Consolidated (preferred)
capture_focus_stack(params: FocusStackParams) -> Result<CameraFrame>

// Granular (available for backward compatibility)
capture_focus_brackets_command(params: FocusBracketParams) -> Result<Vec<CameraFrame>>
capture_hdr_sequence(params: HdrParams) -> Result<Vec<CameraFrame>>
```

### Permissions

```rust
request_camera_permission() -> Result<bool>
check_camera_permission_status() -> Result<PermissionStatus>
```

---

## Platform support

| Platform | Camera capture | Controls | Audio | Recording |
|----------|---------------|----------|-------|-----------|
| Windows | DirectShow / MediaFoundation | IAMCameraControl / IAMVideoProcAmp | WASAPI | ✅ |
| macOS | AVFoundation | AVFoundation | AVFoundation | ✅ |
| Linux | V4L2 | V4L2 | ALSA | ✅ |

---

## Architecture

```
crabcamera/
├── src/commands/        Tauri command handlers (capture, recording, advanced, init)
├── src/platform/        Platform-specific camera backends (Windows, macOS, Linux)
├── src/quality/         Blur, exposure, and composition scoring; Smart Trigger
├── src/recording/       H.264 + Opus encoding; MP4 mux via Muxide
├── src/audio/           CPAL-based audio capture and encoding
├── src/focus_stack/     Laplacian pyramid blend for focus stacking
├── src/headless/        Non-Tauri HeadlessSession API
├── src/bin/             crabcamera-cli binary
├── src/invariant_ppt.rs Runtime invariant assertion framework
└── src/registry.rs      Feature status registry
```

Cargo features:
- `recording`—enables MP4 recording commands (openh264 + Muxide)
- `audio`—enables audio capture and encoding (Opus via CPAL)
- `headless`—enables HeadlessSession API for server/CLI usage

---

## Testing

```bash
# Core library (196 tests)
cargo test --lib

# With recording feature (196 tests)
cargo test --lib --features recording

# Integration tests
cargo test --test commands_capture_test
cargo test --test commands_init_test
cargo test --test commands_advanced_test
cargo test --test commands_permissions_test
cargo test --test focus_stack_test

# Compile check, all features
cargo check --all-features
```

---

## License

[MIT](LICENSE-MIT)—forever.

If CrabCamera saves you time, [sponsoring](https://github.com/sponsors/Michael-A-Kuykendall) keeps it moving forward.

---

*Made with Rust 🦀*
