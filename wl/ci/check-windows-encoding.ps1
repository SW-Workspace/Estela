param(
    [Parameter(Mandatory = $true)]
    [string]$FfmpegDirectory
)

$ErrorActionPreference = "Stop"
$executable = Join-Path (Resolve-Path $FfmpegDirectory).Path "ffmpeg.exe"
if (-not (Test-Path $executable -PathType Leaf)) {
    throw "FfmpegDirectory must contain ffmpeg.exe and its matching runtime DLLs."
}
$version = (& $executable -version 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $version -notmatch '--enable-shared' -or
    $version -match '--enable-(gpl|nonfree)') {
    throw "Supply a runnable dynamically linked LGPL FFmpeg build."
}
$previous = $env:ESTELA_FFMPEG
$env:ESTELA_FFMPEG = $executable
Push-Location (Join-Path $PSScriptRoot "..")
try {
    & cargo test --locked -p estela-platform-win --test encoding -- --include-ignored
    if ($LASTEXITCODE -ne 0) { throw "Native Windows encoding checks failed." }
    & cargo test --locked -p estela-engine --lib -- --include-ignored
    if ($LASTEXITCODE -ne 0) { throw "Recording lifecycle checks failed." }
    & cargo test --locked -p estela-engine --test recording_roundtrip -- --ignored
    if ($LASTEXITCODE -ne 0) { throw "Recording preview checks failed." }
} finally {
    Pop-Location
    $env:ESTELA_FFMPEG = $previous
}
