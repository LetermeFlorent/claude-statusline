# Installe la barre pour tous les comptes Claude Code du poste (Windows).
# -Build compile les sources (Rust requis) au lieu de prendre bin\statusline.exe
param([switch]$Build)
$ErrorActionPreference = "Stop"

$src = Join-Path $PSScriptRoot "bin\statusline.exe"
if ($Build) {
  Push-Location $PSScriptRoot
  try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw "echec de la compilation" }
  } finally { Pop-Location }
  $src = Join-Path $PSScriptRoot "target\release\statusline.exe"
}

$main = Join-Path $env:USERPROFILE ".claude"
$binDir = Join-Path $main "bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
$exe = Join-Path $binDir "statusline.exe"
# Claude Code peut etre en train de lancer la barre : un exe en cours ne se remplace pas, il se renomme
if (Test-Path -LiteralPath $exe) {
  Remove-Item -LiteralPath ($exe + ".old") -Force -ErrorAction SilentlyContinue
  Move-Item -LiteralPath $exe -Destination ($exe + ".old") -Force
}
Copy-Item -LiteralPath $src -Destination $exe
Unblock-File -LiteralPath $exe -ErrorAction SilentlyContinue
Remove-Item -LiteralPath ($exe + ".old") -Force -ErrorAction SilentlyContinue
Write-Host ("programme copie dans " + $exe)

$conf = Join-Path $main "statusline.json"
if (Test-Path -LiteralPath $conf) { Write-Host "statusline.json existe deja, conserve" }
else { Copy-Item -LiteralPath (Join-Path $PSScriptRoot "statusline.json") -Destination $conf; Write-Host "statusline.json installe" }

# Comptes : .claude, chaque .claude-compte*, et ceux declares dans le menu multi-comptes
$dirs = [ordered]@{}
$found = @($main) + @(Get-ChildItem -LiteralPath $env:USERPROFILE -Directory -Filter ".claude-compte*" | ForEach-Object { $_.FullName })
$reg = Join-Path $env:USERPROFILE ".claude-accounts.json"
if (Test-Path -LiteralPath $reg) {
  try {
    $list = Get-Content -LiteralPath $reg -Raw | ConvertFrom-Json
    foreach ($a in $list) { if ($a.Dir) { $found += Join-Path $env:USERPROFILE $a.Dir } }
  } catch { Write-Warning ("liste des comptes illisible : " + $reg) }
}
foreach ($d in $found) {
  if (-not (Test-Path -LiteralPath $d)) { continue }
  $key = $d.TrimEnd("\", "/").ToLowerInvariant()
  if (-not $dirs.Contains($key)) { $dirs[$key] = $d }
}

# Le binaire pose lui-meme l'entree statusLine, compte par compte
$prev = $env:CLAUDE_CONFIG_DIR
try {
  foreach ($d in $dirs.Values) {
    $env:CLAUDE_CONFIG_DIR = $d
    & $exe --install
    if ($LASTEXITCODE -ne 0) { Write-Warning ("compte laisse tel quel : " + $d) }
  }
} finally { $env:CLAUDE_CONFIG_DIR = $prev }
Write-Host "termine, relance Claude Code"
