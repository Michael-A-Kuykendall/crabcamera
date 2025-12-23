# Sigil Tests

This tool is intended to act as a **sigil**: a small mechanical gate that checks whether an invocation (recast from the built artifact) covers the canonical spell’s declared constraints.

## What the sigil guarantees

- Deterministic parsing of the standard glyph inventory (`#Spell:` blocks, `@Entities`, and `^ ! - ~ > : ?` lines).
- Deterministic comparison of **declared constraint strings** (no semantic inference).
- Fail-fast on unknown leading glyphs (nonstandard syntax cannot silently pass).

## What the sigil does not do

- It does not “prove” properties like no-network or no-filesystem-writes; it only checks whether those constraints were declared and matched.

## Running the sigil test suite

From repo root:

- `cargo test --manifest-path tools/glyph-verify/Cargo.toml`

Key edge cases are covered by integration tests in `tools/glyph-verify/tests/sigil.rs` and fixtures in `tools/glyph-verify/tests/fixtures/`.
