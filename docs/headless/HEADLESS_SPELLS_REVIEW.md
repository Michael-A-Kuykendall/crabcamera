# HEADLESS_SPELLS.MD — Review Notes (AI-readable)

Source: `HEADLESS_SPELLS.MD`
Date: 2025-12-19
Scope: logical inconsistencies, ambiguous guarantees, and execution blockers only.

## Summary
- Count: 10 spells
- Severity overview:
  - BLOCKER: 1
  - RISK: 4
  - AMBIG: 7

## Findings

### [HSS-0001] BLOCKER — Dangling Markdown fence
- Location: file tail (after `#Spell: Mock_Backend_For_CI`)
- Problem: file ends with a bare code fence (```), which breaks Markdown parsing/rendering.
- Suggested change: remove the stray fence, or add the intended closing/opening fences consistently.
- Impact: doc consumption tools may drop/garble content after the fence.

---

## Delta Check (newer HEADLESS_SPELLS.MD pasted 2025-12-19)

### Status
- HSS-0001: still present in the pasted version (fixed in repo by removing the trailing fence).

### Previously ambiguous items now explicitly addressed in the pasted version
- Timeouts: `timeout_zero_is_poll` + `timeout_units_are_duration` added.
- Interruptibility: `internal_threads_interruptible` + `platform_backends_support_interrupt` added.
- Session state: `explicit_state_machine` + `calls_after_close_return_error` + `error_on_closed_or_stopped` added.
- Timestamps: `frame_timestamp_from_internal_monotonic_clock` added (replaces unrealistic device-PTS monotonicity).
- Drops: `drop_count_cumulative_per_session` + `drop_count_read_only` added.
- Audio gating: `audio_optional_feature_gated_compile_time` + `audio_failure_is_terminal` + `audio_enabled_if_compiled` added.
- Device ID stability: narrowed to within-process (`device_id_stable_within_process`, `stable_ids_within_listing`, `device_ids_not_persisted_across_runs`).
- Backend seam: `injectable_backend_entrypoint` added to mock spell.

### Remaining open clarifications (optional)
- Buffer representation: now constrained by `public_buffers_are_owned_and_normalized` and `owned_packed_layout`, but the exact concrete representation is still unspecified (OK if it’s intentionally left to implementation; otherwise specify `Vec<u8>` vs `Arc<[u8]>`).

---

### [HSS-0100] AMBIG — “platform_agnostic_types_only” vs buffer representation
- Location: `#Spell: Headless_Core_Surface` → `@HeadlessSurface` guarantee + `@Types` + `@Frame`/`@AudioPacket`
- Problem: `Frame`/`AudioPacket` imply owned buffers, but the exact buffer representation/layout isn’t constrained; backends tend to leak layout assumptions.
- Suggested clarification (pick one):
  - Specify buffer as owned `Vec<u8>` with explicit `format` describing layout; OR
  - Use `Arc<[u8]>` to reduce copies; OR
  - Prohibit zero-copy across public boundary and require normalized packed layouts.
- Impact: without this, “platform agnostic” can accidentally become “backend-shaped.”

### [HSS-0101] AMBIG — Stable device IDs are required by CLI but not guaranteed by core
- Location: `#Spell: CLI_Headless_Harness` → `@CLICommands` `! stable_device_ids`
- Problem: the core spells don’t define what “stable_device_ids” means.
- Suggested clarification: define stability scope explicitly:
  - Option A: stable only within a single process run/listing
  - Option B: stable across runs (requires persistent identity strategy)
  - Option C: stable across hotplug (hard; requires OS-level unique IDs)
- Impact: implementation can’t be correct without knowing the stability domain.

---

### [HSS-0200] AMBIG — Timeout semantics are not fully specified
- Location: `#Spell: Headless_Concurrency_Model` + all contracts using `(session, timeout) -> optional_*`
- Problem: “timeout” can mean poll vs wait; also multiple subsystems have different wait points.
- Suggested clarification:
  - Define `timeout=0` as non-blocking poll.
  - Define whether timeout applies to (a) waiting for next item, (b) waiting for shutdown join, or both.
  - Define units and maximum.
- Impact: subtle bugs + inconsistent behavior across platforms.

### [HSS-0201] RISK — “no_required_async_runtime” with internal threads + bounded waits
- Location: `#Spell: Headless_Concurrency_Model`
- Problem: bounded waits require a consistent shutdown signaling path; some backends don’t unblock reads cleanly.
- Suggested clarification: require a cancellation mechanism that unblocks waits (e.g., internal stop flag + backend-specific interrupt).
- Impact: deadlocks/hangs during stop/close are a real risk if backends can’t be interrupted.

---

### [HSS-0300] RISK — Monotonic timestamps may be unrealistic without normalization rule
- Location: `#Spell: Frame_Delivery_Contract` → `! frames_monotonic_timestamp`
- Problem: device/driver timestamps may jitter or reset; strict monotonicity may be violated.
- Suggested clarification:
  - Specify timestamp source: “timestamp is derived from an internal monotonic clock at receipt time” OR
  - Allow non-monotonic device timestamps but require monotonic `sequence` only.
- Impact: if left as-is, implementation must add normalization (clamp/repair), which is a design choice.

### [HSS-0301] AMBIG — `none_only_on_timeout` + error cases are good but edge states need definition
- Location: `#Spell: Frame_Delivery_Contract` + `#Spell: Audio_Delivery_Contract`
- Problem: unclear behavior on `SessionHandle` closed/stopped:
  - do calls return error immediately?
  - do they return `None`?
- Suggested clarification: define behavior for states {created, opened, started, stopped, closed}.
- Impact: caller code and tests can’t be consistent otherwise.

### [HSS-0302] AMBIG — Drop accounting semantics
- Location: `#Spell: Frame_Delivery_Contract` → `@BufferPolicy` `! drop_count_exposed`
- Problem: not specified whether drop count is per-session cumulative, per-read delta, or per-interval.
- Suggested clarification: define counter type and reset semantics.
- Impact: metrics/CLI output will vary and break consumers.

---

### [HSS-0400] AMBIG — Audio failure behavior vs video stability
- Location: `#Spell: Audio_Delivery_Contract`
- Problem: `! audio_failure_does_not_stop_video` + `! error_on_audio_failure` is coherent, but it’s unclear whether audio can recover or enters a terminal error state.
- Suggested clarification:
  - Option A: audio enters terminal failed state; future audio reads return error.
  - Option B: audio reads can recover; errors are transient.
- Impact: affects session state machine and how CLI reports errors.

### [HSS-0401] AMBIG — Feature-gating surface shape
- Location: `#Spell: Audio_Delivery_Contract` `! audio_optional_feature_gated`
- Problem: if audio is compiled out, what is the API surface?
  - Does `AudioPacket` type exist?
  - Does `audio_ops` exist but returns a fixed error?
- Suggested clarification: pick compile-time vs runtime optionality strategy.
- Impact: public API stability + consumer ergonomics.

---

### [HSS-0500] RISK — Mock backend requires an explicit backend injection seam
- Location: `#Spell: Mock_Backend_For_CI` depends on `PlatformCamera`
- Problem: for CI to run without hardware, there must be an injection point (trait/enum/feature) to swap real backend with mock.
- Suggested clarification: spell should require a single backend abstraction entry point.
- Impact: without this seam, mock backend can’t be used without invasive refactors.

## Minimal delta suggestions (NOT applied)
These are clarifications, not new features.
- Define timeout semantics (`timeout=0` poll, units, state behavior).
- Define timestamp source/normalization rule for monotonic timestamps.
- Define device ID stability scope (per-run vs cross-run).
- Define audio feature-gate strategy (compile-time surface vs runtime error).

## Notes
- No architectural changes proposed here; this is purely a consistency/clarity review for planner consumption.
