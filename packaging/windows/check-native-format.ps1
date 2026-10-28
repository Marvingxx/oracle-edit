param(
  [Parameter(Mandatory=$true)][string] $Cli,
  [Parameter(Mandatory=$true)][string] $ExpectedVersion
)
$ErrorActionPreference = 'Stop'
$Cli = (Resolve-Path -LiteralPath $Cli).Path
function Invoke-Cli([string[]] $CliArgs) {
  $output = & $Cli @CliArgs 2>&1
  if ($LASTEXITCODE -ne 0) { throw "CLI failed ($LASTEXITCODE): $($CliArgs -join ' ')`n$output" }
  return ($output -join "`n")
}
$version = Invoke-Cli -CliArgs @('--version')
if ($version -notmatch ('^oracle-edit-cli ' + [regex]::Escape($ExpectedVersion) + '(\s|$)')) {
  throw "CLI version mismatch: expected $ExpectedVersion, received $version"
}
$tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
$work = [IO.Path]::GetFullPath((Join-Path $tempRoot ('oracle-edit-package-check-' + [guid]::NewGuid().ToString('N'))))
if (-not $work.StartsWith(($tempRoot.TrimEnd('\') + '\'), [StringComparison]::OrdinalIgnoreCase)) {
  throw "unsafe temporary path: $work"
}
New-Item -ItemType Directory -Path $work | Out-Null
try {
  $native = Join-Path $work 'document.oracle-edit'
  $png = Join-Path $work 'roundtrip.png'
  $source = Join-Path $work 'source.png'
  Add-Type -AssemblyName System.Drawing
  $image = New-Object System.Drawing.Bitmap 8, 6
  try {
    $image.SetPixel(0, 0, [System.Drawing.Color]::Lime)
    $image.Save($source, [System.Drawing.Imaging.ImageFormat]::Png)
  } finally {
    $image.Dispose()
  }
  Invoke-Cli -CliArgs @('convert', $source, $native) | Out-Null
  Invoke-Cli -CliArgs @('convert', $native, $png) | Out-Null
  $info = (Invoke-Cli -CliArgs @('info', $png, '--compact')) | ConvertFrom-Json
  if ($info.width -ne 8 -or $info.height -ne 6) { throw 'native round trip changed document dimensions' }
  $legacy = Join-Path $work 'legacy.pcraft'
  Copy-Item -LiteralPath $native -Destination $legacy
  $migrated = Join-Path $work 'migrated.oracle-edit'
  Invoke-Cli -CliArgs @('convert', $legacy, $migrated) | Out-Null
  Invoke-Cli -CliArgs @('info', $migrated, '--compact') | Out-Null
  Write-Output "ok Oracle Edit ${ExpectedVersion}: native save/open/export and legacy import"
} finally {
  Remove-Item -LiteralPath $work -Recurse -Force
}
