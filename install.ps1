param([switch]$Build, [switch]$Uninstall)
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
  checksum = "SHA-256 hash missing from the release manifest or different.", "empreinte SHA-256 absente du manifeste de publication ou différente."
  signature = "signature of the release manifest missing or invalid.", "signature du manifeste de publication absente ou invalide."
  no_signature_check = "ssh-keygen cannot check signatures here, so the signature of the release was not verified, only its SHA-256 hash.", "ssh-keygen ne peut pas vérifier de signature ici, la signature de la publication n'a pas été vérifiée, seulement son empreinte SHA-256."
  release_refused = "released binary refused: {0} Run the script again with -Build, from a clone of the repository, to compile the sources (Rust required).", "binaire publié refusé : {0} Relancez avec -Build, depuis un clone du dépôt, pour compiler les sources (Rust requis)."
  build_failed = "build failed", "échec de la compilation"
  no_sources = "-Build needs the sources: run the script from a clone of the repository", "-Build demande les sources : lancez le script depuis un clone du dépôt"
  swap_failed = "program not replaced, the previous one is kept: {0}", "programme non remplacé, le précédent est conservé : {0}"
  copied = "program copied to {0}", "programme copié dans {0}"
  config_kept = "statusline.json already exists, kept", "statusline.json existe déjà, conservé"
  config_installed = "statusline.json installed", "statusline.json installé"
  registry_unreadable = "account list cannot be read: {0}", "liste des comptes illisible : {0}"
  account_kept = "account left as is: {0}", "compte laissé tel quel : {0}"
  account_missing = "account folder not found, skipped: {0}", "dossier de compte introuvable, ignoré : {0}"
  command_replaced = "command replaced for {0} (was: {1})", "commande remplacée pour {0} (était : {1})"
  done = "done, restart Claude Code", "terminé, relancez Claude Code"
  no_program = "program not found: {0}. Nothing was changed, remove the statusLine block of each account by hand.", "programme introuvable : {0}. Rien n'a été modifié, retirez à la main le bloc statusLine de chaque compte."
  uninstalled = "statusLine removed from the accounts, the program and statusline.json are still there", "statusLine retirée des comptes, le programme et statusline.json sont toujours là"
}
function Get-Text([string]$key) {
  $text = if ($french) { $texts[$key][1] } else { $texts[$key][0] }
  $text -f $args
}

$main = Join-Path $env:USERPROFILE ".claude"
$binDir = Join-Path $main "bin"
$exe = Join-Path $binDir "statusline.exe"
$staged = Join-Path $binDir ("statusline.install-" + $PID + ".exe")
$releaseUrl = "https://github.com/LetermeFlorent/claude-statusline/releases/latest/download/"
if ("$env:STATUSLINE_RELEASE_URL" -cmatch '\Ahttp://127\.0\.0\.1:[0-9]{1,5}/\z') { $releaseUrl = $env:STATUSLINE_RELEASE_URL }
$releaseKey = 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAc/fOxJG/NBMFcBqV6k6pwGdOCfLsloZuAqCWBFmRtN claude-statusline release signing'
$probeKey = 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAINJQEyz/KfjCqflYdMBC6QbZNIGXSyiyKwMALFSLBlOj probe'
$probeSignature = @'
-----BEGIN SSH SIGNATURE-----
U1NIU0lHAAAAAQAAADMAAAALc3NoLWVkMjU1MTkAAAAg0lATLP8p+MKp+Vh0wELpBtk0gZ
dLKLIrAwAsVIsGU6MAAAASc3RhdHVzbGluZS1yZWxlYXNlAAAAAAAAAAZzaGE1MTIAAABT
AAAAC3NzaC1lZDI1NTE5AAAAQGDQeot4d5Dx1rwD4PvikPlszzHbLJLi9U02s28ojvl2eB
6p1Dgr8oaqROcYx988k/36SNWHGORE1c/f7c980ws=
-----END SSH SIGNATURE-----
'@
$asset = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "statusline-windows-arm64.exe" } else { "statusline-windows-x86_64.exe" }
function Get-BinaryLanguage([string]$file) {
  $saved = $ErrorActionPreference
  $ErrorActionPreference = "Continue"
  try { $answer = & $file --language 2>$null | Select-Object -First 1 } catch { $answer = $null } finally { $ErrorActionPreference = $saved }
  return Get-LanguageCode "$answer"
}
function Remove-OldCopies { Get-ChildItem -LiteralPath $binDir -Filter "statusline.exe.old*" | Remove-Item -Force -ErrorAction SilentlyContinue }
function Remove-StaleStaging { Get-ChildItem -LiteralPath $binDir -Filter "statusline.install-*" | Where-Object { $_.FullName -ne $staged -and $_.LastWriteTime -lt (Get-Date).AddMinutes(-10) } | Remove-Item -Force -ErrorAction SilentlyContinue }
function Get-StatusCommand([string]$file) {
  if (-not (Test-Path -LiteralPath $file)) { return $null }
  try { return (Get-Content -LiteralPath $file -Raw -Encoding UTF8 | ConvertFrom-Json).statusLine.command } catch { return $null }
}
function Write-LfFile([string]$path, [string]$text) {
  [IO.File]::WriteAllText($path, ($text -replace "`r`n", "`n"), (New-Object Text.UTF8Encoding $false))
}
function Find-KeyGen {
  $system = Join-Path $env:SystemRoot "System32\OpenSSH\ssh-keygen.exe"
  if (Test-Path -LiteralPath $system) { return $system }
  $found = Get-Command ssh-keygen -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($found) { return $found.Source }
  return $null
}
function Test-Signature([string]$keygen, [string]$key, [string]$message, [string]$signature, [string]$work) {
  Write-LfFile "$work.allowed" "statusline-release namespaces=`"statusline-release`" $key`n"
  $arguments = '-Y verify -f "{0}" -I statusline-release -n statusline-release -s "{1}"' -f "$work.allowed", $signature
  try {
    $run = Start-Process -FilePath $keygen -ArgumentList $arguments -RedirectStandardInput $message -RedirectStandardOutput "$work.out" -RedirectStandardError "$work.err" -Wait -PassThru -NoNewWindow
    return ($run.ExitCode -eq 0)
  } catch { return $false }
}
function Confirm-Manifest([string]$manifest, [string]$signature, [string]$work) {
  $keygen = Find-KeyGen
  Write-LfFile "$work.probe" "statusline probe`n"
  Write-LfFile "$work.probe.sig" ($probeSignature + "`n")
  if ($keygen -and (Test-Signature $keygen $probeKey "$work.probe" "$work.probe.sig" $work)) {
    if (-not (Test-Signature $keygen $releaseKey $manifest $signature $work)) { throw (Get-Text signature) }
  } else {
    Write-Warning (Get-Text no_signature_check)
  }
}
function Get-ManifestHash([string]$manifest) {
  $lines = @(Get-Content -LiteralPath $manifest)
  if ($lines.Count -lt 1 -or $lines[0] -cnotmatch '\Atag v[0-9]+\.[0-9]+\.[0-9]+\z') { throw (Get-Text checksum) }
  $found = @(foreach ($line in ($lines | Select-Object -Skip 1)) {
    $fields = -split $line
    if ($fields.Count -eq 2 -and $fields[1].TrimStart("*") -eq $asset) { $fields[0] }
  })
  if ($found.Count -ne 1 -or $found[0] -cnotmatch '\A[0-9a-f]{64}\z') { throw (Get-Text checksum) }
  return $found[0]
}
function Save-ReleaseBinary([string]$destination) {
  $manifest = $destination + ".manifest"
  $signature = $manifest + ".sig"
  [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
  $ProgressPreference = "SilentlyContinue"
  Write-Host (Get-Text downloading $asset)
  try {
    Invoke-WebRequest -UseBasicParsing -Uri ($releaseUrl + "release-manifest") -OutFile $manifest
    Invoke-WebRequest -UseBasicParsing -Uri ($releaseUrl + "release-manifest.sig") -OutFile $signature
    Invoke-WebRequest -UseBasicParsing -Uri ($releaseUrl + $asset) -OutFile $destination
    Confirm-Manifest $manifest $signature $destination
    $expected = Get-ManifestHash $manifest
    $actual = (Get-FileHash -LiteralPath $destination -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $expected) { throw (Get-Text checksum) }
  } catch {
    Remove-Item -LiteralPath $destination -Force -ErrorAction SilentlyContinue
    throw (Get-Text release_refused $_.Exception.Message)
  } finally {
    foreach ($suffix in ".manifest", ".manifest.sig", ".allowed", ".out", ".err", ".probe", ".probe.sig") {
      Remove-Item -LiteralPath ($destination + $suffix) -Force -ErrorAction SilentlyContinue
    }
  }
}

function Get-Accounts {
  $dirs = [ordered]@{}
  $found = @($main) + @(Get-ChildItem -LiteralPath $env:USERPROFILE -Directory -Filter ".claude-compte*" | ForEach-Object { $_.FullName })
  $reg = Join-Path $env:USERPROFILE ".claude-accounts.json"
  if (Test-Path -LiteralPath $reg) {
    try {
      $list = Get-Content -LiteralPath $reg -Raw -Encoding UTF8 | ConvertFrom-Json
      foreach ($a in $list) {
        if (-not $a.Dir) { continue }
        $listed = Join-Path $env:USERPROFILE $a.Dir
        if (Test-Path -LiteralPath $listed) { $found += $listed } else { Write-Warning (Get-Text account_missing $listed) }
      }
    } catch { Write-Warning (Get-Text registry_unreadable $reg) }
  }
  foreach ($d in $found) {
    if (-not (Test-Path -LiteralPath $d)) { continue }
    $key = $d.TrimEnd("\", "/").ToLowerInvariant()
    if (-not $dirs.Contains($key)) { $dirs[$key] = $d }
  }
  return @($dirs.Values)
}
function Invoke-Accounts([string]$action) {
  $prev = $env:CLAUDE_CONFIG_DIR
  $prevLanguage = $env:STATUSLINE_LANG
  $env:STATUSLINE_LANG = $language
  try {
    foreach ($d in Get-Accounts) {
      $sfile = Join-Path $d "settings.json"
      $old = Get-StatusCommand $sfile
      $env:CLAUDE_CONFIG_DIR = $d
      & $exe $action
      if ($LASTEXITCODE -ne 0) { Write-Warning (Get-Text account_kept $d); continue }
      $new = Get-StatusCommand $sfile
      if ($action -eq "--install" -and $old -and $old -ne $new) { Write-Host (Get-Text command_replaced $d $old) }
    }
  } finally { $env:CLAUDE_CONFIG_DIR = $prev; $env:STATUSLINE_LANG = $prevLanguage }
}

if ($Uninstall) {
  if (-not (Test-Path -LiteralPath $exe)) { throw (Get-Text no_program $exe) }
  $asked = Get-BinaryLanguage $exe
  if ($asked) { $language = $asked; $french = $language -eq "fr" }
  Invoke-Accounts "--uninstall"
  Write-Host (Get-Text uninstalled)
  return
}

New-Item -ItemType Directory -Force -Path $binDir | Out-Null
Remove-StaleStaging
if ($Build) {
  if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot "Cargo.toml"))) { throw (Get-Text no_sources) }
  Push-Location $PSScriptRoot
  try {
    cargo build --release
    if ($LASTEXITCODE -ne 0) { throw (Get-Text build_failed) }
  } finally { Pop-Location }
  $src = Join-Path $PSScriptRoot "target\release\statusline.exe"
} else {
  $src = $staged
  Save-ReleaseBinary $src
}

$backup = $null
try {
  if ($Build) { Copy-Item -LiteralPath $src -Destination $staged -Force }
  Unblock-File -LiteralPath $staged -ErrorAction SilentlyContinue
  if (Test-Path -LiteralPath $exe) {
    Remove-OldCopies
    $backup = $exe + ".old-" + [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()
    Move-Item -LiteralPath $exe -Destination $backup
  }
  Move-Item -LiteralPath $staged -Destination $exe
} catch {
  $reason = $_.Exception.Message
  if ($backup -and -not (Test-Path -LiteralPath $exe) -and (Test-Path -LiteralPath $backup)) {
    Move-Item -LiteralPath $backup -Destination $exe -ErrorAction SilentlyContinue
  }
  Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue
  throw (Get-Text swap_failed $reason)
}
Remove-OldCopies
$asked = Get-BinaryLanguage $exe
if ($asked) { $language = $asked; $french = $language -eq "fr" }
Write-Host (Get-Text copied $exe)

$conf = Join-Path $main "statusline.json"
$template = Join-Path $PSScriptRoot "statusline.json"
if (Test-Path -LiteralPath $conf) { Write-Host (Get-Text config_kept) }
elseif (Test-Path -LiteralPath $template) { Copy-Item -LiteralPath $template -Destination $conf; Write-Host (Get-Text config_installed) }
Invoke-Accounts "--install"
Write-Host (Get-Text done)
