# CrabCamera Video Recording Architecture Analysis

**Document Purpose:** Comprehensive comparison of video recording approaches for CrabCamera v0.5.0  
**Date:** December 14, 2025  
**Status:** Decision pending - seeking external review

---

## Executive Summary

CrabCamera needs video recording capability for v0.5.0. This document analyzes 10 different architectural approaches, weighing trade-offs between bundle size, performance, cross-platform support, and Rust ecosystem alignment.

**Key Insight:** There is a gap in the Rust ecosystem for a simple, unified video muxer crate. Most solutions either wrap ffmpeg or focus on reading, not writing.

---

## Table of Contents

1. [Options Overview](#options-overview)
2. [Detailed Analysis: Each Option](#detailed-analysis)
3. [Gap Analysis: Rust Muxer Ecosystem](#rust-muxer-ecosystem-gap)
4. [Final Comparison Matrix](#final-comparison-matrix)
5. [Recommendation](#recommendation)

---

## Options Overview

| # | Option | Approach | Bundle Impact |
|---|--------|----------|---------------|
| A | ffmpeg-sidecar | Ship ffmpeg binary, call via CLI | +80MB |
| B | ffmpeg-next | Native Rust bindings to ffmpeg libs | +varies (build-time link) |
| C | Cap-style platform crates | Separate encoder per OS | +5-10MB |
| D | openh264-rs | Cisco's patent-free H.264 encoder | +2MB |
| E | rav1e + mp4 | Pure Rust AV1 encoder | +3MB |
| G | WebCodecs | Browser hardware encoding via Tauri webview | +0 (uses browser) |
| H | Image Sequence | Save frames as JPEG, post-process | +0 |
| I | libobs-simple | OBS as a library | +150MB (runtime download) |
| J | webrtc.rs | WebRTC stack with recording | +1MB |

---

## Detailed Analysis

### Option A: ffmpeg-sidecar

**What it is:** Ship the ffmpeg CLI binary alongside your app. Spawn a process and pipe frames to stdin.

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Battle-tested** | ffmpeg has 20+ years of development, handles every edge case |
| **Every codec** | H.264, H.265, AV1, VP9, ProRes, DNxHD, etc. |
| **Every format** | MP4, MKV, MOV, WebM, AVI, etc. |
| **Hardware acceleration** | NVENC, Quick Sync, VCE, VideoToolbox all supported |
| **Streaming** | RTMP, HLS, DASH output built-in |
| **Documentation** | Extensive community knowledge |
| **Proven** | What OBS, HandBrake, and most video apps use |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **Bundle size** | +80MB for ffmpeg binary |
| **Process spawn latency** | ~2 seconds to start ffmpeg process |
| **External dependency** | Not "pure Rust" - shipping a C binary |
| **Version management** | Which ffmpeg version? GPL vs LGPL licensing? |
| **Platform builds** | Need different binaries for Windows/macOS/Linux/ARM |
| **"What everyone does"** | No differentiation from competitors |

#### Verdict
> Reliable but heavyweight. Good as a "Pro Mode" fallback, not ideal as default.

---

### Option B: ffmpeg-next (Rust Bindings)

**What it is:** Native Rust bindings to libavcodec, libavformat, etc. No CLI process.

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Same power as ffmpeg** | All codecs, all formats |
| **No process spawn** | Direct API calls, lower latency |
| **Native speed** | Same performance as C ffmpeg |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **Build complexity** | Requires vcpkg or pkg-config to find ffmpeg libs |
| **ffmpeg version hell** | API changes between ffmpeg versions break things |
| **Unsafe code** | FFI boundary means `unsafe` blocks throughout |
| **Platform-specific builds** | Still need ffmpeg libraries per platform |
| **Documentation sparse** | Rust bindings less documented than CLI |
| **Debugging nightmare** | Errors often come from C layer, hard to trace |

#### Verdict
> All the power of ffmpeg with all the pain of C interop. Avoid unless absolutely necessary.

---

### Option C: Cap-Style Platform Crates

**What it is:** Follow Cap's architecture with separate encoder crates per platform:
- `enc-mediafoundation` (Windows)
- `enc-avfoundation` (macOS)
- `enc-ffmpeg` (Linux fallback)

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Native encoders** | Uses OS-provided hardware encoders |
| **Hardware acceleration** | NVENC/Quick Sync on Windows, VideoToolbox on macOS |
| **No bundled codec** | Uses what's already on the system |
| **Cap proved it works** | Production-tested approach |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **3x maintenance** | Three completely different codebases |
| **Platform expertise needed** | Must understand MediaFoundation, AVFoundation, and ffmpeg |
| **Testing burden** | Every feature needs testing on all 3 platforms |
| **Inconsistent behavior** | Different platforms may have different capabilities |
| **Long development time** | Months of work just for encoding |

#### Verdict
> The "right" way architecturally but massive scope for a small team. Not recommended for v0.5.0.

---

### Option D: openh264-rs ⭐ RECOMMENDED

**What it is:** Rust bindings to Cisco's OpenH264 library. BSD licensed, patent-free (Cisco pays the license fees).

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Pure Rust bindings** | Idiomatic Rust API |
| **Patent-free** | Cisco's license covers H.264 patents |
| **Small footprint** | ~2MB for the library |
| **Real-time 1080p** | Fast enough for 1080p30 on modern CPUs |
| **Cross-platform** | Windows, macOS, Linux all supported |
| **Well-maintained** | 0.9.0 as of late 2024, active development |
| **No build complexity** | Pre-built binaries, no vcpkg needed |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **H.264 only** | No H.265, AV1, VP9 |
| **No hardware acceleration** | CPU encoding only |
| **4K not real-time** | CPU can't keep up with 4K encoding |
| **Still need muxer** | openh264 outputs NAL units, need MP4 container |
| **Audio separate** | Need another crate for audio encoding |

#### Verdict
> Best balance of simplicity, size, and capability. Covers 90% of use cases (1080p and below).

---

### Option E: rav1e + mp4 (Pure Rust AV1)

**What it is:** Pure Rust AV1 encoder from Mozilla/Netflix, paired with pure Rust MP4 muxer.

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **100% Pure Rust** | No C dependencies in encoder (mp4 crate is also pure Rust) |
| **Future codec** | AV1 is ~25% smaller than H.264 at same quality |
| **Auditable** | Can review every line of code |
| **Active development** | Netflix backing, 4k GitHub stars |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **CPU encoding is SLOW** | 1080p30 at speed 10: ~10-15 fps (NOT real-time) |
| **Needs NASM** | x86_64 assembly optimizations require NASM |
| **Player compatibility** | AV1 needs modern players (2020+) |
| **Memory hungry** | AV1 encoding uses significant RAM |
| **720p max real-time** | Even speed 10 can't do 1080p in real-time |

#### Verdict
> Beautiful architecture, wrong codec for real-time recording. AV1 is for offline encoding.

---

### Option G: WebCodecs (Browser Hardware Encoding)

**What it is:** Use Tauri's webview to access the browser's `VideoEncoder` API, which delegates to GPU encoders.

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Zero native dependencies** | Browser has the encoders |
| **Hardware acceleration** | Uses NVENC, Quick Sync, VCE via browser |
| **Cross-platform** | Same code works everywhere |
| **Browser updates encoders** | You get improvements for free |
| **90% browser support** | Chrome 94+, Firefox 130+, Safari 16.4+ |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **JavaScript dependency** | Must transfer frames from Rust to JS |
| **Muxer required** | WebCodecs outputs chunks, not files |
| **JS muxer library needed** | MP4Box.js or mp4-muxer (~50KB) |
| **Single-threaded JS** | Can bottleneck on encoding callback |
| **Debugging complexity** | Errors span two runtimes |
| **Browser quirks** | Safari video-only until recently |

#### Verdict
> Clever but makes JavaScript a critical path component. Acceptable as optional mode, not default.

---

### Option H: Image Sequence + Post-Process

**What it is:** Save raw frames as JPEG/PNG files, convert to video later (user runs ffmpeg or we bundle converter).

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Zero encoding overhead** | Just file writes |
| **Lossless quality** | Frames exactly as captured |
| **Frame-by-frame editing** | Can delete/modify individual frames |
| **1-day implementation** | Trivially simple |
| **Crash recovery** | Files already saved, nothing lost |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **Massive disk usage** | ~10MB/sec for JPEG, ~50MB/sec for PNG |
| **No real-time playback** | Can't preview while recording |
| **Post-processing required** | User must convert to video |
| **Not what users expect** | "Record video" should give video file |

#### Verdict
> Great for specialized use cases (time-lapse, forensics), not for general recording.

---

### Option I: libobs-simple (OBS as a Library)

**What it is:** Rust wrapper around OBS's libobs library.

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Full OBS power** | x264, NVENC, AMF, Quick Sync, all outputs |
| **Battle-tested encoding** | Millions of OBS users validate quality |
| **Streaming built-in** | RTMP, HLS, etc. |
| **Scene composition** | Overlays, transitions, etc. |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **No macOS support** | "We are working on that" - README |
| **150MB runtime download** | OBS binaries downloaded at first run |
| **Immature wrapper** | 26 GitHub stars, 4 open issues |
| **API unstable** | "Will definitely have breaking revisions" |
| **Massive dependency** | You're shipping OBS |
| **Overkill** | CrabCamera doesn't need scene composition |

#### Verdict
> **DISQUALIFIED** - No macOS support is a deal-breaker for CrabCamera.

---

### Option J: webrtc.rs + Recording

**What it is:** Use pure Rust WebRTC stack for streaming, add recording as side effect.

#### FOR ✅

| Benefit | Details |
|---------|---------|
| **Pure Rust** | webrtc.rs is 100% Rust |
| **Streaming-first** | Native WebRTC capability |
| **Differentiation** | Most camera libs don't have this |
| **Good maturity** | 2.4k stars, corporate sponsors |

#### AGAINST ❌

| Drawback | Details |
|----------|---------|
| **Not designed for recording** | WebRTC is for real-time streaming |
| **Complex protocol** | ICE, DTLS, SRTP - lots of moving parts |
| **Overkill for local files** | Massive machinery for "save to disk" |
| **Codec limitations** | VP8/VP9 primarily, H.264 with effort |

#### Verdict
> Great for v0.6.0+ streaming feature, not for v0.5.0 recording.

---

## Rust Muxer Ecosystem Gap

**Your instinct is correct** - there's a gap here. Here's what exists:

### Available Muxer Crates

| Crate | Stars | Purpose | Mux Support | Notes |
|-------|-------|---------|-------------|-------|
| `mp4` | ~300 | MP4 read/write | ✅ Writing | Has `Mp4Writer`, most complete |
| `scuffle-mp4` | ~50 | MP4 parse/write | ✅ Writing | Pure Rust, newer |
| `async-mp4` | ~20 | Async MP4 | ✅ Writing | Async-first design |
| `mp4e` | ~5 | Simple muxer | ✅ Writing | "Simple MP4 muxer" |
| `matroska` | ~100 | MKV metadata | ❌ Read-only | Parser only |
| `webm-iterable` | ~30 | WebM parsing | ❌ Read-only | Parser only |

### The Gap

**What's missing:** A unified, simple, streaming-friendly muxer that:
1. Accepts encoded frames from any encoder (openh264, rav1e, etc.)
2. Writes to MP4, MKV, or WebM
3. Handles audio interleaving
4. Supports fragmented MP4 for streaming
5. Is pure Rust with no ffmpeg dependency

**Current state:**
- `mp4` crate works but requires deep understanding of MP4 boxes
- No equivalent to ffmpeg's "just give me frames and I'll figure it out"
- Each container format is a separate crate
- Audio + video sync is left to user

### Opportunity

A crate called something like `av-mux` that provides:
```rust
let mut muxer = Muxer::new("output.mp4")
    .video_track(VideoCodec::H264, 1920, 1080, 30.0)
    .audio_track(AudioCodec::AAC, 48000, 2)
    .build()?;

muxer.write_video_frame(pts, data, is_keyframe)?;
muxer.write_audio_frame(pts, data)?;
muxer.finalize()?;
```

**This doesn't exist in the Rust ecosystem.** The `mp4` crate is close but requires manual box construction.

---

## Final Comparison Matrix

| Criteria | A: ffmpeg-sidecar | B: ffmpeg-next | C: Cap-style | D: openh264 ⭐ | E: rav1e | G: WebCodecs | H: Image Seq | I: libobs | J: webrtc |
|----------|------------------|----------------|--------------|---------------|----------|--------------|--------------|-----------|-----------|
| **Bundle Size** | +80MB | +varies | +5-10MB | +2MB | +3MB | +0 | +0 | +150MB | +1MB |
| **Real-time 1080p** | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ | ✅ | ✅ | ⚠️ |
| **Real-time 4K** | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | ✅ | ✅ | ⚠️ |
| **Hardware Accel** | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ | N/A | ✅ | ⚠️ |
| **macOS Support** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ❌ | ✅ |
| **Pure Rust** | ❌ | ❌ | ❌ | ⚠️ | ✅ | ❌ (JS) | ✅ | ❌ | ✅ |
| **Implementation Time** | 1 week | 2-3 weeks | 2-3 months | 1-2 weeks | 1-2 weeks | 1 week | 1 day | 2 weeks | 3-4 weeks |
| **Maintenance Burden** | Low | High | Very High | Low | Low | Medium | Very Low | High | Medium |
| **Differentiation** | None | None | Medium | Low | Medium | Medium | Low | None | High |

### Scoring (1-10, higher is better)

| Criteria | Weight | A | B | C | D ⭐ | E | G | H | I | J |
|----------|--------|---|---|---|------|---|---|---|---|---|
| Small bundle | 20% | 2 | 5 | 6 | 9 | 8 | 10 | 10 | 1 | 9 |
| Real-time performance | 25% | 10 | 10 | 10 | 7 | 3 | 9 | 10 | 10 | 6 |
| Cross-platform | 20% | 9 | 7 | 6 | 9 | 9 | 8 | 10 | 3 | 9 |
| Rust ecosystem fit | 15% | 3 | 4 | 5 | 8 | 10 | 4 | 9 | 3 | 9 |
| Implementation ease | 10% | 8 | 4 | 2 | 7 | 7 | 7 | 10 | 5 | 4 |
| Maintenance | 10% | 7 | 3 | 2 | 8 | 7 | 6 | 9 | 4 | 6 |
| **WEIGHTED TOTAL** | | **6.6** | **6.1** | **5.8** | **7.9** | **6.7** | **7.7** | **9.6** | **4.6** | **7.0** |

**Note:** Image Sequence scores high but doesn't deliver what users expect from "video recording."

---

## Recommendation

### Primary: openh264-rs + mp4 crate

**Why:**
- Best balance of bundle size (+2MB) and capability
- Real-time 1080p30 on modern hardware
- Cross-platform without platform-specific code
- Patent-free (Cisco licensed)
- Fits Rust ecosystem philosophy

**Trade-offs accepted:**
- No hardware acceleration (CPU encoding)
- No 4K real-time
- H.264 only (no H.265/AV1)

**Implementation:**
```
nokhwa → openh264 encoder → mp4 crate → file
```

### Optional Fallback: ffmpeg-sidecar

**For users who need:**
- 4K recording
- Hardware acceleration
- Exotic codecs (H.265, ProRes)

**Configuration:**
```toml
[recording]
engine = "native"  # default, uses openh264
# engine = "ffmpeg" # optional, requires ffmpeg binary
```

### Future Consideration: WebCodecs

Could be added as experimental option for users who:
- Want zero binary size
- Accept JavaScript in critical path
- Are okay with browser dependency

### Disqualified

- **libobs-simple:** No macOS support
- **rav1e:** Not real-time
- **Cap-style:** Too much scope

---

## Open Questions for External Review

1. **Is 1080p limitation acceptable?** Most users won't record 4K, but is the limitation surprising?

2. **Should hardware acceleration be a v0.5.0 requirement?** The complexity vs. benefit trade-off.

3. **Is the muxer gap worth filling?** Should CrabCamera contribute an `av-mux` crate to the ecosystem?

4. **ffmpeg bundled or optional download?** Should Pro Mode download ffmpeg on first use vs. bundling it?

5. **WebCodecs as "experimental"?** Worth the complexity for zero-binary-size use case?

---

## Appendix: Research Sources

- ffmpeg-sidecar: https://crates.io/crates/ffmpeg-sidecar (501 stars)
- openh264-rs: https://crates.io/crates/openh264 (107 stars)
- rav1e: https://github.com/xiph/rav1e (4k stars)
- mp4 crate: https://crates.io/crates/mp4 (~300 stars)
- libobs-rs: https://github.com/libobs-rs/libobs-rs (26 stars)
- webrtc.rs: https://webrtc.rs/ (2.4k stars)
- WebCodecs: https://developer.mozilla.org/en-US/docs/Web/API/WebCodecs_API
- Cap: https://github.com/CapSoftware/Cap

---

*Document prepared for external review. Please provide feedback on architecture choices and trade-offs.*
