# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/content-mapper-pair/resolve-the-windows-mount.ps1.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
$windowsPath = [System.IO.Path]::GetFullPath($env:RUNNER_TEMP)
$drive = $windowsPath.Substring(0, 1).ToLowerInvariant()
$tail = $windowsPath.Substring(2).Replace('\', '/')
$mount = "/mnt/$drive$tail"
wsl.exe test -d $mount
if ($LASTEXITCODE -ne 0) { throw "WSL mount is unavailable: $mount" }
"WSL_MOUNT_ROOT=$mount/vize-pair" | Out-File -FilePath $env:GITHUB_ENV -Append
