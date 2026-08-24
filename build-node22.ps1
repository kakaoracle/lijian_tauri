$ErrorActionPreference = "Stop"

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $root

$env:NPM_CONFIG_CACHE = "D:\project\.npm-cache"
$env:RUSTUP_HOME = "D:\project\.rustup"
$env:CARGO_HOME = "D:\project\.cargo-global"
$env:CARGO_TARGET_DIR = Join-Path $root ".cargo-target"
$env:CARGO_NET_OFFLINE = "false"

$node = "D:\software\node22\node.exe"
$tauriCli = Join-Path $root "node_modules\@tauri-apps\cli\tauri.js"
& $node $tauriCli build --no-bundle
if ($LASTEXITCODE -ne 0) {
    throw "Tauri build failed with exit code $LASTEXITCODE"
}

$package = Get-Content -Raw (Join-Path $root "package.json") | ConvertFrom-Json
$releaseDir = Join-Path $env:CARGO_TARGET_DIR "release"
$executable = Join-Path $releaseDir "moyu.exe"
$portableDir = Join-Path $releaseDir "bundle\portable"
$portableExecutable = Join-Path $portableDir "moyu_$($package.version)_x64-portable.exe"

if (-not (Test-Path -LiteralPath $executable)) {
    throw "Portable executable was not produced: $executable"
}
New-Item -ItemType Directory -Path $portableDir -Force | Out-Null
if (Test-Path -LiteralPath $portableExecutable) {
    Remove-Item -LiteralPath $portableExecutable -Force
}
Copy-Item -LiteralPath $executable -Destination $portableExecutable -Force
Write-Output "Portable executable: $portableExecutable"
