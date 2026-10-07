$ErrorActionPreference = "Stop"
$home_ = $env:USERPROFILE
$main = Join-Path $home_ ".claude"
$binDir = Join-Path $main "bin"
foreach ($d in @($main, $binDir)) { if (-not (Test-Path -LiteralPath $d)) { New-Item -ItemType Directory -Path $d | Out-Null } }

$exe = Join-Path $binDir "statusline.exe"
Copy-Item -LiteralPath (Join-Path $PSScriptRoot "bin\statusline.exe") -Destination $exe -Force
Write-Host ("programme copie dans " + $exe)

$conf = Join-Path $main "statusline.json"
if (Test-Path -LiteralPath $conf) { Write-Host "statusline.json existe deja, conserve" }
else { Copy-Item -LiteralPath (Join-Path $PSScriptRoot "statusline.json") -Destination $conf; Write-Host "statusline.json installe" }

$dirs = @($main) + @(Get-ChildItem -LiteralPath $home_ -Directory -Filter ".claude-compte*" | ForEach-Object { $_.FullName })
$cmd = $exe.Replace("\", "/")
foreach ($d in $dirs) {
  $f = Join-Path $d "settings.json"
  if (Test-Path -LiteralPath $f) {
    Copy-Item -LiteralPath $f -Destination ($f + ".bak-statusline") -Force
    $j = Get-Content -LiteralPath $f -Raw | ConvertFrom-Json
  } else { $j = [PSCustomObject]@{} }
  $sl = [PSCustomObject]@{ type = "command"; command = $cmd; refreshInterval = 3 }
  if ($j.PSObject.Properties["statusLine"]) { $j.statusLine = $sl } else { $j | Add-Member -NotePropertyName statusLine -NotePropertyValue $sl }
  [IO.File]::WriteAllText($f, ($j | ConvertTo-Json -Depth 30), (New-Object Text.UTF8Encoding($false)))
  Write-Host ("barre activee dans " + $f)
}
Write-Host "termine, relance Claude Code"
