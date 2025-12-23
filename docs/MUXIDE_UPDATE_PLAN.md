# 🦀 CrabCamera Muxide v0.1.2 Update Plan

**Date:** December 20, 2025  
**Goal:** Update CrabCamera from Muxide v0.1.1 to v0.1.2 for enhanced error handling, metadata support, and future compatibility.

## 📋 Update Checklist

### 1. Core Dependencies
- [x] Update `Cargo.toml` to use `muxide = "0.1.2"`
- [x] Run `cargo update` to fetch new version
- [x] Verify no breaking changes in `cargo check`

### 2. API Migration (src/recording/recorder.rs)
- [ ] Replace old `Muxer::new()` constructor with `MuxerBuilder` pattern
- [ ] Update metadata creation to use new `Metadata` API with language support
- [ ] Ensure audio configuration still works with new builder
- [ ] Test that existing video recording still functions

### 3. Enhanced Metadata Support
- [ ] Add language metadata (`"und"` for undefined) to all recordings
- [ ] Ensure creation time is still set correctly
- [ ] Verify title metadata works as before

### 4. Error Handling Improvements
- [ ] Test new detailed ADTS error messages for audio debugging
- [ ] Verify enhanced video frame validation works
- [ ] Check that Opus packet validation is more robust

### 5. Testing & Validation
- [ ] Run existing test suite (`cargo test`) - all should pass
- [ ] Test video-only recording (1920x1080 H.264)
- [ ] Test audio+video recording (Opus + H.264)
- [ ] Verify MP4 files play correctly in media players
- [ ] Test metadata (title, creation time, language) in output files

### 6. Optional Future Features
- [ ] Consider adding VP9 codec support to RecordingConfig
- [ ] Evaluate fragmented MP4 support for streaming use cases
- [ ] Test CLI tool integration for debugging workflows

## 🔧 Files to Modify

1. `Cargo.toml` - Version bump
2. `src/recording/recorder.rs` - API migration
3. `tests/recording_props.rs` - Update any hardcoded version expectations

## 🧪 Test Commands

```bash
# Update and build
cargo update
cargo build --features recording,audio

# Run tests
cargo test --features recording,audio

# Manual testing
cargo run --example record_video --features recording
cargo run --example functional_test --features recording,audio
```

## ✅ Success Criteria

- [ ] All existing tests pass
- [ ] Video recording produces valid MP4 files
- [ ] Audio+video sync works correctly
- [ ] Metadata is properly embedded
- [ ] No performance regression
- [ ] Error messages are more helpful for debugging

## 🎯 Expected Benefits

- Better error diagnostics for audio/video issues
- More robust frame validation
- Enhanced metadata support
- Future-proofing for VP9/AV1 codecs
- Access to improved CLI tooling

---

**Status:** Ready to execute  
**Estimated Time:** 30-45 minutes  
**Risk Level:** Low (API is backward compatible)</content>
<parameter name="filePath">MUXIDE_UPDATE_PLAN.md