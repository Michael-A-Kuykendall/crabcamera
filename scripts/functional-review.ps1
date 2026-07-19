param(
    [switch]$SkipHardware,
    [switch]$WithCoverage,
    [double]$CoverageThreshold = 60.0
)

$ErrorActionPreference = 'Stop'

Write-Host "[1/5] cargo test --lib --features recording" -ForegroundColor Cyan
cargo test --lib --features recording

Write-Host "[2/5] cargo test --test commands_advanced_test --features recording" -ForegroundColor Cyan
cargo test --test commands_advanced_test --features recording

Write-Host "[3/5] cargo test --test types_test" -ForegroundColor Cyan
cargo test --test types_test

Write-Host "[4/5] cargo check --all-features" -ForegroundColor Cyan
cargo check --all-features

if (-not $SkipHardware) {
    Write-Host "[5/5] cargo run --example hardware_audit --features recording" -ForegroundColor Cyan
    cargo run --example hardware_audit --features recording
} else {
    Write-Host "[5/5] hardware audit skipped (--SkipHardware)" -ForegroundColor Yellow
}

if ($WithCoverage) {
    Write-Host "[coverage] enforcing threshold with scripts/coverage-gate.ps1" -ForegroundColor Cyan
    powershell -ExecutionPolicy Bypass -File scripts/coverage-gate.ps1 -MinCoverage $CoverageThreshold -SkipSmartTrigger
}

Write-Host "Functional review complete." -ForegroundColor Green
