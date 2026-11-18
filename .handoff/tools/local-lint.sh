#!/bin/bash
# usage: local-lint.sh <outdir> <rule-filter-regex|.> proj1,proj2,...
set -e
OUT=$1; FILTER=$2; PROJECTS=$3
WT=/Users/ubugeeei/Source/github.com/ubugeeei-prod/vize.rpm-lane
cd $WT
IFS=, read -ra PS <<< "$PROJECTS"
for p in "${PS[@]}"; do
  if [ -z "$(/bin/ls -A tests/_fixtures/_git/$p 2>/dev/null)" ]; then git submodule update --init --depth 1 -- tests/_fixtures/_git/$p >/dev/null 2>&1 || echo "hydrate failed: $p"; fi
done
mkdir -p $OUT
node tools/commands/fixtures/lint-divergence-report.ts --project "$PROJECTS" --all-mapped-rules --measure-coverage-gap --budget-mode record-only --vize-bin target/ci/vize --output-dir $OUT >$OUT/run.log 2>&1 || { tail -5 $OUT/run.log; exit 1; }
node -e '
const fs=require("fs"),path=require("path");const [dir,filter]=process.argv.slice(1);const re=new RegExp(filter);
for (const f of fs.readdirSync(dir).filter(f=>f.endsWith("-lint-divergence.json")).sort()){
 const a=JSON.parse(fs.readFileSync(path.join(dir,f)));const d=a.divergence;const by={};
 for (const [k,l] of [["FP",d.falsePositives],["FN",d.falseNegatives]]) for (const r of l){ if(!re.test(r.ruleId)) continue; by[r.ruleId]??={FP:0,FN:0}; by[r.ruleId][k]++; }
 console.log(a.project, "total FP="+d.summary.falsePositiveCount, "FN="+d.summary.falseNegativeCount, JSON.stringify(by));
}' $OUT "$FILTER"
