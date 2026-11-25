# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/measure.ps1.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if ($env:PROJECTS -notmatch '^(vue|monorepo):[0-9]+(,(vue|monorepo):[0-9]+)*$' -or $env:RUNS -notmatch '^[1-9][0-9]?$' -or $env:RUNTIME -notmatch '^(tsgo|tsrs)$' -or $env:CHECKERS -notmatch '^(1|8)$') { throw "invalid inputs" }
$out = Join-Path $env:RUNNER_TEMP 'pair-out'
$extra = @(); if ($env:ALLOW) { $extra += $env:ALLOW }
node tools/benchmarks/scripts/typecheck-cold-pair.ts `
  --bin "base=$PWD\pair-bin\base\vize.exe" --bin "head=$PWD\pair-bin\head\vize.exe" `
  --root (Join-Path $env:RUNNER_TEMP 'pair') --projects $env:PROJECTS --runs $env:RUNS `
  --runtime $env:RUNTIME --checkers $env:CHECKERS @extra `
  --label "windows-ntfs-$($env:RUNTIME)-c$($env:CHECKERS)" --out $out | Out-Null
if ($LASTEXITCODE -ne 0) { throw "pair failed" }
Get-Content (Join-Path $out "cold-pair-windows-ntfs-$($env:RUNTIME)-c$($env:CHECKERS).md") | Add-Content $env:GITHUB_STEP_SUMMARY
