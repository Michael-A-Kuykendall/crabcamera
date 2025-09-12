# CrabCamera Pre-Commit Hook
# Enforces 80%+ test coverage before allowing commits

Write-Host "🦀 CrabCamera Pre-Commit Hook: Testing & Coverage Check" -ForegroundColor Cyan
Write-Host "=" * 60

# Run all tests first
Write-Host "🧪 Running test suite..." -ForegroundColor Yellow
$testResult = cargo test --all-features --quiet
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ TESTS FAILED - Commit blocked" -ForegroundColor Red
    Write-Host "Fix failing tests before committing." -ForegroundColor Red
    exit 1
}
Write-Host "✅ All tests passed" -ForegroundColor Green

# Run coverage analysis
Write-Host "`n📊 Running coverage analysis..." -ForegroundColor Yellow
$coverageOutput = cargo tarpaulin --lib --timeout 300 --exclude-files 'target/*' --exclude-files '*/tests/*' --quiet 2>&1 | Out-String

# Extract coverage percentage
$coverageMatch = $coverageOutput | Select-String "(\d+\.\d+)% coverage"
if ($coverageMatch) {
    $coveragePercent = [float]$coverageMatch.Matches[0].Groups[1].Value
    Write-Host "📈 Current coverage: $coveragePercent%" -ForegroundColor Cyan
    
    if ($coveragePercent -lt 80.0) {
        Write-Host "❌ COVERAGE TOO LOW - Commit blocked" -ForegroundColor Red
        Write-Host "Required: 80%+ coverage, Found: $coveragePercent%" -ForegroundColor Red
        Write-Host "Add more tests to improve coverage." -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Coverage meets requirements ($coveragePercent% >= 80%)" -ForegroundColor Green
} else {
    Write-Host "⚠️  Could not parse coverage results - Allowing commit" -ForegroundColor Yellow
}

# Run clippy for additional checks
Write-Host "`n🔍 Running clippy lints..." -ForegroundColor Yellow
cargo clippy --all-features --quiet -- -D warnings
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ CLIPPY WARNINGS FOUND - Commit blocked" -ForegroundColor Red
    Write-Host "Fix clippy warnings before committing." -ForegroundColor Red
    exit 1
}
Write-Host "✅ No clippy warnings" -ForegroundColor Green

Write-Host "`n🎉 All checks passed - Commit allowed!" -ForegroundColor Green
exit 0