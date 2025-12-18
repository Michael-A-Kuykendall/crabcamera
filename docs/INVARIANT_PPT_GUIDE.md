# Testing Guide

CrabCamera’s tests focus on stable *behavioral invariants*: properties that must remain true across platforms and refactors.

The suite is a mix of:

- Unit tests in `src/`
- Integration tests in `tests/`
- Property-based tests using `proptest`

## Property tests

Property tests generate many small inputs and shrink failures to a minimal counterexample.

Primary file:

- `tests/recording_props.rs`

Notes:

- The format-related properties run on every build.
- Recording/encoder properties are currently compiled out (guarded by `#[cfg(any())]`) and do not run in CI.
	This is intentional while the recording stack is still experimental.

## Running the suite

```bash
# All tests
cargo test

# A specific property test file
cargo test --test recording_props

# Increase proptest cases (when applicable)
PROPTEST_CASES=1000 cargo test --test recording_props
```

## About recording invariants

The `tests/recording_props.rs` file contains a larger set of recording/encoder invariants (H.264 Annex B, recorder stats, size bounds).
Those are currently disabled via `#[cfg(any())]` to avoid pulling in heavier recording dependencies by default.

If you want those invariants to run as part of the automated suite, the intended next step is to put the recording module behind a real feature flag (e.g. `recording`) and switch the guard in `tests/recording_props.rs` accordingly.

## Hardware-dependent tests

Some tests may be marked `#[ignore]` when they require physical camera hardware.

```bash
cargo test -- --ignored
```

## Adding a new invariant

1. Write down the invariant in a sentence (what must always be true?).
2. Encode it as a unit/integration test assertion.
3. If it’s input-sensitive, add a `proptest` case that explores edge values.

## References

- proptest: https://docs.rs/proptest
