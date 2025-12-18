# GitHub Copilot Instructions - CrabCamera

## 🔮 SORCERY-DRIVEN DEVELOPMENT

**We use Sorcery notation for architectural intent.** All implementation MUST follow sealed spellbooks.

### Current Active Spellbook
**`docs/AUDIO_SPELLBOOK.md`** - Audio Recording v0.5.0 ✅ **COMPLETE**

### Spell Execution Rules
1. **READ THE SPELL FIRST** - Before implementing, read the spell's guarantees (`!`), exclusions (`-`), and dependencies (`>`)
2. **NO VIBE CODING** - Every line of code must trace back to a spell requirement
3. **SPELLS ARE SEALED** - Don't add features not in the spell. Don't skip requirements.
4. **EXECUTE IN ORDER** - Spells have dependencies (`>`). Follow the DAG.

### Spell Progress (v0.5.0 Audio) - ALL COMPLETE ✅
- [x] #AudioDeviceEnumerate → `src/audio/device.rs`
- [x] #AudioPTSClock → `src/audio/clock.rs`
- [x] #AudioCapturePCM → `src/audio/capture.rs`
- [x] #AudioEncodeOpus → `src/audio/encoder.rs`
- [x] #RecorderIntegrateAudio → `src/recording/recorder.rs`
- [x] #AVSyncPolicy → `src/recording/recorder.rs`
- [x] #TauriAudioCommands → `src/commands/audio.rs`, `src/commands/recording.rs`
- [x] #AudioErrorRecovery → `src/recording/recorder.rs`, `src/commands/recording.rs`
- [x] #RecordingTests_AV → `tests/av_integration.rs`
- [x] #CargoAudioGating → `Cargo.toml`

---

## 🎯 Project Identity

**CrabCamera** is a production-ready desktop camera plugin for Tauri applications.
- **Mission:** Invisible camera infrastructure - drop it in, it works
- **License:** MIT (no GPL contamination)
- **Philosophy:** Pure Rust, zero FFmpeg, single binary

## 🔗 Related Projects

**Muxide** (private, `../muxide/`) is our custom MP4 muxer - a competitive moat that eliminates external dependencies. CrabCamera consumes Muxide via path dependency for video recording.

## ⚠️ Critical Rules

1. **FOLLOW THE SPELLBOOK** - No implementation without a spell
2. **READ BEFORE WRITE** - Always read files before editing
3. **No FFmpeg** - Never add FFmpeg, GStreamer, or libav dependencies
4. **No GPL** - Only MIT/Apache-2.0 compatible dependencies
5. **Muxide is Private** - Don't suggest publishing it to crates.io
6. **Pure Rust** - Avoid C bindings where possible

## 🏗️ Architecture

```
CrabCamera (Tauri Plugin)
    ├── nokhwa (camera capture)
    ├── openh264 (video encoding) [v0.5.0]
    ├── libopus_sys (audio encoding) [v0.5.0]
    ├── cpal (audio capture) [v0.5.0]
    └── muxide (MP4 muxing, private) [v0.5.0]
```

## 📁 Key Files

- `docs/AUDIO_SPELLBOOK.md` - **ACTIVE SPELLBOOK** (source of truth for v0.5.0)
- `src/audio/` - Audio module (spells implemented here)
- `src/commands/` - Tauri command handlers
- `src/platform/windows/` - Windows-specific code

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
