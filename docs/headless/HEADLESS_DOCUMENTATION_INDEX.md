# CrabCamera Headless Expansion — Documentation Index

## Overview

Two companion documents define the headless-first expansion for CrabCamera v0.6.0:

1. **[HEADLESS_EXPANSION_EVALUATION.md](HEADLESS_EXPANSION_EVALUATION.md)** — Technical evaluation and execution roadmap
   - Architecture assessment
   - Current state verification
   - Phase-by-phase implementation plan
   - Risk mitigation strategies
   - Testing approach
   - Timeline and effort estimates

2. **[HEADLESS_SPELLBOOK.md](docs/HEADLESS_SPELLBOOK.md)** — Sealed architectural spells (sorcery notation)
   - 13 sealed spells defining contracts and dependencies
   - Executable intent compression
   - Invocation rules and guarantees
   - Non-goals explicitly stated
   - Dependency graph (acyclic)

---

## For Decision Makers

**Start here:** [HEADLESS_EXPANSION_EVALUATION.md](HEADLESS_EXPANSION_EVALUATION.md) — Executive summary

**Key findings:**
- ✅ Architecture is ready (85% confidence)
- ✅ No rewrite needed—just reorganization and extraction
- ✅ 3-4 weeks to v0.6.0 (one developer)
- ✅ Opens multiple market segments (CLI, Python, Node.js, HTTP, OBS)

---

## For Architects & Technical Leads

**Read both documents:**

1. [HEADLESS_EXPANSION_EVALUATION.md](HEADLESS_EXPANSION_EVALUATION.md) — Understand the architectural seams and design decisions
2. [HEADLESS_SPELLBOOK.md](docs/HEADLESS_SPELLBOOK.md) — Understand the sealed contracts and implementation constraints

**Key responsibilities:**
- Ensure all 13 spells are understood before Phase 1 begins
- Verify assumptions listed in spells against actual codebase
- Design review for `HeadlessSession` API (1 hour, critical path item)
- Approve feature flag strategy before implementation

---

## For Implementation Engineers

**Execute against spells, not prose.**

1. Read [HEADLESS_SPELLBOOK.md](docs/HEADLESS_SPELLBOOK.md) — this is your source of truth
2. Reference [HEADLESS_EXPANSION_EVALUATION.md](HEADLESS_EXPANSION_EVALUATION.md) for detailed implementation guidance
3. Follow the dependency graph in the spellbook (acyclic DAG)
4. Every line of code must trace to a spell guarantee (`!`)
5. Never violate a spell exclusion (`-`)

**Critical spells for each phase:**
- Phase 1: `Headless_Core_Surface`, `Headless_Session_API`, `Headless_Lifecycle_Idempotency`
- Phase 2: `Headless_Frame_Delivery_Contract`, `Headless_Audio_Delivery_Contract`, `CLI_Headless_Harness`
- Phase 3: `Mock_Backend_For_CI`, `Tauri_Adapter_Preservation`, `Headless_Docs_And_NonGoals`

---

## For QA & Testing

**Test these explicit contracts** (from spells):

| Spell | What to Test | Success Criteria |
|-------|--------------|------------------|
| Lifecycle_Idempotency | Open/start/stop/close idempotence | No errors on retry, no hangs |
| Frame_Delivery_Contract | Frame ordering, drops, timeouts | Monotonic sequence, correct None semantics |
| Audio_Delivery_Contract | Audio doesn't block video, timeout paths | Video continues if audio fails |
| Control_Surface_Contract | Control get/set roundtrip | Platform differences documented |
| Mock_Backend_For_CI | Deterministic mock frames | Same seed = same output |
| CLI_Headless_Harness | CLI commands and JSON output | All commands work, JSON parseable |

---

## Document Structure

### HEADLESS_EXPANSION_EVALUATION.md (673 lines)

```
Part 1: Architecture Assessment
  ├─ Module structure analysis
  ├─ Key abstraction points
  ├─ Recording & audio integration
  └─ Testing infrastructure

Part 2: Flagged Claims Verification
  ├─ Claim 1: Code separation
  ├─ Claim 2: Control consistency
  └─ Claim 3: Audio stability

Part 3: Design for Headless Extraction
  ├─ Workspace structure
  ├─ API contract: HeadlessSession
  └─ CLI architecture

Part 4: Execution Roadmap
  ├─ Phase 1: Core extraction (3-4 days)
  ├─ Phase 2: CLI implementation (3-4 days)
  ├─ Phase 3: Documentation & polish (3-4 days)
  └─ Phase 4: Market launch

Part 5-10: Risk mitigation, testing strategy, acceptance criteria, metrics, estimates, roadmap
```

### HEADLESS_SPELLBOOK.md (578 lines)

```
Master Spell: Headless_Architecture_Master
  ├─ Spell 1: Headless_Core_Surface
  ├─ Spell 2: Headless_Session_API
  ├─ Spell 3: Headless_Lifecycle_Idempotency
  ├─ Spell 4: Headless_Frame_Delivery_Contract
  ├─ Spell 5: Headless_Audio_Delivery_Contract
  ├─ Spell 6: Headless_Control_Surface_Contract
  ├─ Spell 7: CLI_Headless_Harness
  ├─ Spell 8: Tauri_Adapter_Preservation
  ├─ Spell 9: Mock_Backend_For_CI
  ├─ Spell 10: Headless_Docs_And_NonGoals
  ├─ Spell 11: Headless_Refactoring_Strategy
  └─ Spell 12: Headless_v0_6_Release

Plus: Dependency graph, invocation rules, sealed verification
```

---

## Quick Reference: 13 Sealed Spells

| # | Spell Name | Dependencies | Key Guarantee |
|---|-----------|--------------|---------------|
| 1 | Headless_Architecture_Master | All others | Master binding |
| 2 | Headless_Core_Surface | Types, Errors, PlatformCamera | UI-free library surface |
| 3 | Headless_Session_API | Session, Lifecycle, Frame, Control | Session-scoped, no globals |
| 4 | Headless_Lifecycle_Idempotency | Errors | All state transitions explicit |
| 5 | Headless_Frame_Delivery_Contract | BufferPolicy, Errors | Monotonic order, explicit drops |
| 6 | Headless_Audio_Delivery_Contract | Errors, Lifecycle | Audio never stops video |
| 7 | Headless_Control_Surface_Contract | Types, Errors | Unsupported → explicit error |
| 8 | CLI_Headless_Harness | Session, Lifecycle, Frame, Control | Thin CLI using only headless |
| 9 | Tauri_Adapter_Preservation | Session, Types, Errors | Tauri still works unchanged |
| 10 | Mock_Backend_For_CI | Frame, Audio, Control contracts | Deterministic for testing |
| 11 | Headless_Refactoring_Strategy | (Implementation guide) | Phases don't break tests |
| 12 | Headless_v0_6_0_Release | All spells | Clear ship list and success metrics |
| 13 | (Master has sub-spells) | (Dependency graph below) | (See spellbook) |

---

## Execution Checklist

Before implementation begins:

- [ ] Read HEADLESS_EXPANSION_EVALUATION.md (full)
- [ ] Read HEADLESS_SPELLBOOK.md (full)
- [ ] Verify assumptions in spellbook against codebase
- [ ] Design review: HeadlessSession API (1 hour)
- [ ] Decide on feature flag strategy (tauri-plugin vs. headless vs. cli)
- [ ] Set up CI for mock backend testing
- [ ] Create GitHub issues for each spell (13 total)
- [ ] Assign phases to sprint calendar

---

## Success Criteria (From Spellbook)

v0.6.0 ships when all of these are true:

✅ **Core API**
- HeadlessSession compiles and passes tests
- Session lifecycle (open → start → frame loop → stop → close) works
- Tauri plugin still works unchanged

✅ **CLI Tool**
- `crabcamera devices` works on Windows/macOS/Linux
- `crabcamera capture` captures N frames deterministically
- `--json` output works for automation

✅ **Documentation**
- docs/HEADLESS.md exists and defines headless semantics
- README mentions headless mode with link to docs
- Troubleshooting guide exists

✅ **Testing**
- Unit tests: 100% pass (mock backend)
- Integration tests: CLI commands work
- Platform tests: Windows/macOS/Linux CI green
- No performance regression vs. Tauri mode

---

## Next Steps

1. **Stakeholder alignment** — Review summary with team (30 min)
2. **Design review** — HeadlessSession API detail (1 hour)
3. **Implementation kickoff** — Phase 1 begins with Core types
4. **Weekly checkpoint** — Track against spell guarantees, not just LOC

---

**Documents sealed:** December 19, 2025  
**Status:** ✅ READY FOR EXECUTION  
**Authority:** Sorcery-sealed architectural contracts
