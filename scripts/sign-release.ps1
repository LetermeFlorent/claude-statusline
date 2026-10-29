[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Tag,
    [string]$Key = (Join-Path $env:USERPROFILE '.ssh\statusline-release'),
    [string]$Repo = 'LetermeFlorent/claude-statusline',
    [switch]$Replace
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
Add-Type -AssemblyName System.Net.Http

$PublicKey = 'ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIAc/fOxJG/NBMFcBqV6k6pwGdOCfLsloZuAqCWBFmRtN'
$Principal = 'statusline-release'
$Binaries = @(
    'statusline-windows-x86_64.exe',
    'statusline-windows-arm64.exe',
    'statusline-linux-x86_64',
    'statusline-linux-arm64',
    'statusline-macos-arm64',
    'statusline-macos-x86_64'
)
$ManifestName = 'release-manifest'
$SigName = 'release-manifest.sig'
$Utf8 = New-Object System.Text.UTF8Encoding($false)

function Get-ReleaseToken {
    if ($env:GITHUB_TOKEN) { return $env:GITHUB_TOKEN }
    $secure = Read-Host -Prompt 'GitHub token' -AsSecureString
    $bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try { return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr) }
    finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr) }
}

function Get-ApiHeaders($token) {
    return @{
        Authorization          = "Bearer $token"
        Accept                 = 'application/vnd.github+json'
        'X-GitHub-Api-Version' = '2022-11-28'
        'User-Agent'           = 'statusline-sign-release'
    }
}

function Find-SshKeygen {
    $bundled = Join-Path $env:SystemRoot 'System32\OpenSSH\ssh-keygen.exe'
    if (Test-Path -LiteralPath $bundled) { return $bundled }
    $found = Get-Command ssh-keygen -ErrorAction SilentlyContinue
    if ($found) { return $found.Source }
    throw 'ssh-keygen was not found.'
}

function Find-DraftRelease($headers) {
    for ($page = 1; $page -le 5; $page++) {
        $url = "https://api.github.com/repos/$Repo/releases?per_page=100&page=$page"
        $list = @(Invoke-RestMethod -Uri $url -Headers $headers -Method Get)
        if ($list.Count -eq 0) { return $null }
        foreach ($release in $list) {
            if ($release.tag_name -eq $Tag) { return $release }
        }
    }
    return $null
}

function Save-Asset($asset, $token, $folder) {
    $target = Join-Path $folder $asset.name
    $client = New-Object System.Net.Http.HttpClient(
        (New-Object System.Net.Http.HttpClientHandler -Property @{ AllowAutoRedirect = $false }))
    try {
        $request = New-Object System.Net.Http.HttpRequestMessage([Net.Http.HttpMethod]::Get, $asset.url)
        $request.Headers.TryAddWithoutValidation('Authorization', "Bearer $token") | Out-Null
        $request.Headers.TryAddWithoutValidation('Accept', 'application/octet-stream') | Out-Null
        $request.Headers.TryAddWithoutValidation('User-Agent', 'statusline-sign-release') | Out-Null
        $response = $client.SendAsync($request, [Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
        if ([int]$response.StatusCode -ge 300 -and [int]$response.StatusCode -lt 400) {
            $location = $response.Headers.Location
            $response.Dispose()
            $request = New-Object System.Net.Http.HttpRequestMessage([Net.Http.HttpMethod]::Get, $location)
            $request.Headers.TryAddWithoutValidation('User-Agent', 'statusline-sign-release') | Out-Null
            $response = $client.SendAsync($request, [Net.Http.HttpCompletionOption]::ResponseHeadersRead).GetAwaiter().GetResult()
        }
        if (-not $response.IsSuccessStatusCode) {
            throw "Download of $($asset.name) failed with HTTP $([int]$response.StatusCode)."
        }
        $source = $response.Content.ReadAsStreamAsync().GetAwaiter().GetResult()
        $sink = [IO.File]::Create($target)
        try { $source.CopyTo($sink) }
        finally { $sink.Dispose(); $source.Dispose() }
        $response.Dispose()
    }
    finally { $client.Dispose() }
    return $target
}

function Test-Manifest($path, $folder) {
    $text = $Utf8.GetString([IO.File]::ReadAllBytes($path))
    if ($text.Contains("`r")) { throw 'The manifest contains carriage returns.' }
    if (-not $text.EndsWith("`n")) { throw 'The manifest does not end with a newline.' }
    $lines = $text.Substring(0, $text.Length - 1).Split("`n")
    if ($lines.Count -lt 2) { throw 'The manifest has no file lines.' }
    if ($lines[0] -ne "tag $Tag") { throw "The manifest first line is not 'tag $Tag'." }
    $seen = @{}
    foreach ($line in $lines[1..($lines.Count - 1)]) {
        if ($line -notmatch '^([0-9a-f]{64})  (\S+)$') { throw "Malformed manifest line: $line" }
        $hash = $Matches[1]
        $name = $Matches[2]
        if ($Binaries -notcontains $name) { throw "Unexpected file in the manifest: $name" }
        if ($seen.ContainsKey($name)) { throw "Listed twice in the manifest: $name" }
        $seen[$name] = $true
        $actual = (Get-FileHash -LiteralPath (Join-Path $folder $name) -Algorithm SHA256).Hash.ToLower()
        if ($actual -ne $hash) { throw "SHA-256 mismatch for $name." }
    }
    foreach ($name in $Binaries) {
        if (-not $seen.ContainsKey($name)) { throw "Missing from the manifest: $name" }
    }
    return $text
}

function Invoke-Tool($exe, $argumentLine, $folder, $stdin, $stdout, $stderr) {
    $params = @{
        FilePath         = $exe
        ArgumentList     = $argumentLine
        WorkingDirectory = $folder
        Wait             = $true
        PassThru         = $true
        NoNewWindow      = $true
    }
    if ($stdin) { $params.RedirectStandardInput = $stdin }
    if ($stdout) { $params.RedirectStandardOutput = $stdout }
    if ($stderr) { $params.RedirectStandardError = $stderr }
    return (Start-Process @params).ExitCode
}

if ($Tag -notmatch '^v\d+\.\d+\.\d+$') { throw "Tag must look like v1.2.3, got '$Tag'." }
if (-not (Test-Path -LiteralPath $Key)) { throw "Private key not found: $Key" }
$Key = (Resolve-Path -LiteralPath $Key).ProviderPath
$sshKeygen = Find-SshKeygen
$token = Get-ReleaseToken
$headers = Get-ApiHeaders $token
$work = Join-Path ([IO.Path]::GetTempPath()) ('sign-release-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null

try {
    $release = Find-DraftRelease $headers
    if (-not $release) { throw "No release with tag $Tag was found." }
    if (-not $release.draft) { throw "Release $Tag is not a draft. Refusing to touch it." }

    $names = @($release.assets | ForEach-Object { $_.name })
    $allowedExtra = @($ManifestName, $SigName, 'SHA256SUMS')
    foreach ($name in $names) {
        if (($Binaries -notcontains $name) -and ($allowedExtra -notcontains $name)) {
            throw "Unexpected asset on the draft: $name"
        }
    }
    foreach ($name in ($Binaries + $ManifestName)) {
        if ($names -notcontains $name) { throw "The draft has no asset named $name." }
    }

    $existing = @($release.assets | Where-Object { $_.name -eq $SigName })
    if ($existing.Count -gt 0 -and -not $Replace) {
        throw "$SigName is already on the draft. Use -Replace to overwrite it."
    }

    foreach ($asset in $release.assets) {
        if (($Binaries -contains $asset.name) -or $asset.name -eq $ManifestName) {
            Write-Host "Downloading $($asset.name)"
            Save-Asset $asset $token $work | Out-Null
        }
    }

    $manifestPath = Join-Path $work $ManifestName
    $manifestText = Test-Manifest $manifestPath $work
    Write-Host ''
    Write-Host $manifestText
    $answer = Read-Host "The checksums match. Sign this manifest for $Tag? (y/N)"
    if ($answer -ne 'y') { throw 'Cancelled.' }

    $signCode = Invoke-Tool $sshKeygen "-Y sign -n $Principal -f `"$Key`" $ManifestName" $work $null $null $null
    $sigPath = Join-Path $work $SigName
    if ($signCode -ne 0 -or -not (Test-Path -LiteralPath $sigPath)) { throw 'Signing failed.' }

    $signers = Join-Path $work 'allowed_signers'
    [IO.File]::WriteAllText($signers, "$Principal namespaces=`"$Principal`" $PublicKey`n", $Utf8)
    $verifyArgs = "-Y verify -f `"$signers`" -I $Principal -n $Principal -s `"$sigPath`""
    $verifyOut = Join-Path $work 'verify.out'
    $verifyErr = Join-Path $work 'verify.err'
    $verifyCode = Invoke-Tool $sshKeygen $verifyArgs $work $manifestPath $verifyOut $verifyErr
    if ($verifyCode -ne 0) {
        throw 'The new signature does not verify against the embedded public key. The private key does not match it.'
    }
    Write-Host 'Signature verified against the embedded public key.'

    foreach ($asset in $existing) {
        Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/assets/$($asset.id)" -Headers $headers -Method Delete | Out-Null
    }

    $uploadUrl = "https://uploads.github.com/repos/$Repo/releases/$($release.id)/assets?name=$SigName"
    Invoke-RestMethod -Uri $uploadUrl -Headers $headers -Method Post -ContentType 'application/octet-stream' -InFile $sigPath | Out-Null

    Write-Host ''
    Write-Host "Uploaded $SigName to the draft for $Tag."
    Write-Host $release.html_url
    Write-Host 'The release is still a draft. Read it, then publish it by hand as the latest release.'
}
finally {
    $token = $null
    $headers = $null
    Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
