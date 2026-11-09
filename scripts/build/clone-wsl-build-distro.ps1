[CmdletBinding()]
param(
    [string]$SourceDistro = 'Debian-Recovered',
    [string]$CloneDistro = 'MarkLinuxBuildTools'
)

$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$targetDirectory = Join-Path $projectRoot 'tools\wsl-build'
$targetVhdx = Join-Path $targetDirectory 'ext4.vhdx'

if (Test-Path -LiteralPath $targetDirectory) {
    throw "Refusing to reuse existing target directory: $targetDirectory"
}

$registrations = Get-ChildItem 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Lxss' |
    ForEach-Object { Get-ItemProperty $_.PSPath }
$sourceRegistration = @($registrations | Where-Object DistributionName -eq $SourceDistro)
if ($sourceRegistration.Count -ne 1) {
    throw "Expected exactly one registered source distribution named $SourceDistro."
}
if ($registrations | Where-Object DistributionName -eq $CloneDistro) {
    throw "A distribution named $CloneDistro is already registered."
}

$sourceBase = [Environment]::ExpandEnvironmentVariables($sourceRegistration[0].BasePath)
$sourceVhdx = Join-Path $sourceBase 'ext4.vhdx'
if (-not (Test-Path -LiteralPath $sourceVhdx -PathType Leaf)) {
    throw "Source WSL VHDX was not found: $sourceVhdx"
}

$sourceBytes = (Get-Item -LiteralPath $sourceVhdx).Length
$driveLetter = (Split-Path -Qualifier $targetDirectory).TrimEnd(':')
$freeBytes = (Get-Volume -DriveLetter $driveLetter).SizeRemaining
$minimumFreeBytes = $sourceBytes + 12GB
if ($freeBytes -lt $minimumFreeBytes) {
    throw "Insufficient free space on $driveLetter`: need source copy plus 12 GiB; have $([math]::Round($freeBytes / 1GB, 1)) GiB."
}

Write-Output "Source: $SourceDistro ($sourceBytes bytes)"
Write-Output "Clone:  $CloneDistro at $targetVhdx"
Write-Output 'The original distribution will remain registered and unchanged.'
New-Item -ItemType Directory -Path $targetDirectory | Out-Null

try {
    Copy-Item -LiteralPath $sourceVhdx -Destination $targetVhdx
    $copiedBytes = (Get-Item -LiteralPath $targetVhdx).Length
    if ($copiedBytes -ne $sourceBytes) {
        throw "VHDX copy size mismatch: source $sourceBytes bytes, copy $copiedBytes bytes."
    }
    $sourceHash = (Get-FileHash -LiteralPath $sourceVhdx -Algorithm SHA256).Hash
    $copyHash = (Get-FileHash -LiteralPath $targetVhdx -Algorithm SHA256).Hash
    if ($sourceHash -ne $copyHash) {
        throw 'VHDX copy SHA-256 verification failed.'
    }
    Write-Output "VHDX SHA-256 verified: $sourceHash"

    & wsl.exe --import-in-place $CloneDistro $targetVhdx
    if ($LASTEXITCODE -ne 0) {
        throw "wsl --import-in-place failed with exit code $LASTEXITCODE. The verified copy is retained at $targetVhdx."
    }
    & wsl.exe --distribution $CloneDistro --user root --exec /bin/true
    if ($LASTEXITCODE -ne 0) {
        throw "The clone was registered but did not start (exit code $LASTEXITCODE). The original distro remains unchanged; the clone is retained for recovery."
    }
    & wsl.exe --distribution $CloneDistro --user root --exec /usr/bin/df -h / /var/tmp
    if ($LASTEXITCODE -ne 0) {
        throw "The clone started but its storage check failed (exit code $LASTEXITCODE). The clone is retained for recovery."
    }
}
catch {
    if (Test-Path -LiteralPath $targetVhdx) {
        Write-Warning "Preserved the build copy for inspection at $targetVhdx"
    }
    throw
}

Write-Output "Build-only WSL copy is ready. Original $SourceDistro remains registered on its existing path."
