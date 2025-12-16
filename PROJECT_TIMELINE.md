# CrabCamera + Muxide Unified Project Timeline

**Created:** December 15, 2025  
**Purpose:** Shared timeline across both repositories for session continuity  
**Location:** This file exists in BOTH repos - keep them in sync!

---

## 🎯 Strategic Vision

**Goal:** Build the dominant Rust video recording stack for desktop applications.

**The Opportunity:**
- mp4e (the only Rust muxer) has 2 stars, 1 contributor, broken docs
- No serious pure-Rust MP4 muxer exists in the ecosystem
- CrabCamera + Muxide together = complete camera-to-file pipeline
- Zero FFmpeg, zero GStreamer, zero GPL = unique selling point

**The Plan:**
1. Complete Muxide to production quality (private)
2. Ship CrabCamera v0.5.0 with video recording (uses Muxide internally)
3. Prove Muxide in production via CrabCamera adoption
4. Eventually open-source Muxide and dominate the Rust muxer space

---

## 📅 Unified Timeline

### Phase 1: Foundation (December 2025) ✅ COMPLETE

| Task | Repo | Status | Notes |
|------|------|--------|-------|
| Muxide v0.1.0 core | Muxide | ✅ Done | H.264 + AAC muxing |
| Fast-start support | Muxide | ✅ Done | moov-first for web |
| Metadata (udta) | Muxide | ✅ Done | Title, creation time |
| B-frame support (ctts) | Muxide | ✅ Done | Full decode/display order |
| Fragmented MP4 | Muxide | ✅ Done | DASH/HLS ready |
| 22 tests passing | Muxide | ✅ Done | Full coverage |
| CrabCamera v0.4.0 | CrabCamera | ✅ Done | 157 tests, quality validation |

### Phase 2: Video Recording (January-February 2025)

| Task | Repo | Priority | Dependencies |
|------|------|----------|--------------|
| Add openh264 crate | CrabCamera | P0 | None |
| Design recording API | CrabCamera | P0 | None |
| Add Muxide path dependency | CrabCamera | P0 | Muxide v0.1.0 ✅ |
| `start_recording()` impl | CrabCamera | P0 | openh264, Muxide |
| `stop_recording()` impl | CrabCamera | P0 | start_recording |
| Recording tests | CrabCamera | P0 | Recording API |
| System audio capture research | CrabCamera | P1 | Platform research |
| Audio + video sync | CrabCamera | P1 | Audio capture |
| Quality presets (720p/1080p/4K) | CrabCamera | P1 | openh264 config |
| **Ship CrabCamera v0.5.0** | CrabCamera | **MILESTONE** | All above |

### Phase 3: Windows Excellence (March-April 2025)

| Task | Repo | Priority | Dependencies |
|------|------|----------|--------------|
| MediaFoundation research | CrabCamera | P0 | None (docs exist) |
| IAMCameraControl wrapper | CrabCamera | P0 | windows crate |
| Focus/exposure controls | CrabCamera | P0 | IAMCameraControl |
| IAMVideoProcAmp wrapper | CrabCamera | P1 | windows crate |
| Brightness/contrast/saturation | CrabCamera | P1 | IAMVideoProcAmp |
| PTZ camera support | CrabCamera | P2 | IAMCameraControl |
| **Ship CrabCamera v0.6.0** | CrabCamera | **MILESTONE** | All above |

### Phase 4: Streaming (May-June 2025)

| Task | Repo | Priority | Dependencies |
|------|------|----------|--------------|
| WebRTC local preview | CrabCamera | P0 | webrtc-rs research |
| Live preview component | CrabCamera | P0 | WebRTC |
| RTMP output research | CrabCamera | P1 | Muxide fMP4 ✅ |
| HLS segment generation | CrabCamera | P1 | Muxide fMP4 ✅ |
| DASH segment generation | CrabCamera | P2 | Muxide fMP4 ✅ |
| **Ship CrabCamera v0.7.0** | CrabCamera | **MILESTONE** | All above |

### Phase 5: Professional Features (July-September 2025)

| Task | Repo | Priority | Dependencies |
|------|------|----------|--------------|
| Multi-camera sync | CrabCamera | P0 | Recording stable |
| Audio mixing (mic + system) | CrabCamera | P1 | Audio capture |
| Picture-in-picture | CrabCamera | P2 | Multi-camera |
| Chroma key (green screen) | CrabCamera | P2 | GPU shaders |
| MKV container support | Muxide | P1 | Research |
| WebM support | Muxide | P2 | VP9 research |
| **Ship CrabCamera v0.8.0** | CrabCamera | **MILESTONE** | All above |
| **Ship Muxide v0.2.0** | Muxide | **MILESTONE** | MKV done |

### Phase 6: Muxide Goes Public (Q4 2025)

| Task | Repo | Priority | Dependencies |
|------|------|----------|--------------|
| Production hardening | Muxide | P0 | Real-world usage |
| Comprehensive fuzzing | Muxide | P0 | Security |
| API stabilization | Muxide | P0 | Breaking changes done |
| Documentation polish | Muxide | P0 | All features stable |
| crates.io publication | Muxide | P0 | All above |
| Announce on r/rust | Muxide | P1 | Publication |
| Awesome Rust PR | Muxide | P1 | Publication |
| **Muxide v1.0.0 PUBLIC** | Muxide | **MILESTONE** | Dominate ecosystem |

---

## 🔄 Session Handoff (December 15, 2025)

### What Was Accomplished This Session

1. **Muxide v0.1.0 completed** with all 5 planned features:
   - `audio_frames` in MuxerStats
   - Metadata (udta box) with title/creation_time
   - Fast-start (moov before mdat) - now default
   - B-frame support (ctts box with signed offsets)
   - Fragmented MP4 (FragmentedMuxer API)

2. **22 tests passing** in Muxide

3. **Documentation created:**
   - `MASTER_ROADMAP.md` in CrabCamera (strategic planning)
   - `IMPLEMENTATION_COMPLETE.md` in Muxide (v0.1.0 summary)
   - `.github/copilot-instructions.md` in both repos
   - This timeline document (in both repos)

### What To Do Next Session

1. **Immediate:** Start CrabCamera v0.5.0
   - Add `openh264` to Cargo.toml
   - Add Muxide as path dependency: `muxide = { path = "../muxide" }`
   - Design `RecordingConfig` struct
   - Implement `start_recording()` command

2. **Research needed:**
   - System audio capture (platform-specific)
   - openh264 encoding parameters for quality presets

3. **Don't forget:**
   - Keep Muxide private until v1.0.0
   - No FFmpeg, no GPL, ever
   - Update this timeline as milestones complete

---

## 📊 Success Metrics

### CrabCamera
- [ ] 200+ GitHub stars (currently ~50?)
- [ ] Featured on Awesome Tauri
- [ ] 5+ community integrations
- [ ] Video recording demo viral on Twitter/X

### Muxide (Post-Publication)
- [ ] 50+ GitHub stars within 1 month
- [ ] Featured on Awesome Rust
- [ ] Adopted by 3+ projects
- [ ] "The" Rust muxer (beat mp4e)

---

## 🚫 Anti-Goals (What We're NOT Doing)

- ❌ Mobile apps
- ❌ Cloud storage
- ❌ Social features
- ❌ Image editing
- ❌ FFmpeg integration
- ❌ GPL dependencies
- ❌ Browser-only version

---

## 📁 Key Files Reference

### CrabCamera
| File | Purpose |
|------|---------|
| `MASTER_ROADMAP.md` | Strategic planning (source of truth) |
| `WINDOWS_CONTROLS_ARCHITECTURE.md` | MediaFoundation plan |
| `MEDIAFOUNDATION_API_RESEARCH.md` | API research |
| `.github/copilot-instructions.md` | AI context |
| `src/platform/windows/` | Windows implementation |

### Muxide
| File | Purpose |
|------|---------|
| `IMPLEMENTATION_COMPLETE.md` | v0.1.0 feature summary |
| `docs/charter.md` | Design principles |
| `docs/contract.md` | API invariants |
| `.github/copilot-instructions.md` | AI context |
| `src/api.rs` | Public API |
| `src/fragmented.rs` | fMP4 streaming |

---

## 💡 Strategic Notes

### Why Keep Muxide Private Initially

1. **Prove it works** - CrabCamera is the test bed
2. **Find edge cases** - Real usage reveals bugs
3. **API stability** - Don't commit to public API too early
4. **Competitive moat** - Advantage while we build features

### When To Go Public

- CrabCamera v0.8.0 shipped successfully
- Muxide used in production for 6+ months
- No breaking API changes needed
- Comprehensive test coverage
- Documentation complete

### How To Dominate the Space

1. **Better quality** - More features than mp4e
2. **Better docs** - Actually working examples
3. **Better testing** - Fuzzing, edge cases
4. **Active maintenance** - Respond to issues
5. **Marketing** - r/rust, Twitter, blog posts

---

*This document is the session handoff. Update it at the START and END of each session.*
