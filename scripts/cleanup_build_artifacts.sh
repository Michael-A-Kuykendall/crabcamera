#!/usr/bin/env bash
set -euo pipefail

# A small cleanup helper to remove tracked build artifacts from the git index
# and optionally delete them locally. Run from repository root.

echo "This script will run 'git rm --cached' on known build artifacts and remove them from the index."
read -p "Proceed (y/N)? " yn
if [[ "$yn" != "y" && "$yn" != "Y" ]]; then
  echo "Aborted. No changes made."
  exit 0
fi

# Files and patterns to remove from the git index
files=(
  "sync_test.pdb"
  "test_simple.pdb"
  "sync_test.*.rcgu.o"
  "test_simple.*.rcgu.o"
)

dirs=(
  "target/package"
)

# Remove files if they are tracked
for f in "${files[@]}"; do
  echo "Attempting to remove tracked files matching: $f"
  git ls-files -z -- "$f" | xargs -0 -r git rm --cached || true
done

# Remove directories from index (if present)
for d in "${dirs[@]}"; do
  if [ -d "$d" ] || git ls-files --error-unmatch "$d" >/dev/null 2>&1; then
    echo "Removing directory from index: $d"
    git rm -r --cached -f "$d" || true
  fi
done

# Remove tarpaulin artifacts if present
git ls-files -z -- "tarpaulin-report.html" "tarpaulin.json" | xargs -0 -r git rm --cached || true

# Success
echo "Staged removal of build artifacts from git index. Commit them with an appropriate message."

echo "Suggested commit message: 'chore: remove checked-in build artifacts (PDBs, rcgu.o, target/package, tarpaulin outputs) and add .gitignore'"

exit 0
