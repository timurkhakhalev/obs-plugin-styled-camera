$ErrorActionPreference = "Stop"

$dstDir = Join-Path $env:APPDATA "obs-studio\\plugins\\StyledCamera"

if (Test-Path -LiteralPath $dstDir) {
  Write-Host "Removing:"
  Write-Host ("  " + $dstDir)
  Remove-Item -Recurse -Force -LiteralPath $dstDir
  Write-Host "Done."
} else {
  Write-Host ("Not installed (missing): " + $dstDir)
}

