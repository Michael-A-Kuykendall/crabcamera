# Muxide: A Pure Rust Video Muxer

**Tagline:** *"Just give me frames. I'll make the video."*

---

## The Problem

The Rust ecosystem has encoders (openh264, rav1e, x264) and container parsers (mp4, matroska), but **no simple, unified muxer** that:

1. Accepts encoded frames from any source
2. Handles audio/video interleaving automatically
3. Supports multiple containers (MP4, MKV, WebM)
4. Just works without understanding container internals

**Current pain:**
```rust
// Today: You need to understand MP4 box structure
let mut mp4 = Mp4Writer::new(...);
let trak = mp4.add_track(TrackConfig { ... box configurations ... });
// Manual sample timing, chunk layout, stts/stss/stsz entries...
```

**What should exist:**
```rust
// Muxide: Just give me frames
let mut mux = Muxide::mp4("output.mp4")
    .video(H264, 1920, 1080, 30.0)
    .audio(AAC, 48000, 2)
    .build()?;

mux.write_video(pts, data, is_keyframe)?;
mux.write_audio(pts, data)?;
mux.finish()?;
```

---

## Market Position

### Existing Crates & Their Lanes

| Crate | Focus | Gap |
|-------|-------|-----|
| `mp4` | Low-level MP4 box read/write | Requires container expertise |
| `matroska` | MKV parsing | Read-only, no writing |
| `webm-iterable` | WebM parsing | Read-only |
| `ffmpeg-sidecar` | Full ffmpeg CLI | 80MB binary, process overhead |
| `gstreamer-rs` | Pipeline-based media | Massive dependency, complex |

### Muxide's Lane

```
┌─────────────────────────────────────────────────────────────────┐
│                        COMPLEXITY SPECTRUM                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Low-level boxes    Simple muxing      Full media framework     │
│  (mp4 crate)        (MUXIDE)           (gstreamer/ffmpeg)       │
│       │                 │                      │                 │
│       ▼                 ▼                      ▼                 │
│  ┌─────────┐       ┌─────────┐          ┌─────────────┐         │
│  │ Manual  │       │ "Just   │          │ Pipelines,  │         │
│  │ boxes   │       │ works"  │          │ filters,    │         │
│  │ timing  │       │ API     │          │ everything  │         │
│  └─────────┘       └─────────┘          └─────────────┘         │
│                         ▲                                        │
│                         │                                        │
│                    MUXIDE LIVES HERE                            │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Muxide is the "middle ground" that doesn't exist:**
- More capable than raw box manipulation
- Less complex than full media frameworks
- Pure Rust, no C dependencies
- Single responsibility: muxing encoded frames into containers

---

## Core Design Principles

### 1. Encoder Agnostic
Muxide doesn't care where your encoded data comes from:
```rust
// From openh264
let nalu = openh264_encoder.encode(&frame)?;
mux.write_video(pts, &nalu, keyframe)?;

// From rav1e
let packet = rav1e_encoder.receive_packet()?;
mux.write_video(pts, &packet.data, packet.frame_type.is_key())?;

// From hardware encoder via FFI
let data = nvenc_encode(&frame)?;
mux.write_video(pts, &data, is_idr)?;

// From WebCodecs (via wasm-bindgen)
mux.write_video(pts, &encoded_chunk.data(), chunk.type == "key")?;
```

### 2. Container Agnostic (Same API)
```rust
// MP4
let mux = Muxide::mp4("out.mp4").video(H264, ...).build()?;

// MKV
let mux = Muxide::mkv("out.mkv").video(H264, ...).build()?;

// WebM
let mux = Muxide::webm("out.webm").video(VP9, ...).build()?;

// Fragmented MP4 (for streaming)
let mux = Muxide::fmp4("out.mp4").video(H264, ...).build()?;
```

### 3. Handles the Hard Stuff Automatically
- **Interleaving:** Audio and video samples properly ordered
- **Timing:** PTS/DTS calculation from frame timestamps
- **Keyframe indexing:** Seek tables built automatically
- **Codec metadata:** SPS/PPS (H.264), sequence headers extracted

### 4. Streaming-Friendly
```rust
// Write to file
let mux = Muxide::mp4(File::create("out.mp4")?).build()?;

// Write to memory buffer
let mux = Muxide::mp4(Vec::<u8>::new()).build()?;

// Write to async stream
let mux = Muxide::mp4(tokio::io::BufWriter::new(file)).build()?;

// Fragmented MP4 with segment callbacks
let mux = Muxide::fmp4_segments(|segment| {
    // Each segment ready for HLS/DASH
    upload_to_cdn(segment)?;
})?;
```

---

## API Design

### Builder Pattern
```rust
use muxide::{Muxide, VideoCodec, AudioCodec, Profile};

let mut mux = Muxide::mp4("recording.mp4")
    // Video track
    .video(VideoCodec::H264)
        .dimensions(1920, 1080)
        .framerate(30.0)
        .profile(Profile::High)
        .done()
    // Audio track (optional)
    .audio(AudioCodec::AAC)
        .sample_rate(48000)
        .channels(2)
        .done()
    // Metadata (optional)
    .metadata()
        .title("My Recording")
        .creation_time(SystemTime::now())
        .done()
    .build()?;
```

### Writing Frames
```rust
// Video frame
mux.write_video(VideoFrame {
    pts: Duration::from_millis(0),
    dts: None,  // Calculated if not provided
    data: &encoded_nalu,
    keyframe: true,
})?;

// Or simplified
mux.write_video_simple(pts_ms, &data, is_keyframe)?;

// Audio frame
mux.write_audio(AudioFrame {
    pts: Duration::from_millis(0),
    data: &encoded_aac,
})?;
```

### Finalization
```rust
// Must call to write container trailer
let stats = mux.finish()?;

println!("Wrote {} video frames, {} audio frames", 
    stats.video_frames, stats.audio_frames);
println!("Duration: {:?}", stats.duration);
println!("File size: {} bytes", stats.bytes_written);
```

---

## Container Support Roadmap

### Phase 1: MP4 (v0.1.0)
- [x] Basic MP4 with H.264 video
- [x] AAC audio support
- [x] Proper moov/mdat structure
- [x] Keyframe seeking (stss box)
- [ ] B-frame support (ctts box)

### Phase 2: Fragmented MP4 (v0.2.0)
- [ ] fMP4 for streaming
- [ ] Segment duration control
- [ ] CMAF compatibility
- [ ] Low-latency mode

### Phase 3: MKV/WebM (v0.3.0)
- [ ] Matroska container
- [ ] WebM subset (VP8/VP9/Opus)
- [ ] Chapter markers
- [ ] Subtitle tracks

### Phase 4: Advanced (v0.4.0+)
- [ ] MOV (QuickTime) compatibility
- [ ] ProRes/DNxHD for pro workflows
- [ ] HDR metadata (HDR10, Dolby Vision)
- [ ] Multi-track (multiple video angles)

---

## Codec Support

### Video Codecs

| Codec | Container Support | Priority |
|-------|-------------------|----------|
| H.264/AVC | MP4, MKV, MOV | ✅ P0 |
| H.265/HEVC | MP4, MKV, MOV | P1 |
| VP9 | WebM, MKV | P1 |
| AV1 | MP4, WebM, MKV | P2 |
| VP8 | WebM | P3 |

### Audio Codecs

| Codec | Container Support | Priority |
|-------|-------------------|----------|
| AAC | MP4, MKV, MOV | ✅ P0 |
| Opus | WebM, MKV | P1 |
| MP3 | MP4, MKV | P2 |
| FLAC | MKV | P3 |
| PCM | All | P2 |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                          Muxide                                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Public API Layer                      │    │
│  │  Muxide::mp4() / ::mkv() / ::webm() / ::fmp4()          │    │
│  │  .video() / .audio() / .metadata()                       │    │
│  │  .write_video() / .write_audio() / .finish()            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                   Interleaving Engine                    │    │
│  │  - Sample queue management                               │    │
│  │  - PTS/DTS ordering                                      │    │
│  │  - Chunk boundaries                                      │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌──────────────┬──────────────┬──────────────┐                 │
│  │  MP4 Backend │  MKV Backend │ WebM Backend │                 │
│  │  (isobmff)   │  (ebml)      │ (ebml)       │                 │
│  └──────────────┴──────────────┴──────────────┘                 │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Write Abstraction                     │    │
│  │  impl Write (File, Vec, AsyncWriter, etc.)              │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Internal Modules

```
muxide/
├── src/
│   ├── lib.rs              # Public API, re-exports
│   ├── builder.rs          # Builder pattern implementation
│   ├── muxer.rs            # Core Muxer trait and engine
│   ├── interleave.rs       # Audio/video interleaving logic
│   ├── timing.rs           # PTS/DTS calculations
│   │
│   ├── containers/
│   │   ├── mod.rs
│   │   ├── mp4/
│   │   │   ├── mod.rs
│   │   │   ├── boxes.rs    # ftyp, moov, mdat, etc.
│   │   │   ├── track.rs    # trak, mdia, minf, stbl
│   │   │   └── fmp4.rs     # Fragmented MP4 extension
│   │   ├── mkv/
│   │   │   ├── mod.rs
│   │   │   └── ebml.rs     # EBML primitives
│   │   └── webm/
│   │       └── mod.rs      # WebM-specific constraints
│   │
│   ├── codecs/
│   │   ├── mod.rs
│   │   ├── h264.rs         # NAL unit parsing, SPS/PPS
│   │   ├── hevc.rs         # HEVC/H.265 support
│   │   ├── av1.rs          # OBU parsing
│   │   ├── aac.rs          # ADTS header handling
│   │   └── opus.rs         # Opus packet handling
│   │
│   └── error.rs            # Error types
│
├── Cargo.toml
├── README.md
└── examples/
    ├── simple_mp4.rs
    ├── with_audio.rs
    ├── fragmented_streaming.rs
    └── from_openh264.rs
```

---

## Differentiation Strategy

### 1. Name: "Muxide"
- Mux + Oxide (Rust reference)
- Easy to say, easy to remember
- Unique in crates.io search

### 2. Documentation-First
- Every public API documented with examples
- Cookbook for common scenarios
- Comparison with alternatives

### 3. Zero Unsafe by Default
```toml
[features]
default = []
unsafe-optimizations = []  # Opt-in only
```

### 4. Excellent Errors
```rust
// Not this:
Error: InvalidData

// This:
Error: InvalidVideoFrame {
    reason: "H.264 NAL unit missing SPS - first frame must be keyframe with SPS/PPS",
    frame_number: 0,
    suggestion: "Ensure encoder outputs SPS/PPS with first keyframe",
}
```

### 5. Minimal Dependencies
```toml
[dependencies]
# Core (always)
thiserror = "1.0"
bytes = "1.0"

# Optional containers
mp4-atoms = { version = "0.1", optional = true }
ebml-writer = { version = "0.1", optional = true }

[features]
default = ["mp4"]
mp4 = ["mp4-atoms"]
mkv = ["ebml-writer"]
webm = ["ebml-writer"]
all-containers = ["mp4", "mkv", "webm"]
```

---

## Example: Complete Recording with openh264

```rust
use muxide::{Muxide, VideoCodec};
use openh264::encoder::{Encoder, EncoderConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up encoder
    let config = EncoderConfig::new(1920, 1080);
    let mut encoder = Encoder::with_config(config)?;
    
    // Set up muxer
    let mut mux = Muxide::mp4("recording.mp4")
        .video(VideoCodec::H264)
            .dimensions(1920, 1080)
            .framerate(30.0)
            .done()
        .build()?;
    
    // Recording loop
    let mut frame_num = 0u64;
    for raw_frame in camera_frames() {
        // Encode
        let yuv = convert_to_yuv(&raw_frame);
        let bitstream = encoder.encode(&yuv)?;
        
        // Mux
        let pts = Duration::from_secs_f64(frame_num as f64 / 30.0);
        let is_keyframe = bitstream.frame_type().is_idr();
        
        mux.write_video_simple(pts, bitstream.as_slice(), is_keyframe)?;
        
        frame_num += 1;
    }
    
    // Finalize
    let stats = mux.finish()?;
    println!("Recorded {} frames in {:?}", stats.video_frames, stats.duration);
    
    Ok(())
}
```

---

## Competitive Analysis

| Feature | muxide | mp4 crate | gstreamer-rs | ffmpeg |
|---------|--------|-----------|--------------|--------|
| Simple API | ✅ | ❌ | ❌ | ❌ |
| Pure Rust | ✅ | ✅ | ❌ | ❌ |
| Multi-container | ✅ | ❌ (MP4 only) | ✅ | ✅ |
| Bundle size | ~500KB | ~200KB | ~50MB | ~80MB |
| Learning curve | Low | High | Very High | High |
| Streaming (fMP4) | ✅ | ❌ | ✅ | ✅ |
| Zero unsafe | ✅ | ⚠️ | ❌ | ❌ |

---

## Go-to-Market

### Phase 1: Prove It Works (Month 1)
1. Ship v0.1.0 with MP4 + H.264 + AAC
2. Use in CrabCamera v0.5.0 as dogfood
3. Publish to crates.io
4. Write announcement blog post

### Phase 2: Build Credibility (Month 2-3)
1. Add fragmented MP4 (streaming use case)
2. Contribute examples to openh264, rav1e docs
3. Post to r/rust, Hacker News
4. Respond to issues within 24 hours

### Phase 3: Expand (Month 4+)
1. Add MKV/WebM support
2. Seek contributors for specialized codecs
3. Integration guides for popular encoders
4. Consider async runtime support

---

## Success Metrics

| Metric | 3 Month | 6 Month | 12 Month |
|--------|---------|---------|----------|
| GitHub Stars | 100 | 500 | 2,000 |
| Crates.io Downloads | 1,000 | 10,000 | 50,000 |
| Dependent Crates | 5 | 20 | 100 |
| Contributors | 1 | 3 | 10 |

---

## Why You Should Build This

1. **CrabCamera needs it anyway** - You're going to build this muxing logic regardless
2. **First-mover advantage** - No one else is doing "simple Rust muxer"
3. **Ecosystem contribution** - Gives back to Rust community
4. **Marketing synergy** - "From the makers of CrabCamera"
5. **Portfolio piece** - Demonstrates systems programming expertise

---

## Next Steps

1. [ ] Reserve `muxide` on crates.io
2. [ ] Create GitHub repo `muxide/muxide` or `yourusername/muxide`
3. [ ] Implement MP4 + H.264 MVP (~1 week)
4. [ ] Integrate into CrabCamera v0.5.0
5. [ ] Write README with compelling examples
6. [ ] Publish v0.1.0

---

*"The best time to plant a tree was 20 years ago. The second best time is now."*

**The Rust ecosystem needs Muxide. You can be the one who builds it.**
