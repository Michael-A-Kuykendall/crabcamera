# glyph-verify

A tiny reference parser + comparator for Sorcery Glyph.

This is intentionally minimal:
- Parses `#Spell:` blocks and the `#^@:!~->?` lines under them.
- Compares a canonical spellbook against an invocation and returns `BOUND`/`NOT BOUND`.
- Treats `!/-/>/:` payloads as declared constraints to match (no semantic inference).

## Sigil

This tool can be used as a **sigil**: after building an artifact, recast it into an invocation in the same glyph system and verify it matches the canonical spell.

Sigil test suite: see [tools/glyph-verify/SIGIL_TEST.md](tools/glyph-verify/SIGIL_TEST.md) and `cargo test`.

## Usage

From the repo root:

- Build: `cargo build --manifest-path tools/glyph-verify/Cargo.toml`
- Compare: `cargo run --manifest-path tools/glyph-verify/Cargo.toml -- <spell> <invocation>`

Options:
- `--deny-extra` Fail if invocation contains extra constraints/entities.
- `--strict-intent` Fail if intent text differs.
- `--allow-open-questions` Allow `?` open-question lines in the invocation.

Exit codes:
- `0` bound
- `1` not bound (constraint mismatch)
- `2` usage/parse errors
