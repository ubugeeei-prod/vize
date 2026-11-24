# DO NOT EDIT. Generated from tools/config/pkl/scripts/tools/benchmarks/scripts/content-mapper-burst-windows.ps1.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
param(
  [Parameter(Mandatory)][string]$Executable,
  [Parameter(Mandatory)][string]$MapperArgument,
  [Parameter(Mandatory)][string]$Project,
  [Parameter(Mandatory)][string]$InputFile,
  [Parameter(Mandatory)][string]$OutputFile
)
$ErrorActionPreference = 'Stop'

$start = @{
  FilePath = $Executable
  # The sole argument is either the content-mapper subcommand or a Windows file path.
  ArgumentList = '"{0}"' -f $MapperArgument
  WorkingDirectory = $Project
  RedirectStandardInput = $InputFile
  RedirectStandardOutput = $OutputFile
  NoNewWindow = $true
  PassThru = $true
  Wait = $true
}
$clock = [System.Diagnostics.Stopwatch]::StartNew()
$child = Start-Process @start
$clock.Stop()
@{
  exitCode = $child.ExitCode
  cpuSeconds = $child.TotalProcessorTime.TotalSeconds
  peakBytes = $child.PeakWorkingSet64
  wallMs = $clock.Elapsed.TotalMilliseconds
} | ConvertTo-Json -Compress
