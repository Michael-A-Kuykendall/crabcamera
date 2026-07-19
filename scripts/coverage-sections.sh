#!/usr/bin/env bash
set -euo pipefail

# Section-targeted coverage for CrabCamera using cargo-tarpaulin.
# Usage:
#   scripts/coverage-sections.sh list
#   scripts/coverage-sections.sh run <section> [min_percent]
#   scripts/coverage-sections.sh run-all [min_percent]
#   scripts/coverage-sections.sh hotspots [log_file]

TARPAULIN_COMMON=(
  --lib
  --engine llvm
  --target-dir C:/t/ccov
  --skip-clean
  --exclude-files "target/*"
  --out Stdout
  --
  --skip smart_trigger
)

features_for_section() {
  case "${1:-}" in
    headless)
      printf '%s\n' "recording,headless"
      ;;
    *)
      printf '%s\n' "recording"
      ;;
  esac
}

sections=(
  core
  commands
  platform
  quality
  recording
  focus_stack
  headless
  testsupport
)

patterns_for_section() {
  case "${1:-}" in
    core)
      printf '%s\n' \
        "src/lib.rs" \
        "src/types.rs" \
        "src/config.rs" \
        "src/errors.rs" \
        "src/permissions.rs" \
        "src/constants.rs" \
        "src/invariant_ppt.rs" \
        "src/registry.rs"
      ;;
    commands)
      printf '%s\n' "src/commands/*"
      ;;
    platform)
      printf '%s\n' "src/platform/*"
      ;;
    quality)
      printf '%s\n' "src/quality/*"
      ;;
    recording)
      printf '%s\n' "src/recording/*"
      ;;
    focus_stack)
      printf '%s\n' "src/focus_stack/*"
      ;;
    headless)
      printf '%s\n' "src/headless/*"
      ;;
    testsupport)
      printf '%s\n' "src/testing/*" "src/tests/*"
      ;;
    *)
      return 1
      ;;
  esac
}

usage() {
  cat <<'EOF'
Section-targeted coverage runner

Commands:
  list
      List defined coverage sections and their file globs.

  run <section> [min_percent]
      Run tarpaulin for one section and optionally enforce min percentage.

  run-all [min_percent]
      Run all sections and print a summary table.

  hotspots [log_file]
      Show lowest-coverage files from a tarpaulin stdout log.
      Default log_file: /tmp/crabcamera-tarpaulin.log
EOF
}

print_section_list() {
  for s in "${sections[@]}"; do
    echo "[$s]"
    patterns_for_section "$s" | sed 's/^/  - /'
  done
}

parse_coverage() {
  local log_file="$1"
  grep -Eo '[0-9]+\.[0-9]+% coverage' "$log_file" | tail -1 | awk '{print $1}' | tr -d '%'
}

show_top_file_gaps() {
  local log_file="$1"
  awk 'BEGIN{FS="[:/ ]+"} /\|\| src\\.*: [0-9]+\/[0-9]+/{gsub("\\\\","/",$2); file=$2; tested=$3; total=$4; pct=(total>0?100*tested/total:0); printf("%6.2f %4d/%-4d %s\n", pct,tested,total,file)}' "$log_file" \
    | sort -n | head -12
}

run_section() {
  local section="$1"
  local min_percent="${2:-}"
  local log_file="/tmp/crabcamera-coverage-${section}.log"

  mapfile -t patterns < <(patterns_for_section "$section") || {
    echo "Unknown section: $section" >&2
    exit 2
  }

  local include_args=()
  for p in "${patterns[@]}"; do
    include_args+=(--include-files "$p")
  done

  local features
  features="$(features_for_section "$section")"

  echo "== Section: $section =="
  echo "Patterns: ${patterns[*]}"

  cargo tarpaulin "${include_args[@]}" --features "$features" "${TARPAULIN_COMMON[@]}" | tee "$log_file"

  local cov
  cov="$(parse_coverage "$log_file")"
  if [[ -z "$cov" ]]; then
    echo "Failed to parse coverage from $log_file" >&2
    exit 3
  fi

  printf 'Section coverage (%s): %.2f%%\n' "$section" "$cov"
  echo "Lowest files in this run:"
  show_top_file_gaps "$log_file"

  if [[ -n "$min_percent" ]]; then
    awk -v c="$cov" -v m="$min_percent" 'BEGIN { if (c+0 < m+0) { printf("Gate failed: %.2f < %.2f\n", c, m); exit 1 } else { printf("Gate passed: %.2f >= %.2f\n", c, m); exit 0 } }'
  fi
}

run_all_sections() {
  local min_percent="${1:-}"
  local summary="/tmp/crabcamera-coverage-sections-summary.txt"
  : > "$summary"

  for s in "${sections[@]}"; do
    local log_file="/tmp/crabcamera-coverage-${s}.log"
    mapfile -t patterns < <(patterns_for_section "$s")

    local include_args=()
    for p in "${patterns[@]}"; do
      include_args+=(--include-files "$p")
    done

    local features
    features="$(features_for_section "$s")"

    echo "Running section: $s"
    cargo tarpaulin "${include_args[@]}" --features "$features" "${TARPAULIN_COMMON[@]}" > "$log_file"

    local cov
    cov="$(parse_coverage "$log_file")"
    if [[ -z "$cov" ]]; then
      cov="NaN"
    fi

    printf '%-12s %s\n' "$s" "$cov" | tee -a "$summary"

    if [[ -n "$min_percent" && "$cov" != "NaN" ]]; then
      awk -v c="$cov" -v m="$min_percent" -v s="$s" 'BEGIN { if (c+0 < m+0) { printf("Gate failed for %s: %.2f < %.2f\n", s, c, m); exit 1 } }'
    fi
  done

  echo
  echo "Section summary:"
  column -t "$summary" || cat "$summary"
}

show_hotspots() {
  local log_file="${1:-/tmp/crabcamera-tarpaulin.log}"
  if [[ ! -f "$log_file" ]]; then
    echo "Log file not found: $log_file" >&2
    exit 4
  fi

  echo "Lowest-coverage files in $log_file"
  awk 'BEGIN{FS="[:/ ]+"} /\|\| src\\.*: [0-9]+\/[0-9]+/{gsub("\\\\","/",$2); file=$2; tested=$3; total=$4; pct=(total>0?100*tested/total:0); printf("%6.2f %4d/%-4d %s\n", pct,tested,total,file)}' "$log_file" | sort -n | head -40
}

main() {
  local cmd="${1:-}"
  case "$cmd" in
    list)
      print_section_list
      ;;
    run)
      if [[ $# -lt 2 ]]; then
        usage
        exit 1
      fi
      run_section "$2" "${3:-}"
      ;;
    run-all)
      run_all_sections "${2:-}"
      ;;
    hotspots)
      show_hotspots "${2:-}"
      ;;
    ""|-h|--help|help)
      usage
      ;;
    *)
      echo "Unknown command: $cmd" >&2
      usage
      exit 1
      ;;
  esac
}

main "$@"
