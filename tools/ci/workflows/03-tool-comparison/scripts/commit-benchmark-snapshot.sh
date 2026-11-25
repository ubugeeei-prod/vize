#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-tool-comparison/scripts/commit-benchmark-snapshot.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node tools/benchmarks/scripts/publish-snapshot.mjs --json tool-benchmark-artifact/tool-benchmark-results.json
node tools/benchmarks/scripts/publish-snapshot.mjs --check
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"
git add README.md tools/benchmarks/results/tool-benchmark-latest.json \
  docs/content/architecture/performance{,-blacksmith}.md \
  docs/content/{ja,zh-CN,fr,pt-BR}/architecture/performance{,-blacksmith}.md
if git diff --cached --quiet; then
  echo "No benchmark snapshot changes to commit."
else
  git commit -m "docs: update blacksmith benchmark snapshot"
  git push origin "HEAD:refs/heads/$VIZE_BENCH_REF"
fi
