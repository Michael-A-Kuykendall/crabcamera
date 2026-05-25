# Agent Instructions

## Session Completion

**When ending a work session**, work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **Run quality gates** (if code changed):
   ```bash
   cargo test --lib --features recording   # must be 85/85
   cargo check --all-features              # must be 0 errors
   ```
2. **PUSH TO REMOTE**:
   ```bash
   git pull --rebase
   git push
   git status  # MUST show "up to date with origin"
   ```
3. **Hand off** — Provide context for next session

**CRITICAL RULES:**
- NEVER stop before pushing — that leaves work stranded locally
- NEVER say "ready to push when you are" — YOU must push
- NEVER `git push` without presenting all changes to the user for approval first
- NEVER add "Co-Authored-By: Claude" or AI attribution in commit messages
- If push fails, resolve and retry until it succeeds

