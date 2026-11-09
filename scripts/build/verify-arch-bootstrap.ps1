[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$lockPath = Join-Path $projectRoot 'tools\build\arch-bootstrap.lock.json'
$lock = Get-Content -Raw -LiteralPath $lockPath | ConvertFrom-Json
$cache = Join-Path $projectRoot 'builds\cache'
New-Item -ItemType Directory -Path $cache -Force | Out-Null
$archive = Join-Path $cache $lock.archive
$signature = Join-Path $cache $lock.signature
$base = "https://fastly.mirror.pkgbuild.com/iso/$($lock.release)"
$curl = (Get-Command curl.exe -ErrorAction Stop).Source

function Get-VerifiedDownload([string]$url,[string]$path) {
    if (Test-Path -LiteralPath $path) { return }
    $partial = "$path.download"
    if (Test-Path -LiteralPath $partial) { throw "Partial download already exists; inspect before retrying: $partial" }
    & $curl --fail --location --retry 3 --output $partial $url
    if ($LASTEXITCODE -ne 0) { throw "Download failed with exit code $LASTEXITCODE : $url" }
    Move-Item -LiteralPath $partial -Destination $path
}

Get-VerifiedDownload "$base/$($lock.archive)" $archive
Get-VerifiedDownload "$base/$($lock.signature)" $signature
$actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
if ($actual -ne $lock.sha256) { throw "Bootstrap SHA-256 mismatch. Expected $($lock.sha256); got $actual. Archive will not be used." }

$gpg = (Get-Command gpg.exe -ErrorAction Stop).Source
$keyHome = Join-Path $cache 'gnupg'
New-Item -ItemType Directory -Path $keyHome -Force | Out-Null
# The detected GnuPG is Git's MSYS build; pass drive paths in its POSIX form.
$driveLetter = $keyHome.Substring(0,1).ToLowerInvariant()
$keyHomePosix = "/$driveLetter/" + $keyHome.Substring(3).Replace('\','/')
$archivePosix = "/$($archive.Substring(0,1).ToLowerInvariant())/" + $archive.Substring(3).Replace('\','/')
$signaturePosix = "/$($signature.Substring(0,1).ToLowerInvariant())/" + $signature.Substring(3).Replace('\','/')
& $gpg --homedir $keyHomePosix --auto-key-locate clear,wkd --locate-external-key $lock.signer_email
if ($LASTEXITCODE -ne 0) { throw 'Could not retrieve the Arch release signing key through WKD.' }
$colon = @(& $gpg --homedir $keyHomePosix --with-colons --fingerprint --list-keys $lock.signer_email)
$fprLine = $colon | Where-Object { $_ -like 'fpr:*' } | Select-Object -First 1
if (!$fprLine) { throw 'No primary fingerprint was returned for the release signer.' }
$fingerprint = ($fprLine -split ':')[9]
if (!$fingerprint.EndsWith($lock.expected_signing_key_id_suffix,[StringComparison]::OrdinalIgnoreCase)) {
    throw "Unexpected release key fingerprint: $fingerprint"
}
& $gpg --homedir $keyHomePosix --verify $signaturePosix $archivePosix
if ($LASTEXITCODE -ne 0) { throw 'Bootstrap signature verification failed.' }
Write-Host "Verified $($lock.archive)"
Write-Host "SHA-256: $actual"
Write-Host "Signer fingerprint: $fingerprint"