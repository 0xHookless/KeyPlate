# Builds the release installer without anything that identifies the build machine:
# local folder paths are rewritten (no user name in the binary) and the linker writes no PDB,
# so the exe carries no debug-symbol link. Then records the installer's SHA-256.
#
#   powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent

# CARGO_ENCODED_RUSTFLAGS separates flags with 0x1F, so paths with spaces stay intact.
$env:CARGO_ENCODED_RUSTFLAGS = @(
    "--remap-path-prefix=$env:USERPROFILE=~",
    "--remap-path-prefix=$root=.",
    "-C", "link-arg=/DEBUG:NONE"
) -join [char]0x1F
Remove-Item Env:RUSTFLAGS -ErrorAction SilentlyContinue

Push-Location $root
try {
    npm run tauri build
    if ($LASTEXITCODE -ne 0) { throw "tauri build failed" }

    $exe = Get-ChildItem target\release\bundle\nsis\*-setup.exe | Sort-Object LastWriteTime | Select-Object -Last 1
    $app = Get-Item target\release\keyplate.exe

    # Refuse to ship a binary that still names this machine's user folder or links a PDB.
    $bytes = [System.Text.Encoding]::ASCII.GetString([System.IO.File]::ReadAllBytes($app.FullName))
    foreach ($p in @($env:USERPROFILE, $env:USERPROFILE.Replace('\', '/'))) {
        if ($bytes.IndexOf($p, [StringComparison]::OrdinalIgnoreCase) -ge 0) { throw "keyplate.exe still contains $p" }
    }
    if ($bytes.Contains("RSDS")) { throw "keyplate.exe still links a PDB" }

    $hash = (Get-FileHash $exe.FullName -Algorithm SHA256).Hash.ToLower()
    "$hash  $($exe.Name)" | Out-File -Encoding ascii (Join-Path $exe.DirectoryName "SHA256.txt")
    Write-Output "$($exe.FullName)"
    Write-Output "SHA-256 $hash"
}
finally {
    Pop-Location
}
