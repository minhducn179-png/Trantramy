param(
    [string]$Archive = 'dist/Trantramy-2.0.0-windows-x64.zip',
    [switch]$SkipGui
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archivePath = (Resolve-Path $Archive).Path
$expected = (Get-Content "$archivePath.sha256").Split(' ')[0]
$actual = (Get-FileHash $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -ne $actual) { throw 'Archive checksum mismatch' }
$directory = Join-Path ([IO.Path]::GetTempPath()) ('trantramy-package-' + [guid]::NewGuid())
$previousDataDir = $env:TRANTRAMY_DATA_DIR
$process = $null
try {
    [IO.Compression.ZipFile]::ExtractToDirectory($archivePath, $directory)
    $root = Join-Path $directory ([IO.Path]::GetFileNameWithoutExtension($archivePath))
    foreach ($file in @('Trantramy.exe', 'binaries/yt-dlp.exe', 'README.md', 'THIRD_PARTY_NOTICES.md', 'DEPENDENCIES.tsv', 'sidecars.json')) {
        if (-not (Test-Path (Join-Path $root $file))) { throw "Missing packaged file: $file" }
    }
    if ((Get-ChildItem (Join-Path $root 'licenses') -Recurse -File).Count -eq 0) { throw 'Missing dependency license texts' }
    $manifest = Get-Content (Join-Path $root 'sidecars.json') -Raw | ConvertFrom-Json
    $sidecarHash = (Get-FileHash (Join-Path $root 'binaries/yt-dlp.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($sidecarHash -ne $manifest.assets.windows.sha256) { throw 'Packaged sidecar checksum mismatch' }
    $versionOutput = Join-Path $directory 'version.txt'
    $versionError = Join-Path $directory 'version-error.txt'
    $process = Start-Process -FilePath (Join-Path $root 'Trantramy.exe') -ArgumentList '--version' -PassThru -Wait -RedirectStandardOutput $versionOutput -RedirectStandardError $versionError
    $version = (Get-Content $versionOutput -Raw).Trim()
    $expectedVersion = 'Trantramy ' + ([IO.Path]::GetFileNameWithoutExtension($archivePath) -replace '^Trantramy-(.+)-windows-x64$', '$1')
    if ($process.ExitCode -ne 0 -or $version -ne $expectedVersion) { throw "Packaged executable version failed: $version" }
    $sidecarVersion = & (Join-Path $root 'binaries/yt-dlp.exe') --version
    if ($LASTEXITCODE -ne 0 -or $sidecarVersion -ne $manifest.release) { throw 'Packaged sidecar failed' }
    Write-Output "Verified $version; yt-dlp=$sidecarVersion; SHA256=$actual"
    if (-not $SkipGui) {
        $env:TRANTRAMY_DATA_DIR = Join-Path $directory 'data'
        $process = Start-Process -FilePath (Join-Path $root 'Trantramy.exe') -WorkingDirectory $directory -PassThru
        Start-Sleep -Seconds 3
        $process.Refresh()
        if ($process.HasExited) { throw "Packaged GUI exited: $($process.ExitCode)" }
        if ($process.MainWindowHandle -eq 0) { throw 'Packaged GUI has no native window' }
        if (-not (Test-Path (Join-Path $env:TRANTRAMY_DATA_DIR 'dlc.db'))) { throw 'Packaged GUI did not initialize isolated database' }
        Write-Output "Verified native window=$($process.MainWindowHandle) and isolated SQLite database"
        $process.CloseMainWindow() | Out-Null
        if (-not $process.WaitForExit(10000)) { $process.Kill(); throw 'Native GUI did not close cleanly' }
    }
} finally {
    if ($process -and -not $process.HasExited) { $process.Kill(); $process.WaitForExit() }
    $env:TRANTRAMY_DATA_DIR = $previousDataDir
    if (Test-Path $directory) { Remove-Item $directory -Recurse -Force }
}
