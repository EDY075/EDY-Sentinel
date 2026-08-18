[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$BundleRoot,

    [Parameter(Mandatory = $true)]
    [string]$ExecutablePath,

    [switch]$RequireTimestamp
)

$ErrorActionPreference = 'Stop'
$resolvedBundleRoot = (Resolve-Path -LiteralPath $BundleRoot).Path
$resolvedExecutable = (Resolve-Path -LiteralPath $ExecutablePath).Path
$artifacts = @($resolvedExecutable)
$artifacts += @(Get-ChildItem -LiteralPath (Join-Path $resolvedBundleRoot 'msi') -Filter '*.msi' -File -ErrorAction Stop | Select-Object -ExpandProperty FullName)
$artifacts += @(Get-ChildItem -LiteralPath (Join-Path $resolvedBundleRoot 'nsis') -Filter '*-setup.exe' -File -ErrorAction Stop | Select-Object -ExpandProperty FullName)
$artifacts = @($artifacts | Sort-Object -Unique)

if ($artifacts.Count -ne 3) {
    throw "Expected exactly the application EXE, one MSI, and one NSIS installer; found $($artifacts.Count)."
}

foreach ($artifact in $artifacts) {
    $signature = Get-AuthenticodeSignature -LiteralPath $artifact
    if ($signature.Status -ne 'Valid' -or $null -eq $signature.SignerCertificate) {
        throw "Authenticode verification failed for $artifact with status $($signature.Status)."
    }
    if ($RequireTimestamp -and $null -eq $signature.TimeStamperCertificate) {
        throw "A trusted timestamp was not found for $artifact."
    }
    Write-Host "VALID AUTHENTICODE: $artifact"
}
