Param(
  [Parameter(Mandatory=$true)][string]$CaUrl,
  [Parameter(Mandatory=$true)][string]$DestPath,
  [string]$RestartCmd
)
$Temp = [System.IO.Path]::GetTempFileName()
try {
  Invoke-WebRequest -UseBasicParsing -Uri $CaUrl -OutFile $Temp
  $content = Get-Content $Temp -Raw
  if ($content -notmatch 'BEGIN CERTIFICATE') {
    Write-Error "Fetched file does not look like a PEM certificate"
    exit 3
  }
  $dir = Split-Path $DestPath -Parent
  if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir | Out-Null }
  Move-Item -Force $Temp ($DestPath + ".tmp")
  Move-Item -Force ($DestPath + ".tmp") $DestPath
  Write-Output "Replaced CA at $DestPath"
  if ($RestartCmd) {
    Write-Output "Running restart command: $RestartCmd"
    iex $RestartCmd
  }
} finally {
  if (Test-Path $Temp) { Remove-Item $Temp -ErrorAction SilentlyContinue }
}
