#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/ci/workflows/03-criterion/scripts/compare-vapor-l3-retained-and-official-compiler-on-exact-head.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
node head/tools/benchmarks/scripts/vapor-compiler-compare.ts \
  --head-sha "$HEAD_SHA" \
  --criterion "$GITHUB_WORKSPACE/head/target/criterion-ab-head.json" \
  --out "$GITHUB_WORKSPACE/vapor-compiler-comparison.json" \
  --summary "$GITHUB_WORKSPACE/vapor-compiler-comparison.md"
