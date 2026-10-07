param([switch]$RetirerAncienne)
$ErrorActionPreference = "Stop"
$home_ = $env:USERPROFILE
$main = Join-Path $home_ ".claude"
$src = Join-Path $PSScriptRoot "mod\barre-etat"
$files = @(".claude-plugin\plugin.json", "hooks\hooks.json", "hooks\register.js", "hooks\render.js", "helper\sysinfo.cs")

if (-not (Test-Path -LiteralPath $main)) { New-Item -ItemType Directory -Path $main | Out-Null }
$conf = Join-Path $main "statusline.json"
if (Test-Path -LiteralPath $conf) { Write-Host "statusline.json existe deja, conserve" }
else { Copy-Item -LiteralPath (Join-Path $PSScriptRoot "statusline.json") -Destination $conf; Write-Host "statusline.json installe" }

# Les comptes secondaires de clm pointent souvent leur dossier skills sur celui
# du premier compte par une jonction : une seule copie suffit alors pour tous
$dirs = @($main) + @(Get-ChildItem -LiteralPath $home_ -Directory -Filter ".claude-compte*" | ForEach-Object { $_.FullName })
$done = @()
foreach ($d in $dirs) {
  $skills = Join-Path $d "skills"
  $real = $skills
  $item = Get-Item -LiteralPath $skills -Force -ErrorAction SilentlyContinue
  if ($item -ne $null -and $item.LinkType) { $real = [string]($item.Target | Select-Object -First 1) }
  $key = $real.TrimEnd("\").ToLower()
  if ($done -contains $key) { Write-Host ("mod deja presente pour " + $d + " (dossier skills partage)"); continue }
  $done += $key
  $dst = Join-Path $skills "barre-etat"
  if (Test-Path -LiteralPath $dst) { Remove-Item -LiteralPath $dst -Recurse -Force }
  foreach ($f in $files) {
    $to = Join-Path $dst $f
    $parent = Split-Path $to
    if (-not (Test-Path -LiteralPath $parent)) { New-Item -ItemType Directory -Path $parent -Force | Out-Null }
    Copy-Item -LiteralPath (Join-Path $src $f) -Destination $to -Force
  }
  Write-Host ("mod installee dans " + $dst)
}

if ($RetirerAncienne) {
  foreach ($d in $dirs) {
    $f = Join-Path $d "settings.json"
    if (-not (Test-Path -LiteralPath $f)) { continue }
    $j = Get-Content -LiteralPath $f -Raw | ConvertFrom-Json
    if (-not $j.PSObject.Properties["statusLine"]) { continue }
    Copy-Item -LiteralPath $f -Destination ($f + ".bak-statusline") -Force
    $j.PSObject.Properties.Remove("statusLine")
    [IO.File]::WriteAllText($f, ($j | ConvertTo-Json -Depth 30), (New-Object Text.UTF8Encoding($false)))
    Write-Host ("ancienne barre retiree de " + $f)
  }
}
Write-Host "termine, relance Claude Code"
