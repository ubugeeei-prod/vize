#!/usr/bin/env pwsh
if ($env:MOONBIT_INSTALL_VERSION) {
  $Version = $env:MOONBIT_INSTALL_VERSION
}
else {
  $Version = "latest"
}

$Version = $Version -replace '\+', '%2B'

if ($env:PROCESSOR_ARCHITECTURE -ne "AMD64" -and $env:PROCESSOR_ARCHITECTURE -ne "ARM64") {
  Write-Output "Install Failed:"
  Write-Output "MoonBit for Windows is currently only available for x86 64-bit && ARM 64-bit Windows.`n"
  exit 1
}

$ErrorActionPreference = "Stop"

if ($env:MOON_HOME) {
  $MoonHome = $env:MOON_HOME
} else {
  $MoonHome = "${HOME}\.moon"
}

$MoonBin = "${MoonHome}\bin"
$MoonLib = "${MoonHome}\lib"
$MoonExe = "${MoonBin}\moon.exe"
$MoonxExe = "${MoonBin}\moonx.exe"

$CLI_MOONBIT = "https://cli.moonbitlang.com"

if ($env:MOONBIT_INSTALL_DEV) {
  $MoonbitUri = "$CLI_MOONBIT/binaries/$Version/moonbit-windows-x86_64-dev.zip"
} else {
  $MoonbitUri = "$CLI_MOONBIT/binaries/$Version/moonbit-windows-x86_64.zip"
}

$CoreUri = "$CLI_MOONBIT/cores/core-$Version.zip"

$oldPreference = $ProgressPreference
$ProgressPreference = 'SilentlyContinue'

try {
  if (-not (Test-Path -Path $MoonHome -PathType Container)) {
    New-Item -Path $MoonHome -ItemType Directory
  }
  Write-Output "Downloading moonbit ..."
  Invoke-WebRequest -Uri $MoonbitUri -OutFile "${HOME}\moonbit.zip"
  if (Test-Path -Path "$MoonHome\lib" -PathType Container) {
    Remove-Item -Force -Recurse "$MoonHome\lib"
  }
  if (Test-Path -Path "$MoonHome\include" -PathType Container) {
    Remove-Item -Force -Recurse "$MoonHome\include"
  }
  Expand-Archive "${HOME}\moonbit.zip" -DestinationPath $MoonHome -Force
  if (Test-Path -LiteralPath $MoonxExe) {
    Remove-Item -Force -LiteralPath $MoonxExe
  }
  New-Item -ItemType HardLink -Path $MoonxExe -Value $MoonExe | Out-Null
  Remove-Item -Force "${HOME}\moonbit.zip"

  Write-Output "Downloading core ..."
  if (Test-Path -Path $MoonLib\core) {
    Remove-Item -Force -Recurse $MoonLib\core
  }
  # Download regular release version
  Invoke-WebRequest -Uri $CoreUri -OutFile $MoonHome\core.zip
  Expand-Archive $MoonHome\core.zip -DestinationPath $MoonLib -Force
  Remove-Item -Force $MoonHome\core.zip

  Write-Output "Bundling core ..."
  Push-Location $MoonLib\core

  $OldPath = $env:Path
  $env:Path = $MoonBin
  moon.exe bundle --warn-list -a --all
  moon.exe bundle --warn-list -a --target wasm-gc --quiet
  $env:PATH = $OldPath

  Pop-Location

  $ProgressPreference = $oldPreference
}
catch {
  $ProgressPreference = $oldPreference
  Write-Output "Install Failed:"
  Write-Output $_.Exception.Message
  exit 1
}

if ($env:Path -split ';' -notcontains $MoonBin) {
  $currentPath = [System.Environment]::GetEnvironmentVariable("PATH", [System.EnvironmentVariableTarget]::User)
  [System.Environment]::SetEnvironmentVariable("PATH", "$MoonBin;$currentPath", [System.EnvironmentVariableTarget]::User)

  Write-Output "Added ~/.moon/bin to the PATH."
  Write-Output "Please restart your terminal to use moonbit."
}
else {
  Write-Output "Moonbit installed successfully."
}

Write-Output "To verify the download binaries, check https://www.moonbitlang.com/download#verifying-binaries for instructions."

Write-Output "To know how to add shell completions, run 'moon shell-completion --help'"
