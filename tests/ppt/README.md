# CrabCamera Property-Based Testing (PPT) with Invariant

This directory contains property-based tests using the Invariant PPT system for rigorous contract testing.

## Testing Philosophy

### Three Phases of Testing

1. **Unit Tests** (`tests/*.rs`)
   - Pure function testing
   - Mock camera backends
   - Fast, deterministic

2. **Functional Tests** (`examples/functional_test.rs`)
   - Real hardware integration
   - Camera warmup sequences
   - Platform-specific behavior

3. **Property-Based Tests** (PPT - this directory)
   - Contract invariants
   - Fuzz testing with valid inputs
   - State machine verification

## Invariants to Test

### Camera System Invariants

```
INVARIANT camera_list_consistency:
  ∀ camera ∈ list_cameras():
    camera.id ≠ ∅ ∧
    camera.name.len() > 0 ∧
    is_valid_camera_id(camera.id)
```

```
INVARIANT format_bounds:
  ∀ format ∈ camera.supported_formats():
    format.width ∈ [1, 8192] ∧
    format.height ∈ [1, 4320] ∧
    format.fps ∈ [0.1, 240.0]
```

```
INVARIANT frame_data_size:
  ∀ frame ∈ capture_frame():
    frame.data.len() == frame.width * frame.height * 3 ||
    is_compressed_frame(frame)  // MJPEG/etc
```

### Recording Invariants

```
INVARIANT recorder_frame_count:
  ∀ recorder: Recorder
    recorder.start() →
    (n writes) →
    recorder.finish() →
    stats.video_frames == n
```

```
INVARIANT muxer_bytes_monotonic:
  ∀ write_i in recorder:
    stats_before.bytes_written ≤ stats_after.bytes_written
```

```
INVARIANT h264_annex_b:
  ∀ encoded_frame ∈ encoder.encode():
    starts_with(encoded_frame, [0,0,0,1]) ∨
    starts_with(encoded_frame, [0,0,1])
```

### State Machine Invariants

```
STATE_MACHINE Camera {
  states: [Disconnected, Idle, Streaming, Recording]
  
  transitions:
    Disconnected → Idle     : new()
    Idle → Streaming        : start_stream()
    Streaming → Idle        : stop_stream()
    Streaming → Recording   : start_recording()
    Recording → Streaming   : stop_recording()
    Idle → Disconnected     : drop()
}

INVARIANT: No capture_frame() in Idle or Disconnected state
INVARIANT: No start_recording() unless Streaming
```

## Test Structure

```
tests/ppt/
├── README.md           # This file
├── camera_props.rs     # Camera system properties
├── recording_props.rs  # Recording module properties  
├── format_props.rs     # Format validation properties
└── state_machine.rs    # State transition properties
```

## Running PPT Tests

```bash
# Run all property-based tests
cargo test --test ppt -- --nocapture

# Run with specific seed for reproducibility
PROPTEST_SEED=12345 cargo test --test ppt

# Run with more cases (default: 256)
PROPTEST_CASES=1000 cargo test --test ppt
```

## Integration with Invariant

The Invariant PPT system provides:

1. **Shrinking** - When a test fails, find minimal failing input
2. **Regression** - Store failing cases for future runs
3. **Coverage** - Track which invariants have been exercised
4. **Fuzzing** - Generate random but valid inputs

## Example Property Test

```rust
use proptest::prelude::*;

proptest! {
    /// Recording always produces valid MP4
    #[test]
    fn recording_produces_valid_output(
        width in 160u32..1920,
        height in 120u32..1080,
        fps in 15.0f32..60.0,
        frames in 1usize..100,
    ) {
        let config = RecordingConfig::new(width, height, fps);
        let mut recorder = Recorder::new(&temp_file(), config)?;
        
        for _ in 0..frames {
            let rgb = random_rgb_frame(width, height);
            recorder.write_rgb_frame(&rgb, width, height)?;
        }
        
        let stats = recorder.finish()?;
        
        // INVARIANT: Frame count matches
        prop_assert_eq!(stats.video_frames, frames as u64);
        
        // INVARIANT: Bytes written is reasonable
        prop_assert!(stats.bytes_written > 0);
        prop_assert!(stats.bytes_written < width * height * frames * 3);
    }
}
```
