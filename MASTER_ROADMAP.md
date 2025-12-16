# CrabCamera + Muxide Master Roadmap

**Last Updated:** December 15, 2025  
**Status:** Strategic Planning Document (Private)

---

## 🔐 Strategic Context

**Muxide** is being developed as a **private competitive moat** - a pure Rust MP4 muxer that eliminates external dependencies and gives CrabCamera a unique advantage. It will NOT be published to crates.io initially.

**CrabCamera** will consume Muxide internally via path dependency, enabling video recording without FFmpeg, GStreamer, or any other heavyweight dependency.

---

## Current State (December 2025)

### CrabCamera v0.4.0 ✅
- 157+ tests passing
- Cross-platform camera access (Windows/macOS/Linux)
- Professional hardware controls
- Quality validation (blur/exposure detection)
- Focus stacking for macro photography
- Device hot-plug monitoring
- Tauri 2.0 plugin architecture

### Muxide v0.1.0 ✅ (Private)
- 22 tests passing
- H.264 Annex B → MP4 muxing
- AAC ADTS audio support
- Fast-start (moov-first) for web streaming
- Metadata (title, creation time)
- B-frame support (ctts box)
- Fragmented MP4 for DASH/HLS
- Zero external dependencies

---

## 🎯 Release Schedule

### CrabCamera v0.5.0 - Video Recording (Q1 2025)
**Theme:** "Record Everything"

| Feature | Priority | Dependency | Status |
|---------|----------|------------|--------|
| openh264 encoder integration | P0 | None | Not Started |
| Muxide integration (path dep) | P0 | Muxide v0.1.0 ✅ | Ready |
| `start_recording()` / `stop_recording()` API | P0 | openh264, Muxide | Not Started |
| Recording with audio (system audio capture) | P1 | platform-specific | Not Started |
| Recording quality presets (720p, 1080p, 4K) | P1 | openh264 | Not Started |
| Recording metadata (title, timestamp) | P2 | Muxide metadata ✅ | Ready |

### CrabCamera v0.6.0 - Windows First-Class (Q2 2025)
**Theme:** "Windows Dominance"

| Feature | Priority | Notes |
|---------|----------|-------|
| MediaFoundation camera controls | P0 | Focus, exposure, white balance |
| IAMCameraControl integration | P0 | Pan/tilt/zoom for PTZ cameras |
| IAMVideoProcAmp integration | P1 | Brightness, contrast, saturation |
| Windows virtual camera support | P2 | OBS Virtual Cam, etc. |
| Windows-specific optimizations | P2 | Hardware encoding hints |

### CrabCamera v0.7.0 - Live Streaming (Q3 2025)
**Theme:** "Stream Anywhere"

| Feature | Priority | Dependency |
|---------|----------|------------|
| WebRTC local preview | P0 | webrtc-rs |
| RTMP streaming output | P1 | Muxide fMP4 ✅ |
| HLS segment generation | P1 | Muxide fMP4 ✅ |
| DASH segment generation | P2 | Muxide fMP4 ✅ |
| Low-latency streaming mode | P2 | Muxide + optimizations |

### CrabCamera v0.8.0 - Professional Features (Q4 2025)
**Theme:** "Broadcast Ready"

| Feature | Priority | Notes |
|---------|----------|-------|
| Multi-camera recording | P0 | Sync multiple streams |
| Picture-in-picture composition | P1 | Overlay management |
| Chroma key (green screen) | P2 | GPU shader |
| Audio mixing (mic + system) | P1 | Cross-platform audio capture |
| Recording overlays (timestamp, logo) | P2 | Text/image overlay |

---

## 🔧 Muxide Evolution (Private)

### Muxide v0.2.0 - Container Expansion
| Feature | Status | Notes |
|---------|--------|-------|
| MKV/Matroska output | Not Started | More flexible than MP4 |
| WebM (VP9/Opus) | Not Started | Web-native format |
| Chapter markers | Not Started | Navigation support |
| Subtitle streams | Not Started | SRT → MP4 embedding |

### Muxide v0.3.0 - Advanced Codecs
| Feature | Status | Notes |
|---------|--------|-------|
| HEVC/H.265 support | Not Started | 4K efficiency |
| AV1 support | Not Started | Future-proof codec |
| Opus audio | Not Started | Better than AAC |
| Multi-track audio | Not Started | Multiple languages |

### Muxide v1.0.0 - Production Hardening
| Feature | Status | Notes |
|---------|--------|-------|
| Async writer support | Not Started | Non-blocking I/O |
| Memory-mapped output | Not Started | Large file performance |
| Corruption recovery | Not Started | Resume from crash |
| Comprehensive fuzzing | Not Started | Security hardening |

---

## 🧪 Research & Experimentation

### Near-Term Research (2025)

| Topic | Purpose | Priority |
|-------|---------|----------|
| Hardware encoder APIs | GPU acceleration | High |
| NVENC/QuickSync/VCE bindings | Platform-specific perf | Medium |
| Candle integration | AI-powered features | Medium |
| OpenCV Rust bindings | Computer vision | Low |

### Future Exploration (2026+)

| Topic | Purpose | Priority |
|-------|---------|----------|
| Neural enhancement | AI upscaling/denoising | Experimental |
| Object tracking | Auto-follow subjects | Experimental |
| Scene detection | Auto-split recordings | Experimental |
| Voice-activated controls | Hands-free operation | Experimental |

---

## 🎯 Strategic Priorities

### Competitive Advantages to Maintain

1. **Zero FFmpeg** - No subprocess spawning, no 80MB binary
2. **Pure Rust** - Memory safety, cross-compilation, single binary
3. **Muxide Moat** - Custom muxer nobody else has
4. **Tauri Native** - First-class desktop integration
5. **MIT License** - No GPL contamination

### What We're NOT Building

- ❌ Cloud storage integration
- ❌ Social media posting
- ❌ Image editing suite
- ❌ Mobile apps
- ❌ Browser-only version

### Dependencies to Avoid

- ❌ FFmpeg (binary or linking)
- ❌ GStreamer
- ❌ libav*
- ❌ Any GPL code
- ❌ Node.js runtime

---

## 📊 Success Metrics

### CrabCamera
- [ ] 200+ GitHub stars
- [ ] Featured on Awesome Tauri list
- [ ] 5+ community integrations
- [ ] Zero open CVEs
- [ ] <1s cold start time

### Muxide (Internal Metrics)
- [ ] 100% test coverage on critical paths
- [ ] <1ms per frame muxing overhead
- [ ] Zero memory leaks under fuzzing
- [ ] Plays in all major players (VLC, QuickTime, browser)

---

## 🔄 Version Sync Strategy

| CrabCamera | Muxide | Notes |
|------------|--------|-------|
| v0.5.0 | v0.1.x | Initial video recording |
| v0.6.0 | v0.1.x | Windows controls (no muxide changes) |
| v0.7.0 | v0.1.x | Streaming (fMP4 already done) |
| v0.8.0 | v0.2.x | Multi-format recording (MKV) |

---

## 📝 Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2025-12-15 | Rejected mp4e crate | 2 stars, broken docs, 1 contributor |
| 2025-12-15 | Created Muxide | Need reliable, owned muxer |
| 2025-12-15 | Keep Muxide private | Competitive moat, not ready for public |
| 2025-12-15 | openh264 for encoding | Patent-free, ~2MB, real-time capable |
| 2025-12-15 | fMP4 in v0.1.0 | Stream-first architecture |

---

## 🚀 Immediate Next Steps

1. **[ ] CrabCamera v0.5.0 Planning**
   - Add openh264 dependency
   - Design recording API
   - Wire up Muxide via path dependency

2. **[ ] MediaFoundation Controls**
   - Implement `find_media_source()` in Windows controls
   - Map IAMCameraControl properties
   - Test with real cameras

3. **[ ] Documentation**
   - Update README with video recording preview
   - Create recording example
   - Document quality presets

---

*This document is the source of truth for CrabCamera + Muxide strategic planning.*
