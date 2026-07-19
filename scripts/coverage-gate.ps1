param(
    [double]$MinCoverage = 60.0,
    [switch]$SkipSmartTrigger = $true
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $false

$tarpaulinArgs = @(
    "tarpaulin",
    "--lib",
    "--features", "recording",
    "--engine", "llvm",
    "--target-dir", "C:/t/ccov",
    "--skip-clean",
    "--exclude-files", "target/*",
    "--out", "Stdout"
)
if ($SkipSmartTrigger) {
    $tarpaulinArgs += @("--", "--skip", "smart_trigger")
}

Write-Host "Running: cargo $($tarpaulinArgs -join ' ')" -ForegroundColor Cyan
$stdoutFile = [System.IO.Path]::GetTempFileName()
$stderrFile = [System.IO.Path]::GetTempFileName()
try {
    $startArgs = @{
        FilePath = "cargo"
        ArgumentList = $tarpaulinArgs
        NoNewWindow = $true
        Wait = $true
        PassThru = $true
        RedirectStandardOutput = $stdoutFile
        RedirectStandardError = $stderrFile
    }
    $proc = Start-Process @startArgs

    $outputText = @(
        Get-Content -Path $stdoutFile -Raw
        Get-Content -Path $stderrFile -Raw
    ) -join [Environment]::NewLine

    $global:LASTEXITCODE = $proc.ExitCode
}
finally {
    Remove-Item -Path $stdoutFile, $stderrFile -ErrorAction SilentlyContinue
}

$outputText | Write-Output

if ($LASTEXITCODE -ne 0) {
    Write-Error "Tarpaulin command failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

$match = [regex]::Match($outputText, "([0-9]+\.[0-9]+)% coverage")
if (-not $match.Success) {
    Write-Error "Could not parse coverage percentage from tarpaulin output."
    exit 2
}

$coverage = [double]$match.Groups[1].Value
Write-Host ("Parsed coverage: {0:N2}%" -f $coverage) -ForegroundColor Yellow

if ($coverage -lt $MinCoverage) {
    Write-Error ("Coverage gate failed: {0:N2}% < required {1:N2}%" -f $coverage, $MinCoverage)
    exit 1
}

Write-Host ("Coverage gate passed: {0:N2}% >= required {1:N2}%" -f $coverage, $MinCoverage) -ForegroundColor Green
