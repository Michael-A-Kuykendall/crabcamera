# GitHub Copilot Instructions - CrabCamera

## 🎯 Project Identity

**CrabCamera** is a production-ready desktop camera plugin for Tauri applications.
- **Mission:** Invisible camera infrastructure - drop it in, it works
- **License:** MIT (no GPL contamination)
- **Philosophy:** Pure Rust, zero FFmpeg, single binary

## 🔗 Related Projects

**Muxide** (private, `../muxide/`) is our custom MP4 muxer - a competitive moat that eliminates external dependencies. CrabCamera consumes Muxide via path dependency for video recording.

## 📋 Current Roadmap (December 2025)

### CrabCamera v0.5.0 - Video Recording (Q1 2025)
- [ ] openh264 encoder integration
- [ ] Muxide integration (path dep to `../muxide`)
- [ ] `start_recording()` / `stop_recording()` API
- [ ] Recording with audio
- [ ] Quality presets (720p, 1080p, 4K)

### CrabCamera v0.6.0 - Windows First-Class (Q2 2025)
- [ ] MediaFoundation camera controls
- [ ] IAMCameraControl (focus, exposure, pan/tilt/zoom)
- [ ] IAMVideoProcAmp (brightness, contrast, saturation)

### CrabCamera v0.7.0 - Live Streaming (Q3 2025)
- [ ] WebRTC local preview
- [ ] RTMP/HLS/DASH streaming (uses Muxide fMP4)

### CrabCamera v0.8.0 - Broadcast Ready (Q4 2025)
- [ ] Multi-camera recording
- [ ] Chroma key (green screen)
- [ ] Audio mixing

## ⚠️ Critical Rules

1. **READ BEFORE WRITE** - Always read files before editing
2. **No FFmpeg** - Never add FFmpeg, GStreamer, or libav dependencies
3. **No GPL** - Only MIT/Apache-2.0 compatible dependencies
4. **Muxide is Private** - Don't suggest publishing it to crates.io
5. **Pure Rust** - Avoid C bindings where possible

## 🏗️ Architecture

```
CrabCamera (Tauri Plugin)
    ├── nokhwa (camera capture)
    ├── openh264 (video encoding) [v0.5.0]
    └── muxide (MP4 muxing, private) [v0.5.0]
```

## 📁 Key Files

- `MASTER_ROADMAP.md` - Strategic planning (source of truth)
- `WINDOWS_CONTROLS_ARCHITECTURE.md` - MediaFoundation implementation plan
- `src/platform/windows/` - Windows-specific code
- `src/commands/` - Tauri command handlers

## 🔧 Development Commands

```bash
cargo test                    # Run all tests (expect 157+)
cargo build --release        # Production build
cargo doc --open             # Generate docs
```

## 📊 Current Stats
- Tests: 157+ passing
- Platforms: Windows, macOS, Linux
- Zero unsafe code in public API
