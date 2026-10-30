param([switch]$Build)
$ErrorActionPreference = "Stop"

function Get-LanguageCode([string]$tag) {
  $code = ($tag.Trim() -split "[_.@-]")[0].ToLowerInvariant()
  if ($code -eq "fr" -or $code -eq "en") { return $code }
  return $null
}
$language = Get-LanguageCode "$env:STATUSLINE_LANG"
if (-not $language) { $language = Get-LanguageCode (Get-UICulture).TwoLetterISOLanguageName }
$french = $language -eq "fr"
$texts = @{
  downloading = "downloading {0}", "téléchargement de {0}"
  checksum = "SHA-256 hash missing from SHA256SUMS or different.", "empreinte SHA-256 absente de SHA256SUMS ou différente."
  release_refused = "released binary refused: {0} Run the script again with -Build to compile the sources (Rust required).", "binaire publié refusé : {0} Relancez avec -Build pour compiler les sources (Rust requis)."
  build_failed = "build failed", "échec de la compilation"
  copied = "program copied to {0}", "programme copié dans {0}"
  config_kept = "statusline.json already exists, kept", "statusline.json existe déjà, conservé"
  config_installed = "statusline.json installed", "statusline.json installé"
  registry_unreadable = "account list cannot be read: {0}", "liste des comptes illisible : {0}"
  account_kept = "account left as is: {0}", "compte laissé tel quel : {0}"
  command_replaced = "command replaced for {0} (was: {1})", "commande remplacée pour {0} (était : {1})"
  done = "done, restart Claude Code", "terminé, relancez Claude Code"
}
function Get-Text([string]$key) {
  $text = if ($french) { $texts[$key][1] } else { $texts[$key][0] }
  $text -f $args
}

$main = Join-Path $env:USERPROFILE ".claude"
$binDir = Join-Path $main "bin"
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
$exe = Join-Path $binDir "statusline.exe"
$releaseUrl = "https://github.com/LetermeFlorent/claude-statusline/releases/latest/download/"
$asset = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "statusline-windows-arm64.exe" } else { "statusline-windows-x86_64.exe" }
function Get-BinaryLanguage([string]$file) {
  $saved = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try { $answer = & $file --language 2>$null | Select-Object -First 1 } catch { $answer = $null } finally { $ErrorActionPreference = $saved }
  return Get-LanguageCode "$answer"
}
function Remove-OldCopies { Get-ChildItem -LiteralPath $binDir -Filter "statusline.exe.old*" | Remove-Item -Force -ErrorAction SilentlyContinue }
function Get-StatusCommand([string]$file) {
  if (-not (Test-Path -LiteralPath $file)) { return $null }
  try { return (Get-Content -LiteralPath $file -Raw -Encoding UTF8 | ConvertFrom-Json).statusLine.command } catch { return $null }
}
function Save-ReleaseBinary([string]$destination) {
  $sums = $destination + ".sums"
  [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
  $ProgressPreference = "SilentlyContinue"
  Write-Host (Get-Text downloading $asset)
  try {
    Invoke-WebRequest -UseBasicParsing -Uri ($releaseUrl + "SHA256SUMS") -OutFile $sums
    Invoke-WebRequest -UseBasicParsing -Uri ($releaseUrl + $asset) -OutFile $destination
    $expected = @(foreach ($line in Get-Content -LiteralPath $sums) {
      $fields = -split $line
      if ($fields.Count -eq 2 -and $fields[1].TrimStart("*") -eq $asset) { $fields[0] }
    })
    $actual = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash
    if ($expected.Count -ne 1 -or $actual -ne $expected[0]) { throw (Get-Text checksum) }
  } catch {
    Remove-Item -LiteralPath $destination -Force -ErrorAction SilentlyContinue
    throw (Get-Text release_refused $_.Exception.Message)
  } finally {
    Remove-Item -LiteralPath $sums -Force -ErrorAction SilentlyContinue
  }
}

if ($Build) {
  Push-Location $PSScriptRoot
  try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw (Get-Text build_failed) }
  } finally { Pop-Location }
  $src = Join-Path $PSScriptRoot "target\release\statusline.exe"
} else {
  $src = $exe + ".download"
  Save-ReleaseBinary $src
}

if (Test-Path -LiteralPath $exe) {
  Remove-OldCopies
  Move-Item -LiteralPath $exe -Destination ($exe + ".old-" + [DateTimeOffset]::UtcNow.ToUnixTimeSeconds())
}
if ($Build) { Copy-Item -LiteralPath $src -Destination $exe } else { Move-Item -LiteralPath $src -Destination $exe }
Unblock-File -LiteralPath $exe -ErrorAction SilentlyContinue
Remove-OldCopies
$asked = Get-BinaryLanguage $exe
if ($asked) { $language = $asked; $french = $language -eq "fr" }
Write-Host (Get-Text copied $exe)

$conf = Join-Path $main "statusline.json"
if (Test-Path -LiteralPath $conf) { Write-Host (Get-Text config_kept) }
else { Copy-Item -LiteralPath (Join-Path $PSScriptRoot "statusline.json") -Destination $conf; Write-Host (Get-Text config_installed) }

$dirs = [ordered]@{}
$found = @($main) + @(Get-ChildItem -LiteralPath $env:USERPROFILE -Directory -Filter ".claude-compte*" | ForEach-Object { $_.FullName })
$reg = Join-Path $env:USERPROFILE ".claude-accounts.json"
if (Test-Path -LiteralPath $reg) {
  try {
    $list = Get-Content -LiteralPath $reg -Raw | ConvertFrom-Json
    foreach ($a in $list) { if ($a.Dir) { $found += Join-Path $env:USERPROFILE $a.Dir } }
  } catch { Write-Warning (Get-Text registry_unreadable $reg) }
}
foreach ($d in $found) {
  if (-not (Test-Path -LiteralPath $d)) { continue }
  $key = $d.TrimEnd("\", "/").ToLowerInvariant()
  if (-not $dirs.Contains($key)) { $dirs[$key] = $d }
}

$prev = $env:CLAUDE_CONFIG_DIR
$prevLanguage = $env:STATUSLINE_LANG
$env:STATUSLINE_LANG = $language
try {
  foreach ($d in $dirs.Values) {
    $sfile = Join-Path $d "settings.json"
    $old = Get-StatusCommand $sfile
    $env:CLAUDE_CONFIG_DIR = $d
    & $exe --install
    if ($LASTEXITCODE -ne 0) { Write-Warning (Get-Text account_kept $d); continue }
    $new = Get-StatusCommand $sfile
    if ($old -and $old -ne $new) { Write-Host (Get-Text command_replaced $d $old) }
  }
} finally { $env:CLAUDE_CONFIG_DIR = $prev; $env:STATUSLINE_LANG = $prevLanguage }
Write-Host (Get-Text done)
