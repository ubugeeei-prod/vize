#!/usr/bin/env bash
# DO NOT EDIT. Generated from tools/config/pkl/scripts/.github/scripts/workflows/criterion-bench/select-affected-criterion-suites.sh.pkl.
# Edit the Pkl source and run `node tools/commands/generate/repository-config.ts`.
if [[ "$VAPOR_ONLY" == "true" ]]; then
  printf '%s\n' '{"selected":["vize_atelier_vapor"],"reason":"Explicit Davinci Vapor native/retained comparison."}' > "$GITHUB_WORKSPACE/criterion-impact.json"
  echo 'has_suites=true' >> "$GITHUB_OUTPUT"
else
  node head/tools/benchmarks/scripts/criterion-impact.ts \
    --repo-dir "$GITHUB_WORKSPACE/head" \
    --base-sha "$PR_BASE_SHA" \
    --head-sha "$PR_HEAD_SHA" \
    --out "$GITHUB_WORKSPACE/criterion-impact.json"
fi
if node "$VIZE_CI_FILES/select-affected-criterion-suites.cjs" "$GITHUB_WORKSPACE/criterion-impact.json"; then
  echo 'vapor_selected=true' >> "$GITHUB_OUTPUT"
fi
