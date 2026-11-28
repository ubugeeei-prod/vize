# DO NOT EDIT. Generated from tools/ci/workflows/03-typecheck-compare/scripts/mapper-measure.ps1.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if ($env:SIZES -notmatch '^[0-9]+(,[0-9]+)*$' -or $env:RUNS -notmatch '^[1-9][0-9]?$' -or $env:GIANT_BLOCKS -notmatch '^[0-9]+$') { throw "invalid inputs" }
$out = Join-Path $env:RUNNER_TEMP 'pair-out'
$extra = @(); if ($env:ALLOW) { $extra += $env:ALLOW }
if ($env:REFERENCE -eq 'true') {
  npm ci --prefix tools/benchmarks/typecheck-hosts --ignore-scripts --no-audit --no-fund | Out-Null
  if ($LASTEXITCODE -ne 0) { throw "hosts install failed" }
  $hosts = "$PWD\tools\benchmarks\typecheck-hosts\node_modules"
  $extra += @('--reference', "vue-mapper=$hosts\@vue\content-mapper\server.js", '--typescript', "$hosts\typescript")
}
node tools/benchmarks/scripts/content-mapper-pair.ts `
  --bin "base=$PWD\pair-bin\base\vize.exe" --bin "head=$PWD\pair-bin\head\vize.exe" `
  --root (Join-Path $env:RUNNER_TEMP 'pair') --sizes $env:SIZES --giant-blocks $env:GIANT_BLOCKS --runs $env:RUNS @extra `
  --label windows-ntfs --out $out | Out-Null
if ($LASTEXITCODE -ne 0) { throw "pair failed" }
Get-Content (Join-Path $out 'pair-windows-ntfs.md') | Add-Content $env:GITHUB_STEP_SUMMARY
