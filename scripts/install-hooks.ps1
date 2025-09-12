# Install CrabCamera Development Hooks
# Sets up pre-commit hooks for the repository

Write-Host "🦀 Installing CrabCamera Development Hooks" -ForegroundColor Cyan

# Create .git/hooks directory if it doesn't exist
$hooksDir = ".git\hooks"
if (-not (Test-Path $hooksDir)) {
    New-Item -Path $hooksDir -ItemType Directory -Force
    Write-Host "✅ Created hooks directory" -ForegroundColor Green
}

# Install pre-commit hook
$preCommitHook = @"
#!/bin/sh
# CrabCamera pre-commit hook
# Runs tests and coverage analysis

echo "Running CrabCamera pre-commit checks..."
powershell.exe -ExecutionPolicy Bypass -File scripts/pre-commit.ps1
exit $?
"@

$preCommitPath = "$hooksDir\pre-commit"
$preCommitHook | Out-File -FilePath $preCommitPath -Encoding ASCII -Force

# Make executable (Windows doesn't need chmod, but let's be consistent)
if (Get-Command icacls -ErrorAction SilentlyContinue) {
    icacls $preCommitPath /grant Everyone:RX | Out-Null
}

Write-Host "✅ Installed pre-commit hook" -ForegroundColor Green

# Create coverage enforcement configuration
$coverageConfig = @{
    minimum_coverage = 80.0
    enforce_on_commit = $true
    exclude_files = @("target/*", "*/tests/*", "demos/*")
    timeout_seconds = 300
}

$coverageConfig | ConvertTo-Json | Out-File -FilePath "tarpaulin.json" -Encoding UTF8 -Force
Write-Host "✅ Created coverage configuration" -ForegroundColor Green

Write-Host ""
Write-Host "🎯 Development hooks installed successfully!" -ForegroundColor Green
Write-Host "⚡ Pre-commit will now enforce:" -ForegroundColor Yellow
Write-Host "   - All tests must pass" -ForegroundColor White
Write-Host "   - 80%+ code coverage required" -ForegroundColor White  
Write-Host "   - No clippy warnings allowed" -ForegroundColor White
Write-Host ""
Write-Host "🔧 To skip hooks temporarily: git commit --no-verify" -ForegroundColor Cyan