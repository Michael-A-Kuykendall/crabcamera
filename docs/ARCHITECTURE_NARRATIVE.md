# CrabCamera: Architecture Narrative & Project History

> A chronological account of how this codebase was conceived, what decisions were made, what bets were placed, which ones were called off, and what the result looks like today. Reconstructed from the full chat session history and release log.

---

## Origin: v0.1.0 — December 2024

CrabCamera started as a single sharp idea: **make camera access invisible inside Tauri applications**. Desktop apps built on Tauri had no clean, idiomatic way to reach USB cameras across Windows, macOS, and Linux. The plan was a Tauri 2.0 plugin that handled all three platforms via `nokhwa` as the unified capture backend.

The first release was deliberately thin:
- `initialize_camera_system` — bring up the platform backend
- `get_available_cameras` — enumerate devices
- `capture_single_photo` — grab a frame
- `start_camera_preview` / `stop_camera_preview` — manage stream lifecycle

The foundation was set up for correctness from the start: async/await throughout, zero unsafe code, memory-safe Rust, MIT/Apache-2.0 dual license. A mock system was included so tests could run without hardware.

---

## Early Ambition: v0.2.0 — January 2025

Before v0.1.0 had time to settle, a larger vision was already taking shape. v0.2.0 expanded scope aggressively, introducing the full professional camera control surface:

- **Manual focus, exposure, white balance, aperture, zoom, brightness, contrast, saturation, sharpness**
- **Burst capture** with configurable intervals
- **HDR exposure bracketing**
- **Focus stacking** for macro photography
- **RwLock over blocking Mutex** — the first internal architecture refinement, making the async usage model more correct
- **Memory pool system** for zero-copy frame buffers
- **Plant photography optimization** — a specialization that would prove prescient about the codebase's eventual positioning

A signature design principle emerged here: the `CameraControls` struct plus `CameraCapabilities` for runtime hardware detection. The idea was that professional photography workflows shouldn't require knowing which camera is connected; the library would probe capabilities and degrade gracefully.

---

## Windows Parity: v0.3.0 — January 2025

The cross-platform promise was put under pressure immediately. `nokhwa` gives uniform capture across platforms, but professional camera controls — the things v0.2.0 had just shipped — require talking to platform-native APIs. On Windows that means **MediaFoundation** (`IAMCameraControl`, `IAMVideoProcAmp`). 

v0.3.0 shipped a hybrid architecture that would become a template for the whole project:

```
┌─────────────────────────────────────────────┐
│               Tauri Command Layer            │
└─────────────────────────────────────────────┘
              │ CameraControls (unified API)
┌─────────────────────────────────────────────┐
│           PlatformCamera enum               │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │ nokhwa       │  │ MediaFoundationControls│ │
│  │ (capture)    │  │ (IAMCameraControl     │ │
│  │              │  │  IAMVideoProcAmp)     │ │
│  └──────────────┘  └──────────────────────┘ │
└─────────────────────────────────────────────┘
```

COM apartment threading and safe interface cleanup were addressed, and all Windows types were made `Send + Sync` to satisfy Tauri's async command handlers. Thanks to community reporters (`@thomasmoon`, `@eduramiba`), macOS permission dialog bugs were also tracked down — the Objective-C block syntax in `permissions.rs` was wrong and had to be replaced with `block::ConcreteBlock`.

---

## Production Photography System: v0.4.0 — October 2025

v0.4.0 was the first release that deserved the label "production-ready." Three systems were built:

### Quality Intelligence
Laplacian edge detection for blur analysis plus histogram analysis for exposure, with composite scoring, best-frame selection, and exponential backoff for retries. `capture_with_quality_retry()` became the recommended way to capture — you specify a quality floor and the library handles everything else.

### Configuration Management
A TOML-based runtime config system (`crabcamera.toml`) backed by serde-typed structs and a lazy-static global `RwLock`. Every section of the config (`camera`, `quality`, `storage`, `advanced`) could be read or updated independently via Tauri commands without a restart.

### Device Hot-Plug Recovery
A `DeviceMonitor` with async event channels, per-device reconnect state, and exponential backoff. The reconnect path was designed to be invisible to callers: `capture_single_photo()` would internally detect a missing device, wait for re-enumeration, re-create the camera instance, and retry — the user's code never needed to handle the disconnect.

### Computational Photography
Focus stacking via a full capture → align → merge pipeline: center-of-mass alignment with rotation/scale correction, Laplacian sharpness maps, and 5-level Gaussian pyramid blending to avoid seam artifacts.

**80 tests, all passing, 3,500+ new lines.** No breaking changes from v0.3.0.

---

## Audio & A/V Sync: v0.5.0 — December 2025

The single biggest architectural addition. Recording video without audio was useful; recording video with synchronized audio was what real-world apps needed.

The audio pipeline added 10 major components, with the hardest problems being:

1. **A/V drift** — solved with a shared `PTSClock` ensuring audio and video timestamps track a single source of truth, sync within ±40ms
2. **`cpal::Stream` not `Send`** — audio runs in a dedicated OS thread, communicating via a bounded `crossbeam-channel` of 256 frames
3. **PTS double-counting** — a critical bug where `buffer_start_pts` was being incorrectly updated after encoding, causing 2x-speed audio playback. Fixed by driving PTS from `samples_encoded` alone.

Opus encoding at 48kHz/128kbps with proper 960-sample (20ms) frame buffering, backed by `libopus_sys` FFI. The `start_recording()` command gained an optional `audio_device_id` parameter — opt-in, no breaking change.

8 proptest fuzz tests, criterion benchmarks at multiple resolutions, hardware validation with OBSBOT Tiny 4K + USB microphones.

---

## The WebRTC Chapter: v0.6.0–v0.6.3 — January 2026

This was the most expensive detour in the project's history, and the most instructive.

The thesis was attractive: if CrabCamera already captures video and audio with synchronized PTS, why not stream it via WebRTC? A browser could display a live camera feed from the desktop app — zero additional server required.

What shipped at v0.6.0 was technically impressive:
- Full WebRTC peer connection with SDP offer/answer
- RFC 6184-compliant H.264 NAL unit fragmentation
- RFC 7587 Opus RTP packetization  
- ICE candidate handling, data channel support
- Property-based testing of RTP invariants (1000+ cases per packetizer)
- 163 total tests, 94 WebRTC-specific

The production audit cycle between v0.6.0, v0.6.1, v0.6.2, and v0.6.3 resolved a cascade of issues: duplicate `use` imports causing build failures, incorrect SDP test assertions, an erroneous cleanup script (`cleanup_build_artifacts.sh`) that caused interactive prompts in CI, and Tauri 2.x documentation that still referenced the old `tauri.conf.json` plugin registration style.

**Then it was removed.**

---

## The Strategic Reset: v0.7.0 — January 2026

The decision to remove WebRTC was made deliberately and documented clearly. The core problem: WebRTC as a library consumer lived outside CrabCamera's value proposition. The implementation required network infrastructure (STUN/TURN), browser-side signaling coordination, and NAT traversal — none of which CrabCamera could own cleanly as a camera plugin. The ~1000 lines of WebRTC code were non-functional in real deployments and added complexity without delivering value to the people who actually used the library.

The removal was clean: all WebRTC dependencies, feature flags, test suites, and configuration options were deleted. The library's core identity was restated: **camera capture and recording excellence, nothing else**.

This set the stage for everything that came after being higher quality.

---

## Frame Callbacks: v0.7.1 — January 2026

A community contribution (inspired by PR #8 from `@saurL`) that unlocked a real-time use case. `set_frame_callback` lets callers register a closure that fires as each frame arrives — no polling, no buffering — backed by `Arc<Mutex<>>` for thread-safe callback registration across all three platforms.

---

## The Invariant Era: v0.8.x — January–May 2026

The most recent architectural phase changed how correctness is expressed in the codebase itself.

### Invariant Superhighway (v0.8.0/0.8.1)

What was internally called "FeedMe" methodology was formalized and renamed to the **Invariant PPT** (Publish-Prove-Test) framework. Two new macros drive runtime correctness:

- `assert_invariant!` — enforces architectural contracts in debug builds, logs to a ring-buffer crash dump
- `assert_performance_invariant!` — panics if operations like frame analysis exceed a latency budget

These aren't just assertions; they form a "black box recorder" pattern. If the system crashes, the invariant ring buffer tells you what contracts were last checked and when they last passed.

**Smart Trigger** built on top of invariant guarantees: automated capture that waits until quality is stable (not blurry, good exposure, persistent over a configurable duration) before firing. Intelligent timeout handling ensures a shot is never permanently missed.

### Type Safety Hardening (v0.8.1)

`QualityReport` was refactored to use `Option<T>` for metric components, eliminating the "invalid partial result" class of crashes. Focus stacking gained strict dimension-match contracts.

Issue #9 (Tauri v2 frontend "undefined invoke" error) was fixed in documentation — the registration pattern shown in the quick-start guide was outdated.

### Image Analysis CLI (v0.8.3)

`crabcamera-cli` gained an `analyze-image` command for scoring static image files on blur and exposure. This makes quality algorithm development testable against static datasets without requiring a live camera.

The most recent commit on `main` (`9548315`) is a Hard Mode audit fixing clippy warnings, documentation gaps, and test coverage. The commit just before it (`3af7760`) eliminated all remaining platform stubs — `crab-160`.

---

## Current Architecture Map

```
crabcamera (lib)
├── src/lib.rs                 — feature-gated module tree
├── src/platform/              — PlatformCamera trait + Windows/macOS/Linux impls
│   ├── windows.rs             — nokhwa + MediaFoundation hybrid
│   ├── macos.rs               — nokhwa + AVFoundation permissions
│   └── linux.rs               — nokhwa + v4l2 group checks
├── src/commands/              — Tauri command handlers (one file per domain)
│   ├── init.rs                — camera init, diagnostics
│   ├── capture.rs             — single/burst/quality-retry capture
│   ├── advanced.rs            — professional controls
│   ├── recording.rs           — A/V recording lifecycle
│   ├── audio.rs               — audio device commands
│   ├── quality.rs             — blur/exposure/scoring
│   ├── focus_stack.rs         — focus stacking pipeline
│   ├── config.rs              — TOML config CRUD
│   ├── device_monitor.rs      — hot-plug events
│   └── permissions.rs         — OS permission gating
├── src/recording/             — H.264 encoder, Opus encoder, muxer
├── src/audio/                 — cpal capture, PTSClock, sync buffer
├── src/focus_stack/           — capture → align → merge pipeline
├── src/quality/               — blur, exposure, validator, smart trigger
├── src/headless/              — headless capture session (CLI use)
├── src/invariant_ppt.rs       — assert_invariant!, black box recorder
├── src/timing/                — nanosecond PTS clock
├── src/config.rs              — serde TOML config with lazy RwLock global
├── src/permissions.rs         — PermissionStatus + platform dispatch
├── src/errors.rs              — unified CrabCameraError enum
└── src/types.rs               — CameraFrame, CameraControls, CameraFormat, etc.
```

**Feature flags:**
- `tauri` — enables the Tauri command layer (default on)
- `recording` — H.264 video recording via muxide
- `audio` — Opus audio capture and encoding
- `headless` — CLI/server operation without a Tauri host
- `webrtc` — **removed in v0.7.0**, flag retained as no-op for compatibility

---

## Open GitHub Issues (as of 2026-05-25)

### Issue #11: "Using without Tauri" (opened 2026-02-21)
A user asks whether CrabCamera can be used in non-Tauri Rust applications. The `headless` feature flag exists precisely for this purpose — it exposes a `HeadlessSession` API that has no Tauri dependency. This is a documentation gap more than a missing feature.

### Issue #10: "crabcamera.initialize_camera_system not allowed. Plugin not found" (opened 2026-02-04)
A user followed the quick-start and received a plugin-not-found error when calling `plugin:crabcamera|initialize_camera_system`. The root cause is that the Tauri 2.x plugin registration syntax changed between v1 and v2 — the crate must be added to the Tauri builder with `.plugin(crabcamera::init())`. This was documented in v0.6.3 (Tauri 2.x doc fixes) and again closed by the v0.8.1 fix for issue #9, but the symptom in #10 is slightly different (invoke path resolution, not frontend `invoke` availability). Needs a clear comment pointing to the v0.8.x quick-start with Tauri 2.x builder syntax.

---

## Beads Work Queue

Two open items under the `crab-rwx` epic (P2):

- **`crab-rwx` (epic):** Rigorous audit — make the codebase teachable, production-ready, and idiomatic Rust. Eliminate all shortcuts, unwraps, and poor docs.
- **`crab-rwx.1` (task):** Child task of the epic, not yet described.

The latest commit's description ("Hard Mode audit fixes") suggests this epic is actively being worked in the current session. The platform stub elimination (`crab-160`) landed just above it in the log.

---

## Summary: Where We Are

CrabCamera is at **v0.8.3**, a stable, focused library for cross-platform camera capture inside Tauri 2.x applications and headless Rust programs. The core loop is:

1. **Capture** — single frame, burst, quality-gated retry, or smart trigger
2. **Record** — H.264 video + Opus audio, muxed with precise PTS sync
3. **Analyze** — blur and exposure scoring, real-time or against static files
4. **Compose** — focus stacking, HDR bracketing
5. **Verify** — invariant checks throughout, black-box ring buffer for post-mortem

The WebRTC experiment demonstrated that the project has enough mass to build complex systems, and enough discipline to remove them when they don't fit. The invariant framework shows a maturing engineering culture that cares about provability, not just functionality.

The `crab-rwx` audit epic is the active frontier: making all of this not just work, but be understandable and idiomatic enough to serve as a teaching artifact.
