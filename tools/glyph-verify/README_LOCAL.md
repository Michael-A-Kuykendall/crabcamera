This directory is a vendored copy of the Sorcery `glyph-verify` tool.

Use it as the gate between:
- canonical spell text (e.g. `HEADLESS_SPELLS.MD`)
- an invocation record you write after implementing a slice

Commands:
- Build: `cargo build --manifest-path tools/glyph-verify/Cargo.toml`
- Check binding: `cargo run --manifest-path tools/glyph-verify/Cargo.toml -- <spell.md> <invocation.md>`

Recommended flags when gating:
- `--deny-extra`
- `--strict-intent`

Exit codes:
- `0` BOUND
- `1` NOT BOUND
- `2` parse/usage
