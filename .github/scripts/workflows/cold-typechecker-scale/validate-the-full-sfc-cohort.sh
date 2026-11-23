#!/usr/bin/env bash
[[ "$VIZE_BENCH_SFCS" =~ ^(2500|10000|50000)$ ]]
baseline_selection=$(node tools/benchmarks/scripts/cold-typechecker-scale-build.ts --select-baseline --receipt "$RUNNER_TEMP/cold-scale/baseline-selection.json")
[[ "$baseline_selection" =~ ^BASE_SHA=[0-9a-f]{40}$ ]]
printf '%s\n' "$baseline_selection" >> "$GITHUB_ENV"
bench_support_scripts=$(node "$VIZE_CI_FILES/validate-the-full-sfc-cohort.ts")
printf 'VIZE_BENCH_FILES=%s\n' "$((VIZE_BENCH_SFCS + bench_support_scripts))" >> "$GITHUB_ENV"
