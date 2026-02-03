param(
  [string]$Source = ""
)

$ErrorActionPreference = "Stop"

function Resolve-SourceDir {
  param([string]$Path)
  if ($Path -and (Test-Path -LiteralPath $Path)) { return (Resolve-Path -LiteralPath $Path).Path }
  if (Test-Path -LiteralPath ".\\dist\\windows\\StyledCamera") { return (Resolve-Path -LiteralPath ".\\dist\\windows\\StyledCamera").Path }
  if (Test-Path -LiteralPath ".\\StyledCamera") { return (Resolve-Path -LiteralPath ".\\StyledCamera").Path }
  throw "Usage: .\\scripts\\install-windows.ps1 -Source C:\\path\\to\\StyledCamera (folder). Expected .\\StyledCamera or .\\dist\\windows\\StyledCamera."
}

$srcDir = Resolve-SourceDir -Path $Source

$dstDir = Join-Path $env:APPDATA "obs-studio\\plugins\\StyledCamera"

Write-Host "Installing StyledCamera to:"
Write-Host ("  " + $dstDir)

if (Test-Path -LiteralPath $dstDir) {
  Remove-Item -Recurse -Force -LiteralPath $dstDir
}
New-Item -ItemType Directory -Force -Path $dstDir | Out-Null

# Use -Path (not -LiteralPath) so the wildcard expands.
Copy-Item -Recurse -Force -Path (Join-Path $srcDir "*") -Destination $dstDir

Write-Host "Done."
Write-Host "Next: start OBS and verify the plugin loads (Help -> Log Files -> View Current Log)."
