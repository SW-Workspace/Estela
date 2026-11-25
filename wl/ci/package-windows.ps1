param(
    [Parameter(Mandatory = $true)]
    [string]$FfmpegDirectory,
    [string]$Destination = (Join-Path $PSScriptRoot "../target/windows-portable")
)

$ErrorActionPreference = "Stop"
$Destination = [System.IO.Path]::GetFullPath($Destination)
if (Test-Path $Destination) { throw "Choose a new destination to avoid stale runtime files." }
$runtime = (Resolve-Path $FfmpegDirectory).Path
$executable = Join-Path $runtime "ffmpeg.exe"
if (-not (Test-Path $executable -PathType Leaf)) {
    throw "FfmpegDirectory must contain ffmpeg.exe and its runtime DLLs."
}
$version = (& $executable -version 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0) { throw "The supplied FFmpeg runtime cannot run." }
if ($version -match '--enable-(gpl|nonfree)' -or $version -notmatch '--enable-shared') {
    throw "Supply a dynamically linked LGPL FFmpeg build without GPL or nonfree components."
}
$encoders = (& $executable -hide_banner -encoders 2>&1 | Out-String)
if ($LASTEXITCODE -ne 0 -or $encoders -notmatch '\bmpeg4\b') {
    throw "The supplied FFmpeg runtime must include the native MPEG-4 encoder."
}
Push-Location (Join-Path $PSScriptRoot "..")
try {
    & cargo build --release --locked -p estela-app
    if ($LASTEXITCODE -ne 0) { throw "Estela build failed." }
    New-Item -ItemType Directory -Path $Destination | Out-Null
    Copy-Item "target/release/estela-app.exe" $Destination
    $bundle = Join-Path $Destination "ffmpeg"
    New-Item -ItemType Directory -Path $bundle | Out-Null
    Copy-Item (Join-Path $runtime "*") $bundle -Recurse
    & (Join-Path $bundle "ffmpeg.exe") -version
    if ($LASTEXITCODE -ne 0) { throw "The packaged FFmpeg runtime cannot run." }
} finally {
    Pop-Location
}
