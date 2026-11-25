#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-criterion/scripts/run-criterion-a-b.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node head/tools/benchmarks/scripts/criterion-ab.ts \
  --base-dir "$GITHUB_WORKSPACE/base" \
  --head-dir "$GITHUB_WORKSPACE/head" \
  --target-dir "$GITHUB_WORKSPACE/head/target" \
  --selection "$GITHUB_WORKSPACE/criterion-impact.json" \
  ${CRITERION_AB_THRESHOLD:+--threshold "$CRITERION_AB_THRESHOLD"} \
  --out "$GITHUB_WORKSPACE/criterion-ab-summary.md"
