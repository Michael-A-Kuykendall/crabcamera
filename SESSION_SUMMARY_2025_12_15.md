# Session Summary: CrabCamera v0.5.0 Video Recording Architecture + Muxide Creation

**Date:** December 15, 2025  
**Workspace:** `crabcamera-muxide.code-workspace`

---

## 1. Session Overview

This session focused on researching and deciding the video recording architecture for CrabCamera v0.5.0, which led to the creation of **Muxide** - a new pure Rust MP4 muxer crate.

### Key Decisions Made

1. **Encoder Choice:** openh264 (Cisco's patent-free H.264, ~2MB, real-time 1080p30 capable)
2. **Muxer Choice:** Muxide (custom-built) instead of mp4e (rejected due to immaturity: 2 stars, 1 contributor)
3. **Container:** MP4 (ISOBMFF) only for v0.1.0
4. **Format:** H.264 Annex B input, AVCC output in container

---

## 2. Research Conducted

### Video Recording Options Evaluated (10 total)

| Option | Verdict |
|--------|---------|
| A: FFmpeg subprocess | Rejected - 80MB binary, IPC overhead |
| B: GStreamer | Rejected - 50MB deps, complex |
| C: libobs-rs | Rejected - immature (26 stars), no macOS |
| D: WebCodecs + JS muxer | Browser-only, not native |
| E: rav1e (AV1) | Too slow for real-time 1080p |
| F: openh264 + mp4 crate | Manual box manipulation required |
| G: openh264 + custom muxer | **Selected** → became Muxide |
| H: x264-rs | GPL licensing issues |
| I: Hardware encoders | Platform-specific complexity |
| J: Hybrid approach | Deferred to future |

### mp4e Evaluation

- **Version:** 1.0.5 on crates.io
- **GitHub:** 2 stars, 17 commits, 1 contributor
- **Status:** Broken docs.rs builds
- **Decision:** Rejected as too risky for production dependency

---

## 3. Muxide Development

### Current State: v0.1.0 Complete ✅

**Location:** `c:\Users\micha\repos\muxide\`

**Test Results:** 14/14 tests passing
```
cargo test → 14 passed, 0 failed
cargo doc → builds without warnings
cargo package --list → clean manifest
```

### Implemented Features

| Feature | Status |
|---------|--------|
| Basic MP4 with H.264 video | ✅ |
| AAC audio support (ADTS) | ✅ |
| Proper moov/mdat structure | ✅ |
| Keyframe seeking (stss box) | ✅ |
| 90 kHz media timescale | ✅ |
| Annex B → AVCC conversion | ✅ |
| Dynamic SPS/PPS from stream | ✅ |
| Audio/video interleaving | ✅ |
| MuxerStats on finish | ✅ |
| Descriptive error messages | ✅ |

### API Contract (What CrabCamera Will Consume)

```rust
use muxide::api::{Muxer, MuxerConfig};
use std::fs::File;

let file = File::create("recording.mp4")?;
let config = MuxerConfig::new(1920, 1080, 30.0);
let mut muxer = Muxer::new(file, config)?;

// Write frames (from openh264 encoder)
muxer.write_video(pts_secs, annex_b_data, is_keyframe)?;

// Optional audio
// muxer.write_audio(pts_secs, adts_data)?;

let stats = muxer.finish_with_stats()?;
println!("Wrote {} frames, {} bytes", stats.video_frames, stats.bytes_written);
```

### Key Invariants (Enforced by Muxide)

1. First video frame MUST be keyframe with SPS/PPS
2. Video PTS must be strictly increasing (monotonic)
3. Audio PTS must be non-decreasing
4. Audio cannot arrive before first video frame
5. No B-frames (v0.1.0 limitation)

---

## 4. Files Created This Session

### In CrabCamera (`c:\Users\micha\repos\crabcamera\`)

| File | Purpose |
|------|---------|
| `VIDEO_RECORDING_ARCHITECTURE_ANALYSIS.md` | 10-option comparison with weighted scoring |
| `MUXIDE_SPEC.md` | Full Muxide specification and roadmap |
| `crabcamera-muxide.code-workspace` | VS Code workspace combining both projects |
| `SESSION_SUMMARY_2025_12_15.md` | This document |

### In Muxide (`c:\Users\micha\repos\muxide\`)

Built by separate AI session, verified this session:
- `src/lib.rs`, `src/api.rs`, `src/muxer/mp4.rs`
- `docs/charter.md`, `docs/contract.md`
- `slice_ladder.md` (13 slices, all complete)
- 11 test files, all passing
- Fixtures for video/audio samples

---

## 5. Roadmap Items NOT Yet Implemented

### Muxide v0.1.x Quick Wins (Low Effort, High Value)

| Feature | Effort | Notes |
|---------|--------|-------|
| Metadata (`udta` box) | Low | Title, creation time |
| Fast-start (`moov` before `mdat`) | Medium | Critical for web playback |
| B-frame support (`ctts` box) | Medium | Future-proofs for other encoders |

### Muxide v0.2.0 (Fragmented MP4)

- fMP4 for streaming (HLS/DASH)
- Segment duration control
- CMAF compatibility

### Muxide v0.3.0 (Multi-Container)

- MKV/Matroska support
- WebM (VP9/Opus)
- Chapter markers

---

## 6. CrabCamera v0.5.0 Integration Path

Once Muxide is finalized and published:

1. Add `muxide` as dependency in `Cargo.toml`
2. Add `openh264` for encoding
3. Create `src/recording/` module with:
   - `VideoRecordingConfig` types
   - `RecordingSession` state machine
   - Integration with existing camera capture
4. Add CLI commands: `record start`, `record stop`
5. Expose via Python bindings

---

## 7. Open Questions for Next Session

### Muxide Strategy

1. **Licensing:** Open source (MIT/Apache-2.0) vs proprietary?
2. **Publication:** Publish to crates.io immediately or wait?
3. **Marketing:** Announce on r/rust, HN, or keep quiet?
4. **Monetization:** Dual-license? Support contracts? Keep internal only?

### Technical Decisions

1. Should fast-start be default or opt-in?
2. Priority: B-frames vs fragmented MP4?
3. Async support (tokio) - when/if?

---

## 8. Commands to Resume Work

```bash
# Open the combined workspace
code "c:\Users\micha\repos\crabcamera\crabcamera-muxide.code-workspace"

# Run Muxide tests
cd c:\Users\micha\repos\muxide && cargo test

# Build CrabCamera
cd c:\Users\micha\repos\crabcamera && cargo build
```

---

## 9. Key Context for Future AI Sessions

- **Muxide is NOT published** - still private, not on crates.io
- **CrabCamera v0.5.0** depends on Muxide being ready
- **mp4e exists** but was explicitly rejected (user said "I don't want to go with the two star GitHub thing")
- **Slice-gated engineering doctrine** was used for Muxide development
- **openh264** is the chosen encoder (patent-free, real-time capable)

---

*End of session summary. Open the workspace file to continue development.*
